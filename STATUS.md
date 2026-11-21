# STATUS — read before you start, update your row when you finish or get blocked
Contract version: v1 (team plan doc, section 2: https://claude.ai/artifact/FwSjUMmWjo91UqDKn1WNKp — spec copy in [docs/spec.md](docs/spec.md))

| Person | Task | State | Files | Blocked by | Updated |
|---|---|---|---|---|---|
| Celia | Engine + Tauri | Implemented; integration pending | `src/`, `Cargo.toml`, `build.rs`, `tauri.conf.json`, `tests/golden.rs`, `.github/workflows/release.yml` | Waiting for `data/`, `tests/golden/`, and the questionnaire UI | 2026-11-21 14:24 JST |
| Xin | JP data (+ Day 2 analyzer) | Day 1 data/ done: lateness+absence × professor/boss/coworker × email/chat, 19 verbs, 7 check rules; `cargo test` green. Placeholders used: {verb:…}, {reason_cause}, {day_phrase}, {event}, {minutes}, {sender_name}, {recipient_name}, {reason_other}. New fields: `event` (professor only), `day`, toggle `reachable` (show_if in_group=true) | `data/` (+ `src/engine/analyze.rs` Day 2) | - | 2026-11-21 15:31 JST |
| Zine | Frontend | Day 1 pushed: Compose + Output panels (scenario picker, recipient/channel, FieldMeta form w/ show_if, live compose, Why? trace, Copy, check list, inline required errors). Tested in browser w/ stub; please smoke-test in app | `frontend/` (compose.js, stub.js, style.css, main.js) | - | 2026-11-21 15:47 JST |
| Thomas | Tests + pitch + submission | doing | `tests/golden/*.json`, `README.md`, `docs/` | - | 13:20 |

## Celia — verification
- Verified the four Tauri commands in the Windows WebView. `analyze` returns empty guesses on Day 1. The professor/lateness/email smoke test confirms the rule trace and byte-identical output for repeat input.
- Until Xin's data lands, builds use examples from `tests/fixtures/engine/`; files in `data/` override matching examples automatically on rebuild. The golden runner is ready, but no team cases are present yet.
- The tag-triggered Windows `.exe` and macOS `.dmg` workflow requires official data and golden cases. No release has been built or published.

## Rules
- Push to main with `git pull --rebase` first, until Day 2 14:00 freeze; after that, PRs reviewed by Celia.
- Only Celia edits Cargo.toml, src/main.rs, tauri.conf.json.
- No ML models or AI APIs in the product.

## Reassignment log
- 2026-11-21 13:20 initial split

## Contract changes
- v1 initial
