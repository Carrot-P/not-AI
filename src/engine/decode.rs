//! Decode mode: explains a received keigo message in plain terms. No ML: phrase rules come from
//! data/decode_rules.json, honorific verbs are a reverse lookup of data/keigo_verbs.json.
use super::{DecodeItem, DecodeOutput, DecodeSummary};
use regex::Regex;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize)]
struct RawRule {
    id: String,
    kind: String,
    pattern: String,
    plain: String,
}

struct Rule {
    id: String,
    kind: String,
    re: Regex,
    plain: String,
    why: String,
}

#[derive(Deserialize)]
struct VerbTable {
    verbs: BTreeMap<String, BTreeMap<String, String>>,
}

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        let raw: Vec<RawRule> = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/engine-data/decode_rules.json"
        )))
        .expect("invalid embedded decode_rules.json");
        let verbs: VerbTable = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/engine-data/keigo_verbs.json"
        )))
        .expect("invalid embedded keigo_verbs.json");
        build(raw, &verbs)
    })
}

fn build(raw: Vec<RawRule>, table: &VerbTable) -> Vec<Rule> {
    let mut out: Vec<Rule> = raw
        .into_iter()
        .map(|r| Rule {
            why: format!("phrase rule in decode_rules.json: /{}/", r.pattern),
            re: Regex::new(&r.pattern).unwrap_or_else(|e| panic!("{}: {e}", r.id)),
            id: r.id,
            kind: r.kind,
            plain: r.plain,
        })
        .collect();
    // Reverse lookup: honorific form -> dictionary verbs. Skip "#variant" rows (fixed phrases).
    let mut forms: BTreeMap<(String, &str), Vec<&str>> = BTreeMap::new();
    for (verb, f) in &table.verbs {
        if verb.contains('#') {
            continue;
        }
        for level in ["sonkei", "kenjo"] {
            if let Some(stem) = f.get(level).and_then(|t| t.strip_suffix("ます")) {
                if stem.chars().count() >= 2
                    && Some(stem) != f.get("teinei").and_then(|t| t.strip_suffix("ます"))
                {
                    forms
                        .entry((stem.to_string(), level))
                        .or_default()
                        .push(verb);
                }
            }
        }
    }
    for ((stem, level), verbs) in forms {
        let plain = verbs.join(" / ");
        let (who, label) = if level == "sonkei" {
            (
                "an action by you (the reader) or a third party, said respectfully",
                "sonkei",
            )
        } else {
            ("the sender's own action, said humbly", "kenjo")
        };
        out.push(Rule {
            id: format!("decode.verb.{label}.{}", verbs[0]),
            kind: "honorific".into(),
            re: Regex::new(&format!(
                "{}(?:ます|ました|ません|まして|ませ)",
                regex::escape(&stem)
            ))
            .unwrap(),
            plain: format!("= {plain} ({who})."),
            why: format!("keigo_verbs.json: {plain} → {label} {stem}ます"),
        });
    }
    out
}

pub fn decode(received: &str) -> DecodeOutput {
    decode_with(received, rules())
}

fn utf16(text: &str, byte: usize) -> usize {
    text[..byte].encode_utf16().count()
}

/// The sentence (split on 。！？ and newlines) that contains byte range start..end.
fn sentence(text: &str, start: usize, end: usize) -> String {
    let is_stop = |c: char| matches!(c, '。' | '！' | '？' | '\n');
    let s = text[..start]
        .rfind(is_stop)
        .map_or(0, |i| i + text[i..].chars().next().unwrap().len_utf8());
    let e = text[end..].find(is_stop).map_or(text.len(), |i| {
        end + i + text[end + i..].chars().next().unwrap().len_utf8()
    });
    text[s..e].trim().to_string()
}

fn decode_with(text: &str, rules: &[Rule]) -> DecodeOutput {
    // (byte_start, byte_end, rule, plain)
    let mut hits: Vec<(usize, usize, &Rule, String)> = Vec::new();
    for rule in rules {
        for caps in rule.re.captures_iter(text) {
            let m = caps.get(0).unwrap();
            if m.is_empty() {
                continue;
            }
            let mut plain = String::new();
            caps.expand(&rule.plain, &mut plain);
            hits.push((m.start(), m.end(), rule, plain));
        }
    }
    // Longest match wins at each position; overlapping shorter hits are dropped.
    hits.sort_by_key(|h| (h.0, std::cmp::Reverse(h.1 - h.0)));
    let mut items = Vec::new();
    let mut summary = Vec::new();
    let mut taken_until = 0;
    for (s, e, rule, plain) in hits {
        if s < taken_until {
            continue;
        }
        taken_until = e;
        if matches!(rule.kind.as_str(), "request" | "deadline" | "refusal") {
            summary.push(DecodeSummary {
                kind: rule.kind.clone(),
                sentence: sentence(text, s, e),
                plain: plain.clone(),
            });
        }
        items.push(DecodeItem {
            rule_id: rule.id.clone(),
            kind: rule.kind.clone(),
            start: utf16(text, s),
            end: utf16(text, e),
            text: text[s..e].to_string(),
            plain,
            why: rule.why.clone(),
        });
    }
    DecodeOutput { items, summary }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_requests_deadlines_verbs_and_refusals() {
        let text = "田中様\nお忙しいところ恐れ入りますが、資料をご覧になりましたら、11月25日（水）までにご返信いただけますと幸いです。\nご提案の件は、前向きに検討いたします。";
        let out = decode(text);
        let ids: Vec<&str> = out.items.iter().map(|i| i.rule_id.as_str()).collect();
        assert!(ids.contains(&"decode.cushion.busy"), "{ids:?}");
        assert!(ids.contains(&"decode.cushion.osoreirimasu"), "{ids:?}");
        assert!(
            ids.iter().any(|i| i.starts_with("decode.verb.sonkei.")),
            "{ids:?}"
        );
        assert!(ids.contains(&"decode.refusal.kentou"), "{ids:?}");
        let dl = out
            .items
            .iter()
            .find(|i| i.rule_id == "decode.deadline.date")
            .expect("deadline");
        assert_eq!(dl.plain, "Deadline: 11月25日（水）.");
        assert!(out.items.iter().any(|i| i.kind == "request"));
        assert!(
            out.summary
                .iter()
                .any(|s| s.kind == "request" && s.sentence.contains("ご返信"))
        );
        // Spans are UTF-16 offsets into the original text.
        let u: Vec<u16> = text.encode_utf16().collect();
        for i in &out.items {
            assert_eq!(String::from_utf16(&u[i.start..i.end]).unwrap(), i.text);
        }
    }

    #[test]
    fn does_not_mistake_moushiwake_for_a_verb() {
        let out = decode("大変申し訳ございません。山田と申します。");
        let hits: Vec<&str> = out.items.iter().map(|i| i.text.as_str()).collect();
        assert_eq!(hits, ["申します"]);
    }
}
