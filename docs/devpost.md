# DevPost: not-AI

**Tagline:** Correct Japanese business emails, written by rules, not by AI.

## Inspiration
Writing makes up about 40% of work-related ChatGPT messages (OpenAI, "How People Use ChatGPT", NBER WP 34255, 2025).
In Japan, a lot of that writing is keigo, and getting the politeness wrong has real costs:
- In a 2026 survey of 1,293 Japanese business people (日本ビジネスメール協会), 73% said they feel anxious about their
  own emails. Writing one email took 6 min 19 s on average.
- 64% of the same respondents already use generative AI for email, yet frequent AI users showed no clear drop in
  measured writing time.
- Japan now hosts a record 408,069 international students (JASSO, May 2025) and 2.57 million foreign workers
  (MHLW, Oct 2025). Many of them write keigo as a second or third language.

The hackathon theme asked for a task people hand to LLMs that could be done without AI. Keigo is a rule system:
who you're writing to, whether they are inside or outside your group, and whose action a verb describes. So we
wrote the rules.

## What it does
You pick a scenario (late, absent, deadline extension, reschedule, meeting request, apology, follow-up or thanks),
a recipient (professor, boss, coworker or client) and a channel (email or chat). Then you fill in a few fields.
not-AI writes the message:
- **Email:** a subject line, an address, お世話になっております or お疲れ様です, the body and a signature.
- **Chat:** one short line.

You can click any sentence to see **Why?**, the rule that wrote it. For example: *recipient = professor (outside
group, higher status), so your own action takes 謙譲語: 到着いたします.*

There are two more tools:
- **Checker:** 11 rules flag errors in text you type or edit, such as 二重敬語 (おっしゃられる, お伺いさせていただく),
  了解しました or ご苦労様 to a superior, お疲れ様 to an outside contact, and chat messages that are too long.
- **Received-message analyzer:** paste an email you received, and regex rules guess the recipient type, scenario,
  dates and names. They fill in the form and show the evidence for each guess. It also accepts short instructions
  in English or Chinese.

## How we built it
- **Tauri 2** desktop app with a **Rust** engine that has no Tauri dependency, so it can be unit-tested with `cargo test`.
- **Composition engine:** a message is a sequence of slots (subject, address, greeting, self-intro, core, apology,
  request, closing, signature). For each slot, the engine picks the first variant whose `when` condition matches
  the context. The context covers recipient, channel, rank, in-group, the derived politeness level (formal, polite
  or casual), fields and toggles.
- **Keigo verb table:** `{verb:…}` placeholders conjugate *your own* actions into 謙譲語, teinei or plain form, and
  `{verb_their:…}` conjugates *their* actions into 尊敬語.
- **Data:** all language data is hand-written JSON (`data/`), using **serde** and **serde_json**. Checker and
  analyzer patterns use the **regex** crate. The UI is built with **Vite** and **Dockview** panels.
- **Golden tests:** each scenario × recipient × channel combination is tested for required phrases, forbidden
  phrases, subject presence and byte-identical output on repeated runs.

## Proof there's no AI
- No ML models, AI APIs or trained tokenizers (no MeCab, kuromoji or spaCy).
- It runs in airplane mode.
- The same input always gives the same output, and the tests assert this.
- Every sentence has a rule trace, and there are no model files in the repo.

## Challenges we ran into
- **Keigo direction:** the same verb becomes 伺う when it's your own action and いらっしゃる when it's theirs. The
  engine has to know whose action each slot describes.
- **Politeness has more than two settings:** 本日 vs 今日, ございます vs あります, and whether a chat message gets a
  greeting at all all depend on recipient × channel × in-group.
- **No morphological analyser:** the analyzer and checker had to work with weighted regex rules only, because
  tokenizers like MeCab are trained models.
- **Explaining keigo to judges who don't speak Japanese:** the Why? trace turned out to be the answer.

## Accomplishments we're proud of
- 8 scenarios × 4 recipients × 2 channels, all deterministic and all traceable.
- A received-email analyzer with 47 weighted rules, still with no AI.
- The rules are explainable enough that the app can teach keigo, not just produce it.

## What we learned
- A lot of "write it nicely" work is a bounded rule system.
- Rule-based output is correct by construction inside its coverage and silent outside it. We say so openly.

## What's next
- More scenarios, and English scenarios for general workplace use.
- A practice mode that hides the output and quizzes you on the Why? rule.
- A browser extension that runs the checker on Gmail and Slack drafts.

## Built with
rust · tauri · serde · serde_json · regex · javascript · vite · dockview

## Team
Celia (engine, Tauri, release) · Xin (Japanese data, analyzer) · Zine (frontend) · Thomas (tests, docs, pitch)

## Links
- Repo: https://github.com/Carrot-P/not-AI (GPL-3.0-only)
