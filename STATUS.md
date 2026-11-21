# STATUS — read before you start, update your row when you finish or get blocked
Contract version: v1 (team plan doc, section 2: https://claude.ai/artifact/FwSjUMmWjo91UqDKn1WNKp — spec copy in docs/spec.md)

| Person | Task | State | Files | Blocked by | Updated |
|---|---|---|---|---|---|
| Celia | Engine + Tauri | doing | src/, Cargo.toml, tauri.conf.json, tests/golden.rs | - | 13:20 |
| Xin | JP data (+ Day 2 analyzer) | doing | data/ (+ src/engine/analyze.rs Day 2) | rustup install | 13:20 |
| Zine | Frontend | doing | frontend/ | - | 13:20 |
| Thomas | Tests + pitch + submission | doing | tests/golden/*.json, README.md, docs/ | - | 13:20 |

## Rules
- Push to main with `git pull --rebase` first, until Day 2 14:00 freeze; after that, PRs reviewed by Celia.
- Only Celia edits Cargo.toml, src/main.rs, tauri.conf.json.
- No ML models or AI APIs in the product.

## Reassignment log
- 2026-11-21 13:20 initial split

## Contract changes
- v1 initial
