// Compose + Output panels. All scenario fields come from list_scenarios (FieldMeta); nothing hard-coded.
import common from '../data/common.json';
import { setEditorText, scheduleCheck } from './editor.js';
const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);
// Recipients come from data/common.json so new ones (e.g. client) appear automatically.
const RECIPIENTS = Object.entries(common.recipients ?? {}).map(([id, r]) => [id, (r.label_en ?? id).split(' /')[0]]);
const CHANNELS = [['email', 'Email'], ['chat', 'Chat']];

const state = { scenarios: [], scenario: null, recipient: 'professor', channel: 'email', lang: 'ja', fields: {}, toggles: {}, auto: {} }; // auto: key -> analyze guess that filled it
function el(tag, { dataset, ...props } = {}, ...kids) {
  const n = Object.assign(document.createElement(tag), props);
  if (dataset) Object.assign(n.dataset, dataset);
  n.append(...kids);
  return n;
}

const composeEl = el('div', { className: 'compose' });
const formEl = el('div');

// "Received message" box: analyze() guesses -> prefill the form, marking what was auto-filled.
const receivedText = el('textarea', { className: 'received', lang: 'ja', rows: 4, placeholder: 'Paste the message you received…' });
const receivedNote = el('div', { className: 'rnote' });
const fillBtn = el('button', { type: 'button', textContent: 'Fill form from message' });
fillBtn.onclick = async () => {
  if (!receivedText.value.trim()) return;
  try {
    const { guesses } = await invoke('analyze', { received: receivedText.value });
    state.auto = {};
    const skipped = [];
    for (const g of guesses) {
      const key = g.field.replace(/^fields\./, '');
      if (key === 'scenario' && !state.scenarios.some((s) => s.id === g.value)) { skipped.push(g); continue; }
      if (key === 'recipient' && !RECIPIENTS.some(([id]) => id === g.value)) { skipped.push(g); continue; }
      if (['scenario', 'recipient', 'channel'].includes(key)) state[key] = g.value;
      else state.fields[key] = g.value;
      state.auto[key] = g;
    }
    const shown = new Set(['scenario', 'recipient', 'channel', ...(current()?.fields.filter(visible).map((f) => f.key) ?? [])]);
    const n = Object.keys(state.auto).filter((k) => shown.has(k)).length; // a guess for a field this scenario lacks isn't shown
    receivedNote.textContent = n ? `Auto-filled ${n} item(s) — marked "auto". Hover a mark for the evidence.` : 'Nothing recognised; fill the form manually.';
    if (skipped.length) receivedNote.textContent += ` Skipped: ${skipped.map((g) => `${g.field}=${g.value}`).join(', ')}`;
    renderForm(); schedule();
  } catch (err) { receivedNote.textContent = `Analyze failed: ${err}`; }
};
// Decode: explain the received message (hidden requests, deadlines, soft refusals, keigo verbs).
const KIND_LABEL = { request: 'They ask you to', deadline: 'Deadline', refusal: 'Soft no / don\'t', cushion: 'Softener', info: 'Info', honorific: 'Keigo verb' };
const decodeBtn = el('button', { type: 'button', textContent: 'Decode message' });
const decodeEl = el('div', { className: 'decode' });
decodeBtn.onclick = async () => {
  const text = receivedText.value;
  if (!text.trim()) return;
  try {
    const { items, summary } = await invoke('decode', { received: text });
    const marked = el('div', { className: 'decoded', lang: 'ja' });
    let pos = 0; // spans are UTF-16 offsets, same as JS string indices
    for (const i of items) {
      marked.append(text.slice(pos, i.start), el('mark', { className: `k-${i.kind}`, textContent: text.slice(i.start, i.end), title: `${i.plain}\n${i.rule_id} · ${i.why}` }));
      pos = i.end;
    }
    marked.append(text.slice(pos));
    const list = el('ul', { className: 'decode-summary' }, ...summary.map((s) =>
      el('li', { className: `k-${s.kind}` }, el('b', { textContent: `${KIND_LABEL[s.kind] ?? s.kind}: ` }), s.plain, el('div', { className: 'src', lang: 'ja', textContent: s.sentence }))));
    decodeEl.replaceChildren(
      summary.length ? el('h4', { textContent: 'What this message wants' }) : el('p', { textContent: 'No requests, deadlines or refusals found.' }), list,
      el('h4', { textContent: `Annotated (${items.length}) — hover a highlight` }), marked);
  } catch (err) { decodeEl.textContent = `Decode failed: ${err}`; }
};
composeEl.append(el('details', { className: 'received-box', open: true },
  el('summary', { textContent: 'Received message (optional)' }), receivedText, fillBtn, decodeBtn, receivedNote, decodeEl), formEl);

