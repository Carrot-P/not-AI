# STATUS — read before you start, update your row when you finish or get blocked

Contract version: v1 (see team plan doc, section 2)

| Person | Task | State | Files | Blocked by | Updated |
|---|---|---|---|---|---|
| Celia | Engine + Tauri | Implemented; integration pending | `src/`, `Cargo.toml`, `build.rs`, `tauri.conf.json`, `tests/golden.rs`, `.github/workflows/release.yml` | Waiting for `data/`, `tests/golden/`, and the questionnaire UI | 2026-11-21 14:24 JST |

- Verified the four Tauri commands in the Windows WebView. `analyze` returns empty guesses on Day 1. The professor/lateness/email smoke test confirms the rule trace and byte-identical output for repeat input.
- Until Xin's data lands, builds use examples from `tests/fixtures/engine/`; files in `data/` override matching examples automatically on rebuild. The golden runner is ready, but no team cases are present yet.
- The tag-triggered Windows `.exe` and macOS `.dmg` workflow requires official data and golden cases. No release has been built or published.

## Contract changes
- v1 initial
