use super::{
    CheckContext, Issue,
    compose::{Conditions, derive_context, matches},
};
use regex::Regex;
use serde::{Deserialize, Deserializer, de::Error};
use std::sync::OnceLock;

#[derive(Deserialize)]
struct Rule {
    id: String,
    #[serde(default)]
    when: Conditions,
    severity: String,
    message: String,
    suggestion: Option<String>,
    #[serde(flatten)]
    kind: Kind,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Kind {
    Regex {
        #[serde(deserialize_with = "deserialize_regex")]
        pattern: Regex,
    },
    MaxChars {
        max: usize,
    },
}

fn deserialize_regex<'de, D: Deserializer<'de>>(d: D) -> Result<Regex, D::Error> {
    Regex::new(&String::deserialize(d)?).map_err(D::Error::custom)
}

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/engine-data/check_rules.json"
        )))
        .expect("invalid embedded check_rules.json")
    })
}

impl Rule {
    fn issue(&self, start: usize, end: usize) -> Issue {
        Issue {
            rule_id: self.id.clone(),
            severity: self.severity.clone(),
            start,
            end,
            message: self.message.clone(),
            suggestion: self.suggestion.clone(),
        }
    }
}

pub fn check(text: &str, ctx: &CheckContext) -> Vec<Issue> {
    check_rules(text, ctx, rules())
}

fn check_rules(text: &str, ctx: &CheckContext, rules: &[Rule]) -> Vec<Issue> {
    let ctx = derive_context(Default::default(), &ctx.recipient, &ctx.channel, &ctx.lang);
    let mut issues = Vec::new();
    for rule in rules.iter().filter(|rule| matches(&rule.when, &ctx)) {
        match &rule.kind {
            Kind::Regex { pattern } => {
                let (mut prev_byte, mut prev_end) = (0, 0);
                for hit in pattern.find_iter(text) {
                    let start = prev_end + text[prev_byte..hit.start()].encode_utf16().count();
                    let end = start + hit.as_str().encode_utf16().count();
                    issues.push(rule.issue(start, end));
                    (prev_byte, prev_end) = (hit.end(), end);
                }
            }
            Kind::MaxChars { max } => {
                if text.chars().count() > *max {
                    issues.push(rule.issue(0, text.encode_utf16().count()));
                }
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_utf16_spans_and_unicode_character_limit() {
        let rules: Vec<Rule> = serde_json::from_str(
            r#"[
            {"id":"regex", "kind":"regex", "pattern":"了解", "when":{"rank":["above"]},
             "severity":"error", "message":"Use a more polite phrase"},
            {"id":"length", "kind":"max_chars", "max":3, "when":{"channel":"chat"},
             "severity":"info", "message":"Too long"}
        ]"#,
        )
        .unwrap();
        let mut ctx = CheckContext {
            recipient: "professor".into(),
            channel: "chat".into(),
            lang: "ja".into(),
        };
        let issues = check_rules("🙂了解🍵了解", &ctx, &rules);
        let spans: Vec<_> = issues
            .iter()
            .map(|issue| (issue.rule_id.as_str(), issue.start, issue.end))
            .collect();
        assert_eq!(spans, [("regex", 2, 4), ("regex", 6, 8), ("length", 0, 8)]);
        ctx.recipient = "coworker".into();
        assert!(check_rules("🙂了解", &ctx, &rules).is_empty());
        ctx.channel = "email".into();
        assert!(check_rules("🙂了解🍵了解", &ctx, &rules).is_empty());
    }
}
