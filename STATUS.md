# STATUS — read before you start, update your row when you finish or get blocked
Contract version: v1 (team plan doc, section 2: https://claude.ai/artifact/FwSjUMmWjo91UqDKn1WNKp — spec copy in [docs/spec.md](docs/spec.md))

| Person | Task | State | Files | Blocked by | Updated |
|---|---|---|---|---|---|
| Celia | Engine + Tauri | Implemented; integration pending | `src/`, `Cargo.toml`, `build.rs`, `tauri.conf.json`, `tests/golden.rs`, `.github/workflows/release.yml` | Waiting for `data/`, `tests/golden/`, and the questionnaire UI | 2026-11-21 14:24 JST |
| Xin | JP data + analyzer | Day 2 done: (1) `client` recipient; (2) scenarios `extension`, `reschedule`, `thanks` (4 recipients × email/chat, chat ≤120 chars); (3) `analyze()` in `src/engine/analyze.rs` + `data/analyze_rules.json` (47 weighted regex rules → guesses `recipient`, `channel`, `scenario`, `sender_name`, `recipient_name`, `client_company`, `sender_company`, `event`, `task`, `meeting`, `orig_date`, `day`; one guess per field, evidence ≤30 chars). Touched `src/engine/mod.rs` only to swap the stub for `mod analyze`. | **Celia:** add `scenarios/extension.json`, `reschedule.json`, `thanks.json` to `SCENARIO_FILES` in `src/engine/compose.rs`. **Zine:** paste box → `analyze` → prefill form (guess.field = scenario/recipient/channel or a field key) | 2026-11-21 18:28 JST |
| Zine | Frontend | Day 1 UI done. Day 2 #1 pushed: Editor highlights check issues inline (wavy underline by severity), hover shows message + click-to-apply suggestion, re-checks on typing and on recipient/channel change. Next: Received-message box → analyze (needs Xin's analyzer). Browser-tested w/ stub; please smoke-test in app | `frontend/` (compose.js, editor.js, stub.js, style.css, main.js) | - | 2026-11-21 18:05 JST |
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
