//! Rule-based reading of a received message: guesses who sent it, which channel it came on,
//! which reply scenario fits, and fills form fields. No ML: every guess is a regex hit in
//! data/analyze_rules.json, and the winning rule is returned as evidence.
use super::{AnalyzeOutput, Guess};
use regex::Regex;
use serde::{Deserialize, Deserializer, de::Error};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize)]
struct Rule {
    id: String,
    field: String,
    /// Literal value, or capture references like "$1" / "${name}".
    value: String,
    #[serde(deserialize_with = "deserialize_regex")]
    pattern: Regex,
    #[serde(default = "one")]
    weight: u32,
}

fn one() -> u32 {
    1
}

fn deserialize_regex<'de, D: Deserializer<'de>>(d: D) -> Result<Regex, D::Error> {
    Regex::new(&String::deserialize(d)?).map_err(D::Error::custom)
}

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/engine-data/analyze_rules.json"
        )))
        .expect("invalid embedded analyze_rules.json")
    })
}

/// Evidence is shown in the UI; keep it to a readable snippet.
fn shorten(text: &str) -> String {
    const MAX: usize = 30;
    if text.chars().count() <= MAX {
        text.into()
    } else {
        text.chars().take(MAX).chain(['…']).collect()
    }
}

pub fn analyze(received: &str) -> AnalyzeOutput {
    analyze_with(received, rules())
}

struct Candidate<'a> {
    score: u32,
    best_weight: u32,
    evidence: String,
    rule_id: &'a str,
}

/// Each rule votes once (its first hit) for one value of one field, with its weight.
/// Per field the value with the highest total wins; ties go to the value voted for first.
/// Fields come out in the order their first rule appears in the file, so output is deterministic.
fn analyze_with(received: &str, rules: &[Rule]) -> AnalyzeOutput {
    let mut field_order: Vec<&str> = Vec::new();
    let mut votes: BTreeMap<&str, Vec<(String, Candidate)>> = BTreeMap::new();
    for rule in rules {
        let Some(caps) = rule.pattern.captures(received) else {
            continue;
        };
        let mut value = String::new();
        caps.expand(&rule.value, &mut value);
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }
        let evidence = shorten(caps[0].trim_matches(|c: char| c.is_whitespace() || c == '。'));
        if !field_order.contains(&rule.field.as_str()) {
            field_order.push(&rule.field);
        }
        let list = votes.entry(&rule.field).or_default();
        match list.iter_mut().find(|(v, _)| *v == value) {
            Some((_, c)) => {
                c.score += rule.weight;
                if rule.weight > c.best_weight {
                    c.best_weight = rule.weight;
                    c.evidence = evidence;
                    c.rule_id = &rule.id;
                }
            }
            None => list.push((
                value,
                Candidate {
                    score: rule.weight,
                    best_weight: rule.weight,
                    evidence,
                    rule_id: &rule.id,
                },
            )),
        }
    }
    let guesses = field_order
        .into_iter()
        .filter_map(|field| {
            let list = votes.remove(field)?;
            let mut best: Option<(String, Candidate)> = None;
            for (value, c) in list {
                if best.as_ref().is_none_or(|(_, b)| c.score > b.score) {
                    best = Some((value, c));
                }
            }
            let (value, c) = best?;
            Some(Guess {
                field: field.to_string(),
                value,
                evidence: c.evidence,
                rule_id: c.rule_id.to_string(),
            })
        })
        .collect();
    AnalyzeOutput { guesses }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(out: &'a AnalyzeOutput, field: &str) -> Option<&'a str> {
        out.guesses
            .iter()
            .find(|g| g.field == field)
            .map(|g| g.value.as_str())
    }

    #[test]
    fn weighted_vote_capture_values_and_tie_order() {
        let rules: Vec<Rule> = serde_json::from_str(
            r#"[
            {"id":"a", "field":"recipient", "value":"boss", "pattern":"お疲れ様"},
            {"id":"b", "field":"recipient", "value":"client", "pattern":"株式会社", "weight":3},
            {"id":"c", "field":"recipient", "value":"boss", "pattern":"部長"},
            {"id":"d", "field":"recipient_name", "value":"$1", "pattern":"(\\p{Han}{1,4})部長"},
            {"id":"e", "field":"channel", "value":"chat", "pattern":"ない"}
        ]"#,
        )
        .unwrap();
        let out = analyze_with("お疲れ様です。佐藤部長より", &rules);
        assert_eq!(get(&out, "recipient"), Some("boss"));
        assert_eq!(get(&out, "recipient_name"), Some("佐藤"));
        assert_eq!(get(&out, "channel"), None);
        assert_eq!(out.guesses[0].rule_id, "a");
        let out = analyze_with("株式会社ABC 佐藤部長", &rules);
        assert_eq!(get(&out, "recipient"), Some("client"));
        assert_eq!(out.guesses[0].evidence, "株式会社");
    }

    #[test]
    fn real_rules_read_typical_messages() {
        let professor = "山田さん\n\nレポートの提出期限は11月28日（金）です。\n「データ構造」の授業で説明した通りです。\n\n情報科学科 教授\n佐藤 一郎";
        let out = analyze(professor);
        assert_eq!(get(&out, "recipient"), Some("professor"));
        assert_eq!(get(&out, "channel"), Some("email"));
        assert_eq!(get(&out, "scenario"), Some("extension"));
        assert_eq!(get(&out, "sender_name"), Some("山田"));
        assert_eq!(get(&out, "event"), Some("データ構造"));
        assert_eq!(get(&out, "recipient_name"), Some("佐藤"));
        // A title followed by ordinary text on the next line is not a name.
        let out = analyze("田中教授\n先日はご相談に乗っていただき、ありがとうございました。");
        assert_ne!(get(&out, "recipient_name"), Some("先日"));

        let client = "株式会社XYZ\n山田様\n\nいつもお世話になっております。\n株式会社ABCの佐藤です。\n\n次回のお打ち合わせについて、11月25日（火）14時でご都合はいかがでしょうか。\n\n株式会社ABC\n佐藤";
        let out = analyze(client);
        assert_eq!(get(&out, "recipient"), Some("client"));
        assert_eq!(get(&out, "scenario"), Some("reschedule"));
        assert_eq!(get(&out, "client_company"), Some("株式会社ABC"));
        assert_eq!(get(&out, "sender_company"), Some("株式会社XYZ"));
        assert_eq!(get(&out, "recipient_name"), Some("佐藤"));
        assert_eq!(get(&out, "orig_date"), Some("11月25日（火）14時"));

        let coworker = "おつかれ！今どこ？もう会議始まってるよー";
        let out = analyze(coworker);
        assert_eq!(get(&out, "recipient"), Some("coworker"));
        assert_eq!(get(&out, "channel"), Some("chat"));
        assert_eq!(get(&out, "scenario"), Some("lateness"));

        assert_eq!(
            serde_json::to_vec(&analyze(professor)).unwrap(),
            serde_json::to_vec(&analyze(professor)).unwrap()
        );
    }
}
