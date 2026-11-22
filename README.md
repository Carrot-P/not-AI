# not-AI — Japanese business writing without AI

TUJ Fall 2026 Hackathon · theme: **anti-AI** (do an LLM task without AI).

Writing is about 40% of work-related ChatGPT messages (OpenAI, *How People Use ChatGPT*, NBER WP 34255, 2025). Existing non-AI writing tools (Hemingway, proselint, Vale) only **check** text. not-AI also **writes** it. It is a desktop app that helps international students and workers in Japan produce correct keigo emails and chat messages, using rules only.

## Features
- **Compose:** pick a scenario (lateness, absence, apology, deadline extension, reschedule, meeting request, follow-up, thanks), a recipient (professor, boss, coworker, client) and a channel (email/chat), then fill a short form. A rule-based composition engine builds the message slot by slot (subject, address, greeting, core, request, closing, signature) and conjugates verbs into kenjōgo / sonkeigo / teineigo.
- **Why? trace:** every sentence shows the rule that produced it and why.
- **Checker:** regex rules flag mistakes such as double keigo, 了解しました to a superior, or chat messages that are too long, and offer one-click fixes.
- **Analyze:** paste a received message, or describe your situation in English/Chinese. Weighted regex rules guess the scenario, recipient and fields and fill in the form, with evidence for each guess.
- **Decode:** explains received keigo, including requests, deadlines and soft refusals. English glosses for every generated segment.
- Date and season rules (時候の挨拶).

## No-AI proof
No ML models, AI APIs or trained tokenizers (no MeCab/kuromoji, spaCy, LanguageTool). All behaviour comes from JSON rule files in `data/`. The output is deterministic, the app runs offline (airplane mode), and the repo contains no model files.

## Libraries
- Rust: tauri 2, serde, serde_json, regex (build: tauri-build)
- Frontend: Vite, Dockview

## Run
Requires Rust (stable) and Node.js, plus the Tauri 2 system prerequisites.

```sh
npm ci
cargo run        # builds the frontend automatically, then opens the app
cargo test       # engine tests
npx vite frontend   # UI only in a browser (stubbed engine), localhost:5173
```

## Layout
`src/engine/` rule engine (compose, keigo, check, analyze, decode; no Tauri dependency) · `data/` scenarios and rule JSON · `frontend/` HTML/JS UI.

## Team
Celia, Xin. Built with help from Claude. The product itself uses no AI.

## License
[GPL-3.0-only](LICENSE)
