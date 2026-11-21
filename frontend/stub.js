// Browser-only fallback: used when not running inside Tauri (npx vite frontend).
if (!window.__TAURI__) {
  const show = { channel: 'email' };
  const scenario = (id, label_en, label_ja, extra = []) => ({ id, label_en, label_ja, langs: ['ja'], fields: [
    { key: 'sender_name', label: 'Your family name', kind: 'text', options: [], required: true, show_if: show },
    { key: 'recipient_name', label: 'Their family name', kind: 'text', options: [], required: true, show_if: show },
    { key: 'reason', label: 'Reason', kind: 'select', options: ['train_delay', 'illness', 'oversleep', 'other'], required: true, show_if: {} },
    ...extra] });
  const scenarios = [
    scenario('lateness', 'Will be late', '遅刻の連絡', [
      { key: 'minutes', label: 'Minutes late', kind: 'number', options: [], required: true, show_if: {} },
      { key: 'ask_handouts', label: 'Ask for handouts', kind: 'toggle', options: [], required: false, show_if: { recipient: 'professor' } },
      { key: 'reachable', label: 'Reachable if urgent', kind: 'toggle', options: [], required: false, show_if: { in_group: 'true' } }]),
    scenario('absence', 'Will be absent', '欠席の連絡'),
  ];
  const honor = { professor: '先生', boss: '様', coworker: 'さん' };
  window.__TAURI__ = { core: { invoke: async (cmd, args) => {
    if (cmd === 'list_scenarios') return scenarios;
    if (cmd === 'check') return args.text.includes('すみません')
      ? [{ rule_id: 'check.casual_apology', severity: 'warn', start: args.text.indexOf('すみません'), end: args.text.indexOf('すみません') + 5, message: '(stub) casual apology', suggestion: '申し訳ございません' }] : [];
    if (cmd === 'analyze') return { guesses: [] };
    const { input } = args, sc = scenarios.find((s) => s.id === input.scenario), email = input.channel === 'email';
    for (const f of sc.fields) if (f.required && Object.entries(f.show_if).every(([k, v]) => input[k] === v) && !input.fields[f.key]) throw `Missing required field: ${f.key}`;
    const formal = input.recipient !== 'coworker';
    const seg = (slot, text, why) => ({ slot, text, rule_id: `${slot}.${formal ? 'formal' : 'casual'}`, why: `(stub) ${why}` });
    const segments = [
      email && seg('address', `${input.fields.recipient_name}${honor[input.recipient]}`, `recipient=${input.recipient}`),
      seg('greeting', formal ? 'お疲れ様です。' : 'お疲れ様！', `politeness=${formal ? 'formal' : 'casual'}`),
      seg('core', input.scenario === 'lateness' ? `${input.fields.minutes}分ほど遅れます。` : '本日は欠席します。', `scenario=${input.scenario}`),
      seg('apology', formal ? '大変申し訳ございません。' : 'すみません！', `politeness=${formal ? 'formal' : 'casual'}`),
    ].filter(Boolean);
    return { subject: email ? `${sc.label_ja}` : null, segments, text: segments.map((s) => s.text).join(email ? '\n' : '') };
  } } };
}