const autoBadge = (key) => {
  const g = state.auto[key];
  return g ? [el('span', { className: 'auto', textContent: 'auto', title: `「${g.evidence}」 · ${g.rule_id}` })] : [];
};
const outputEl = el('div', { className: 'output' });
let seq = 0, timer = 0, last = null, glossOpen = false;

const current = () => state.scenarios.find((s) => s.id === state.scenario);
// Same derivation as engine derive_context(): rank/in_group from the recipient, politeness from the first matching rule.
const matches = (when, ctx) => Object.entries(when).every(([k, v]) => k in ctx && (Array.isArray(v) ? v.includes(ctx[k]) : ctx[k] === v));
function derived() {
  const ctx = { recipient: state.recipient, channel: state.channel, lang: state.lang };
  const p = common.recipients?.[state.recipient];
  if (p) Object.assign(ctx, { rank: p.rank, in_group: String(p.in_group) });
  const rule = (common.politeness ?? []).find((r) => matches(r.when, ctx));
  if (rule) ctx.politeness = rule.value;
  return ctx;
}
const ctxValue = (key) => {
  const d = derived();
  if (key in d) return d[key];
  if (key in state.toggles) return String(state.toggles[key]);
  return state.fields[key];
};
const visible = (f) => Object.entries(f.show_if || {}).every(([k, v]) => ctxValue(k) === v);

function segmented(name, options, key) {
  const group = el('div', { className: 'seg', role: 'group', ariaLabel: name });
  for (const [value, label] of options) {
    const b = el('button', { type: 'button', textContent: label, className: state[key] === value ? 'on' : '' });
    b.setAttribute('aria-pressed', state[key] === value);
    b.onclick = () => { state[key] = value; delete state.auto[key]; renderForm(); schedule(); };
    group.append(b);
  }
  return el('div', { className: 'row' }, el('span', { className: 'lbl' }, name, ...autoBadge(key)), group);
}

function fieldInput(f) {
  const id = `f-${f.key}`;
  let input;
  if (f.kind === 'select') {
    input = el('select', { id }, el('option', { value: '', textContent: '—' }),
      ...f.options.map((o) => el('option', { value: o, textContent: o.replaceAll('_', ' ') })));
    input.value = state.fields[f.key] ?? '';
  } else if (f.kind === 'toggle') {
    input = el('input', { id, type: 'checkbox', checked: !!state.toggles[f.key] });
  } else {
    input = el('input', { id, type: f.kind === 'number' ? 'number' : 'text', value: state.fields[f.key] ?? '' });
    if (f.kind === 'number') input.min = 0;
  }
  input.oninput = input.onchange = () => {
    if (state.auto[f.key]) { delete state.auto[f.key]; wrapEl.classList.remove('is-auto'); wrapEl.querySelector('.auto')?.remove(); }
    if (f.kind === 'toggle') state.toggles[f.key] = input.checked;
    else state.fields[f.key] = input.value;
    if (f.kind === 'select' || f.kind === 'toggle') renderForm(); // show_if may depend on it
    schedule();
  };
  const label = el('label', { htmlFor: id, textContent: f.label + (f.required ? ' *' : '') }, ...autoBadge(f.key));
  const wrapEl = el('div', { className: `field kind-${f.kind}${state.auto[f.key] ? ' is-auto' : ''}`, dataset: { key: f.key } }, label, input, el('div', { className: 'ferr' }));
  return wrapEl;
}

function renderForm() {
  const sc = current();
  const picker = el('select', { id: 'scenario' }, ...state.scenarios.map((s) =>
    el('option', { value: s.id, textContent: `${s.label_en} · ${s.label_ja}` })));
  picker.value = state.scenario ?? '';
  picker.onchange = () => { state.scenario = picker.value; delete state.auto.scenario; renderForm(); schedule(); };
  formEl.replaceChildren(
    el('div', { className: 'row' }, el('label', { className: 'lbl', htmlFor: 'scenario' }, 'Scenario', ...autoBadge('scenario')), picker),
    segmented('To', RECIPIENTS, 'recipient'),
    segmented('Via', CHANNELS, 'channel'),
    el('div', { className: 'fields' }, ...(sc ? sc.fields.filter(visible).map(fieldInput) : [])),
  );
}

function buildInput() {
  const sc = current(), fields = {}, toggles = {};
  for (const f of sc.fields.filter(visible)) {
    const value = state.fields[f.key];
    if (f.kind === 'toggle') toggles[f.key] = !!state.toggles[f.key];
    else if (f.kind === 'select' ? f.options.includes(value) : value) fields[f.key] = String(value);
  }
  return { scenario: sc.id, recipient: state.recipient, channel: state.channel, lang: state.lang, fields, toggles };
}

function schedule() { clearTimeout(timer); timer = setTimeout(run, 120); }

