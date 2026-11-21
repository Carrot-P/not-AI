# not-AI — Spec (decided 2026-11-21 13:10)

Full team plan (contract, task cards, timeline): https://claude.ai/artifact/FwSjUMmWjo91UqDKn1WNKp

## Problem / user
- Writing ≈ 40% of work ChatGPT messages (OpenAI, NBER WP 34255, 2025). Non-AI tools (Hemingway, proselint, write-good, Vale) only CHECK; we GENERATE.
- User: international students/workers in Japan unsure of keigo. Mostly Japanese scenarios, a few general English ones (e.g. absence notice to professor). Email AND chat.

## Concept: composition engine (not fill-in-the-blank)
- Message = slots (subject, address, greeting, self_intro, core, apology, request, closing, signature).
- Per slot: first variant whose `when` matches context (recipient, channel, lang, rank, in_group, derived politeness formal/polite/casual, fields, toggles). Scenario variants first, then common.
- Placeholders: `{field}`, `{phrase}`, `{verb:辞書形}` (own action: kenjo/teinei/plain), `{verb_their:…}` (sonkei/teinei/plain).
- Every segment returns rule_id + why → "Why?" trace. Deterministic.
- Checker: regex/max_chars rules (二重敬語, 了解しました to superior, chat too long).

## Steps
- Step 1 (Day 1): questionnaire (dropdowns, switches, short fields).
- Step 2 (Day 2 wow): paste received message → analyze() rules guess recipient/in-group/scenario → form fills → reply. Stretch: EN/ZH description → form.

## MVP 17:00 Day 1
lateness + absence · professor/boss/coworker · email/chat · ~15 verbs · 3 check rules (list) · Why? trace.

## Day 2
analyze() wow · client recipient · meeting_request/extension/apology/follow_up · date/season rules · inline highlights · English scenarios · GitHub Release (tauri-action). Freeze 14:00 (PRs only after), submit 15:30, deadline 16:00.

## No-AI rule
No ML models, AI APIs, trained tokenizers (no kuromoji/MeCab, spaCy, LanguageTool). Proof: airplane-mode demo, Why? trace, identical output for identical input, no model files.

## Stack
Tauri 2 desktop app. Rust engine in `src/engine/` (no Tauri dep, `cargo test`), `[lib] name="not_ai"`. Frontend HTML/JS modules in `frontend/`, bundled by Vite, Dockview panels (Celia's package.json) + `stub.js` for browser dev. Data JSON in `data/` via `include_str!`. Crates: tauri, serde, serde_json, regex. `withGlobalTauri: true`.

## Interface (v1)
- `list_scenarios() -> ScenarioMeta[]`
- `compose(input: ComposeInput) -> Result<ComposeOutput, String>`; ComposeInput {scenario, recipient, channel, lang, fields: map<str,str>, toggles: map<str,bool>}; ComposeOutput {subject?, segments[{slot,text,rule_id,why}], text}
- `check(text, ctx{recipient,channel,lang}) -> Issue[{rule_id,severity,start,end (UTF-16),message,suggestion?}]`
- `analyze(received) -> {guesses[{field,value,evidence,rule_id}]}`
- Data: data/common.json, data/scenarios/<id>.json, data/keigo_verbs.json, data/check_rules.json, data/analyze_rules.json, tests/golden/*.json (must_contain / must_not_contain / expect_subject).

## Run
`npm ci && cargo run` (app; Cargo builds the frontend) · `npx vite frontend` → localhost:5173 (UI only) · `cargo test`

## Team
- Celia: Rust engine + Tauri shell + release workflow
- Xin: Japanese content data; Day 2 analyze rules + analyze.rs
- Zine: frontend
- Thomas: STATUS.md, golden tests, README/DevPost, slides, submission, Day 2 English scenarios
