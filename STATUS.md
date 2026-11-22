# STATUS — read before you start, update your row when you finish or get blocked
Contract version: v1 (team plan doc, section 2: https://claude.ai/artifact/FwSjUMmWjo91UqDKn1WNKp — spec copy in [docs/spec.md](docs/spec.md))

| Person | Task | State | Files | Blocked by | Updated |
|---|---|---|---|---|---|
| Celia | Engine + Tauri | Day 2 integrated: eight JP scenarios, client, date/season rules | `src/engine/compose.rs`, `data/`, `frontend/`, `tests/golden.rs`, `.github/workflows/release.yml` | Team golden cases and macOS release run pending | Day 2 integration |
| Xin | JP data + analyzer | Day 2 done: `client`, `extension`, `reschedule`, `thanks`, and `analyze()` with 47 weighted regex rules. All three scenarios are registered in the engine. | `data/`, `src/engine/analyze.rs`, `src/engine/mod.rs` | - | 2026-11-21 18:28 JST |
| Zine | Frontend | Day 2 pushed: Editor inline highlights with suggestions, received-message box with form prefill and evidence badges, recipients loaded from JSON | `frontend/` | - | 2026-11-21 18:55 JST |
| Thomas | Tests + pitch + submission | Golden cases + README in PR #2 (Xin-reviewed). PR #3: `docs/demo.md`, `docs/devpost.md`, English scenarios `lateness_en`/`absence_en` (registered in compose.rs, UI picks lang from scenario) + 4 EN golden cases. ⚠ Demo blocker: analyzer → reschedule reply fails `Missing required field: client_company` (see demo.md). Next: slides, DevPost page | `tests/golden/*.json`, `README.md`, `docs/`, `data/scenarios/*_en.json` | - | 2026-11-22 14:45 JST |

## Celia — verification
- Local meeting request, apology, and seasonal follow-up coexist with Xin’s client/address rules, deadline extension, rescheduling, thank-you scenarios and analyzer. Extension uses Xin's field names so it stays compatible with analyzer guesses.
- Rust tests exercise the six Day 2 scenarios across recipients and channels, date/season selection, client wording, and analyzer-to-scenario integration. A Windows desktop smoke test verified all eight scenarios in the picker, analyzer autofill into rescheduling, and editor suggestions. Team-owned `tests/golden/*.json` files are still absent.
- Windows debug NSIS packaging passed before integration. The tag-triggered workflow packages Windows `.exe` and macOS `.dmg` into a draft release once the team golden cases are present. No tag or release has been published.

## Rules
- Push to main with `git pull --rebase` first, until Day 2 14:00 freeze; after that, PRs reviewed by Celia.
- Only Celia edits Cargo.toml, src/main.rs, tauri.conf.json.
- No ML models or AI APIs in the product.

## Reassignment log
- 2026-11-21 13:20 initial split

## Contract changes
- v1 initial