async function run() {
  if (!current()) return;
  const my = ++seq, input = buildInput();
  composeEl.querySelectorAll('.field').forEach((n) => { n.classList.remove('invalid'); n.querySelector('.ferr').textContent = ''; });
  try {
    const out = await invoke('compose', { input });
    if (my !== seq) return;
    last = out;
    const issues = await invoke('check', { text: out.text, ctx: { recipient: input.recipient, channel: input.channel, lang: input.lang } });
    if (my === seq) { renderOutput(out, issues); scheduleCheck(); } // recipient/channel change re-checks the editor
  } catch (err) {
    if (my !== seq) return;
    const msg = String(err), key = msg.match(/Missing required field: (\S+)/)?.[1];
    const field = key && composeEl.querySelector(`.field[data-key="${key}"]`);
    if (field) { field.classList.add('invalid'); field.querySelector('.ferr').textContent = 'Required'; }
    const label = key ? current().fields.find((f) => f.key === key)?.label : null;
    outputEl.replaceChildren(el('div', { className: 'error', role: 'alert', textContent: label ? `Please fill in: ${label}` : msg }));
  }
}

function renderOutput(out, issues) {
  const why = el('div', { className: 'why', textContent: 'Click any part of the message to see Why?' });
  const body = el('div', { className: 'msg', lang: 'ja' });
  let pos = 0;
  for (const s of out.segments) { // map each segment onto out.text so separators stay exact
    const at = out.text.indexOf(s.text, pos);
    if (at < 0) continue;
    if (at > pos) body.append(out.text.slice(pos, at));
    const span = el('span', { className: `segment slot-${s.slot}`, textContent: s.text, tabIndex: 0, title: s.gloss ?? s.rule_id });
    span.onclick = span.onkeydown = (e) => {
      if (e.type === 'keydown' && e.key !== 'Enter') return;
      body.querySelectorAll('.segment.sel').forEach((n) => n.classList.remove('sel'));
      span.classList.add('sel');
      why.replaceChildren(el('b', { textContent: 'Why? ' }), el('code', { textContent: s.rule_id }), el('div', { textContent: s.why }),
        ...(s.gloss ? [el('div', { className: 'gloss-line', textContent: `Meaning: ${s.gloss}` })] : []));
    };
    body.append(span);
    pos = at + s.text.length;
  }
  if (pos < out.text.length) body.append(out.text.slice(pos));

  const copy = el('button', { type: 'button', textContent: 'Copy' });
  copy.onclick = async () => {
    const txt = out.subject ? `件名：${out.subject}\n\n${out.text}` : out.text;
    try { await navigator.clipboard.writeText(txt); copy.textContent = 'Copied ✓'; }
    catch { copy.textContent = 'Copy failed'; }
    setTimeout(() => { copy.textContent = 'Copy'; }, 1200);
  };
  const toEditor = el('button', { type: 'button', textContent: 'To editor' });
  toEditor.onclick = () => setEditorText(out.text);

  // Per-segment English gloss (data/glosses.json); open state survives re-renders.
  const glossed = [...(out.subject_gloss ? [['件名', out.subject_gloss]] : []), ...out.segments.filter((s) => s.gloss).map((s) => [s.text.trim(), s.gloss])];
  const gloss = el('details', { className: 'gloss', open: glossOpen }, el('summary', { textContent: 'English meaning' }),
    el('ol', {}, ...glossed.map(([ja, en]) => el('li', {}, el('span', { className: 'ja', lang: 'ja', textContent: ja }), el('div', { textContent: en })))));
  gloss.ontoggle = () => { glossOpen = gloss.open; };
  const list = el('ul', { className: 'issues' }, ...issues.map((i) => el('li', { className: `sev-${i.severity}` },
    el('span', { className: 'sev', textContent: i.severity }),
    el('q', { lang: 'ja', textContent: out.text.slice(i.start, i.end) }), ` ${i.message}`,
    ...(i.suggestion ? [el('div', { className: 'sugg', textContent: `→ ${i.suggestion}` })] : []))));
  outputEl.replaceChildren(
    ...(out.subject ? [el('div', { className: 'subject', lang: 'ja' }, el('span', { className: 'lbl', textContent: '件名' }), out.subject)] : []),
    body, el('div', { className: 'actions' }, copy, toEditor), why, ...(glossed.length ? [gloss] : []),
    el('h3', { textContent: issues.length ? `Check: ${issues.length} issue(s)` : 'Check: no issues ✓' }), list,
  );
}

export async function initCompose() {
  try {
    state.scenarios = await invoke('list_scenarios');
    state.scenario = state.scenarios[0]?.id ?? null;
  } catch (err) {
    outputEl.replaceChildren(el('div', { className: 'error', textContent: `Engine unavailable: ${err}` }));
  }
  renderForm();
  schedule();
}

export const checkCtx = () => ({ recipient: state.recipient, channel: state.channel, lang: state.lang });
export const composePanel = () => ({ element: composeEl, init() {} });
export const outputPanel = () => ({ element: outputEl, init() {} });
