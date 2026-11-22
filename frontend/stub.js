// Browser-only fallback: use real form metadata with canned output.
import common from '../data/common.json';

if (!window.__TAURI__) {
  const files = import.meta.glob('../data/scenarios/*.json', { eager: true, import: 'default' });
  const scenarios = ['lateness', 'absence', 'meeting_request', 'extension', 'apology', 'follow_up', 'reschedule', 'thanks']
    .map(id => files[`../data/scenarios/${id}.json`])
    .map(({ id, label_en, label_ja, langs, fields }) => ({ id, label_en, label_ja, langs, fields: fields.map(f => ({ options: [], ...f })) }));
  const honor = { professor: '先生', boss: 'さん', coworker: 'さん', client: '様' };
  window.__TAURI__ = { core: { invoke: async (cmd, args) => {
    if (cmd === 'list_scenarios') return scenarios;
    if (cmd === 'check') return args.text.includes('すみません')
      ? [{ rule_id: 'check.casual_apology', severity: 'warn', start: args.text.indexOf('すみません'), end: args.text.indexOf('すみません') + 5, message: '(stub) casual apology', suggestion: '申し訳ございません' }] : [];
    if (cmd === 'decode') { // canned: highlight one known request if present
      const t = args.received, k = 'いただけますと幸いです', at = t.indexOf(k);
      return at < 0 ? { items: [], summary: [] } : { items: [{ rule_id: 'stub.request', kind: 'request', start: at, end: at + k.length, text: k, plain: '(stub) Please do this.', why: 'stub' }],
        summary: [{ kind: 'request', sentence: t, plain: '(stub) Please do this.' }] };
    }
    if (cmd === 'analyze') return { guesses: [ // canned: pretend the message is from a coworker on chat
      { field: 'recipient', value: 'coworker', evidence: 'おつかれ', rule_id: 'stub.recipient' },
      { field: 'channel', value: 'chat', evidence: '！', rule_id: 'stub.channel' },
      { field: 'scenario', value: 'lateness', evidence: '今どこ', rule_id: 'stub.scenario' },
      { field: 'recipient_name', value: '佐藤', evidence: '佐藤', rule_id: 'stub.name' }] };
    const { input } = args, sc = scenarios.find((s) => s.id === input.scenario), email = input.channel === 'email';
    const profile = common.recipients[input.recipient];
    const ctx = { ...input.fields, ...input, rank: profile.rank, in_group: String(profile.in_group) };
    for (const f of sc.fields) if (f.required && Object.entries(f.show_if ?? {}).every(([k, v]) => ctx[k] === v) && !input.fields[f.key]?.trim()) throw `Missing required field: ${f.key}`;
    const formal = input.recipient !== 'coworker';
    const seg = (slot, text, why) => ({ slot, text, rule_id: `${slot}.${formal ? 'formal' : 'casual'}`, why: `(stub) ${why}`, gloss: `(stub) English meaning of the ${slot}` });
    const segments = [
      email && seg('address', `${input.fields.recipient_name}${honor[input.recipient]}`, `recipient=${input.recipient}`),
      seg('greeting', formal ? 'お疲れ様です。' : 'お疲れ様！', `politeness=${formal ? 'formal' : 'casual'}`),
      seg('core', input.scenario === 'lateness' ? `${input.fields.minutes}分ほど遅れます。` : `（${sc.label_ja}のサンプル）`, `scenario=${input.scenario}`),
      seg('apology', formal ? '大変申し訳ございません。' : 'すみません！', `politeness=${formal ? 'formal' : 'casual'}`),
    ].filter(Boolean);
    return { subject: email ? `${sc.label_ja}` : null, segments, text: segments.map((s) => s.text).join(email ? '\n' : '') };
  } } };
}
