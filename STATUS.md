# STATUS — read before you start, update your row when you finish or get blocked
Contract version: v1 (team plan doc, section 2: https://claude.ai/artifact/FwSjUMmWjo91UqDKn1WNKp — spec copy in [docs/spec.md](docs/spec.md))

| Person | Task | State | Files | Blocked by | Updated |
|---|---|---|---|---|---|
| Celia | Engine + Tauri | Implemented; integration pending | `src/`, `Cargo.toml`, `build.rs`, `tauri.conf.json`, `tests/golden.rs`, `.github/workflows/release.yml` | Waiting for `data/`, `tests/golden/`, and the questionnaire UI | 2026-11-21 14:24 JST |
| Xin | JP data + analyzer | Day 2 done: (1) `client` recipient; (2) scenarios `extension`, `reschedule`, `thanks` (4 recipients × email/chat, chat ≤120 chars); (3) `analyze()` in `src/engine/analyze.rs` + `data/analyze_rules.json` (47 weighted regex rules → guesses `recipient`, `channel`, `scenario`, `sender_name`, `recipient_name`, `client_company`, `sender_company`, `event`, `task`, `meeting`, `orig_date`, `day`; one guess per field, evidence ≤30 chars). Touched `src/engine/mod.rs` only to swap the stub for `mod analyze`. | **Celia:** add `scenarios/extension.json`, `reschedule.json`, `thanks.json` to `SCENARIO_FILES` in `src/engine/compose.rs`. **Zine:** paste box → `analyze` → prefill form (guess.field = scenario/recipient/channel or a field key) | 2026-11-21 18:28 JST |
| Zine | Frontend | Day 2 pushed: (1) Editor inline check highlights + hover/apply suggestion; (2) Received-message box in Compose → `analyze` → prefills scenario/recipient/channel/fields, "auto" badges with evidence+rule_id on hover, badge clears on manual edit; unknown scenario guesses are skipped and listed. To-buttons now read recipients from data/common.json (Client shows). Browser-tested w/ stub; **needs in-app smoke test** | `frontend/` | Celia: new scenarios need SCENARIO_FILES to appear in picker | 2026-11-21 18:55 JST |
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
