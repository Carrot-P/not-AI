// Editor panel: textarea + an overlay that draws check issues as underlines at Issue.start..end.
// The overlay mirrors the textarea's text with transparent glyphs, so only the marks are visible.
import { checkCtx } from './compose.js';

const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);
const wrap = document.createElement('div');
const area = document.createElement('textarea');
const overlay = document.createElement('div');
const tip = document.createElement('div');
wrap.className = 'editor-wrap';
overlay.className = 'editor-overlay';
overlay.setAttribute('aria-hidden', 'true');
tip.className = 'issue-tip';
tip.hidden = true;
area.setAttribute('aria-label', 'Text editor');
area.spellcheck = false;
area.placeholder = 'Type or paste a message here (or press "To editor" in Output) to check it.';
wrap.append(area, overlay);
document.body.append(tip);

let seq = 0, timer = 0, hideTimer = 0;

function render(text, issues) {
  const marks = issues.filter((i) => i.end > i.start && i.end <= text.length).sort((a, b) => a.start - b.start);
  overlay.replaceChildren();
  let pos = 0;
  for (const i of marks) {
    if (i.start < pos) continue; // overlapping issue: keep the first
    overlay.append(text.slice(pos, i.start));
    const m = document.createElement('mark');
    m.className = `sev-${i.severity}`;
    m.textContent = text.slice(i.start, i.end);
    m.onmouseenter = () => showTip(m, i);
    m.onmouseleave = scheduleHide;
    m.onmousedown = (e) => { e.preventDefault(); area.focus(); area.setSelectionRange(i.start, i.end); };
    overlay.append(m);
    pos = i.end;
  }
  overlay.append(text.slice(pos) + '\n'); // trailing newline keeps the last line's height in sync
  overlay.scrollTop = area.scrollTop;
}

function showTip(mark, issue) {
  clearTimeout(hideTimer);
  tip.replaceChildren();
  const head = document.createElement('div');
  const sev = document.createElement('b');
  sev.className = 'sev';
  sev.textContent = issue.severity;
  head.append(sev, ' ', issue.message);
  tip.append(head);
  if (issue.suggestion) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.textContent = `→ ${issue.suggestion}`;
    btn.title = 'Replace with suggestion';
    btn.onclick = () => {
      area.setRangeText(issue.suggestion, issue.start, issue.end, 'end');
      tip.hidden = true;
      area.focus();
      area.dispatchEvent(new Event('input'));
    };
    tip.append(btn);
  }
  const r = mark.getBoundingClientRect();
  tip.hidden = false;
  tip.style.left = `${Math.min(r.left, innerWidth - tip.offsetWidth - 8)}px`;
  tip.style.top = `${r.bottom + 4}px`;
}
function scheduleHide() { clearTimeout(hideTimer); hideTimer = setTimeout(() => { tip.hidden = true; }, 250); }
tip.onmouseenter = () => clearTimeout(hideTimer);
tip.onmouseleave = scheduleHide;

async function runCheck() {
  const my = ++seq, text = area.value;
  render(text, []); // keep text aligned immediately; marks arrive after check
  if (!text.trim()) return;
  try {
    const issues = await invoke('check', { text, ctx: checkCtx() });
    if (my === seq) render(text, issues);
  } catch { /* check never blocks typing */ }
}
export function scheduleCheck() { clearTimeout(timer); timer = setTimeout(runCheck, 200); }

area.addEventListener('input', scheduleCheck);
area.addEventListener('scroll', () => { overlay.scrollTop = area.scrollTop; tip.hidden = true; });
export function setEditorText(text) { area.value = text; scheduleCheck(); }
export const editorPanel = () => ({ element: wrap, init() {}, focus: () => area.focus() });
