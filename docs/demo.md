# Demo script (90 s)

**Setup before going on stage:** build is installed, app is open, airplane mode is OFF, the stopwatch is
visible, and the sample email below is on the clipboard. A pre-recorded backup video is open in another window.

| Time | Do | Say |
|---|---|---|
| 0:00 | Turn **airplane mode ON** and show the Wi-Fi icon | "Everything you'll see runs offline. No model, no API." |
| 0:08 | Fill in: Lateness · Professor · Email · train delay · 15 min · データ構造 | "I'm 15 minutes late for class because of a train delay. I want to email my professor." |
| 0:20 | The formal email appears (subject, 佐藤先生, お世話になっております, 到着いたします) | "A full email with a subject line and the correct keigo, written in under a second." |
| 0:30 | Switch the recipient to **Coworker** and the channel to **Chat** | "Same situation, but it's for a coworker on Slack." |
| 0:37 | The short line appears (お疲れ様です…すみません！) | "It's short, casual and has no subject. The politeness level changes because the rules say it should." |
| 0:45 | Click 「到着いたします」 → **Why?** | "Every sentence shows the rule that wrote it. The recipient is a professor, which means an outside person of higher status, so my own action takes 謙譲語: 到着いたします." |
| 0:58 | Paste the client email (below) into **Received message** | "Now the Day 2 feature. I received this email from a client." |
| 1:05 | The form fills itself (client, email, reschedule, 佐藤商事, 11月25日14時) and the evidence badges show | "Regex rules read the email: 株式会社 means a client, and 候補日 means they want to reschedule. Each guess shows its evidence." |
| 1:12 | Type the new options in 候補日 (e.g. 11月28日) | |
| 1:15 | The reply appears (佐藤商事 / 佐藤様 / お世話になっております) | "This is the correct reply to a client." |
| 1:22 | Click Generate twice and show the identical output | "Same input, same output, every time. That's how you know it isn't AI." |
| 1:30 | End | |

**Sample received email** (copy it to the clipboard before the demo):

```
佐藤商事株式会社
営業部 部長 佐藤

山田様

いつもお世話になっております。
佐藤商事の佐藤です。
11月25日14時の打ち合わせですが、ご都合が悪くなった場合は候補日をお知らせください。
よろしくお願いいたします。
```

**Optional extras if time allows**
- **Checker:** type 「おっしゃられる」 in the Editor, and the 二重敬語 flag and its suggestion appear.
- **English instruction:** type "I'll be 15 minutes late to class because of a train delay, email to my professor".
  The form then fills in professor, email, lateness, train_delay and 15.

**⚠ Known blocker (found 14:15, Day 2): fix before rehearsal.** The analyzer puts the client's company
into `sender_company`, but the `reschedule` reply needs `client_company`, so compose returns
"Missing required field: client_company". Either the analyzer rule (Xin) or the frontend prefill (Zine)
needs to map it. Until then, type 佐藤商事 into the client company field by hand during the demo.

**If something breaks:** switch to the backup video and keep talking over it.
