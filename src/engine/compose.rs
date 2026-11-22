use super::{ComposeInput, ComposeOutput, ScenarioMeta, Segment, keigo};
use regex::{Captures, Regex};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

const SCENARIO_FILES: &[&str] = &[
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/lateness.json"
    )),
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/absence.json"
    )),
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/meeting_request.json"
    )),
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/extension.json"
    )),
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/apology.json"
    )),
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/follow_up.json"
    )),
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/reschedule.json"
    )),
    include_str!(concat!(
        env!("OUT_DIR"),
        "/engine-data/scenarios/thanks.json"
    )),
];

pub(super) type Context = BTreeMap<String, String>;
pub(super) type Conditions = BTreeMap<String, Expected>;
type Slots = BTreeMap<String, Vec<Variant>>;

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum Expected {
    One(String),
    Any(Vec<String>),
}

#[derive(Deserialize)]
struct Common {
    recipients: BTreeMap<String, Recipient>,
    politeness: Vec<Politeness>,
    slots: Slots,
}

#[derive(Deserialize)]
struct Recipient {
    rank: String,
    in_group: bool,
}

#[derive(Deserialize)]
struct Politeness {
    #[serde(default)]
    when: Conditions,
    value: String,
}

#[derive(Deserialize)]
struct Scenario {
    #[serde(flatten)]
    meta: ScenarioMeta,
    slot_order: Vec<String>,
    #[serde(default)]
    phrases: BTreeMap<String, Phrase>,
    slots: Slots,
}

#[derive(Deserialize)]
struct Variant {
    id: String,
    #[serde(default)]
    when: Conditions,
    #[serde(default)]
    skip: bool,
    #[serde(flatten)]
    text: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Phrase {
    by: String,
    map: BTreeMap<String, PhraseText>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PhraseText {
    Plain(String),
    ByPoliteness { formal: String, casual: String },
}

fn common() -> &'static Common {
    static COMMON: OnceLock<Common> = OnceLock::new();
    COMMON.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/engine-data/common.json"
        )))
        .expect("invalid embedded common.json")
    })
}

fn scenarios() -> &'static [Scenario] {
    static SCENARIOS: OnceLock<Vec<Scenario>> = OnceLock::new();
    SCENARIOS.get_or_init(|| {
        SCENARIO_FILES
            .iter()
            .map(|json| serde_json::from_str(json).expect("invalid embedded scenario JSON"))
            .collect()
    })
}

pub(super) fn matches(conditions: &Conditions, ctx: &Context) -> bool {
    conditions.iter().all(|(key, expected)| {
        ctx.get(key).is_some_and(|actual| match expected {
            Expected::One(value) => actual == value,
            Expected::Any(values) => values.contains(actual),
        })
    })
}

pub(super) fn derive_context(
    mut ctx: Context,
    recipient: &str,
    channel: &str,
    lang: &str,
) -> Context {
    for (key, value) in [
        ("recipient", recipient),
        ("channel", channel),
        ("lang", lang),
    ] {
        ctx.insert(key.into(), value.into());
    }
    // Input fields cannot override derived values.
    for key in ["rank", "in_group", "politeness"] {
        ctx.remove(key);
    }
    if let Some(profile) = common().recipients.get(recipient) {
        ctx.insert("rank".into(), profile.rank.clone());
        ctx.insert("in_group".into(), profile.in_group.to_string());
    }
    if let Some(rule) = common()
        .politeness
        .iter()
        .find(|rule| matches(&rule.when, &ctx))
    {
        ctx.insert("politeness".into(), rule.value.clone());
    }
    ctx
}

pub fn list_scenarios() -> Vec<ScenarioMeta> {
    scenarios().iter().map(|s| s.meta.clone()).collect()
}

pub fn compose(input: &ComposeInput) -> Result<ComposeOutput, String> {
    let scenario = scenarios()
        .iter()
        .find(|s| s.meta.id == input.scenario)
        .ok_or_else(|| format!("Unknown scenario: {}", input.scenario))?;
    compose_scenario(input, scenario, &common().slots)
}

fn compose_scenario(
    input: &ComposeInput,
    scenario: &Scenario,
    shared_slots: &Slots,
) -> Result<ComposeOutput, String> {
    let mut values = input.fields.clone();
    values.extend(
        input
            .toggles
            .iter()
            .map(|(key, on)| (key.clone(), on.to_string())),
    );
    let ctx = derive_context(values, &input.recipient, &input.channel, &input.lang);
    for field in &scenario.meta.fields {
        let visible = field
            .show_if
            .iter()
            .all(|(key, value)| ctx.get(key) == Some(value));
        let filled = if field.kind == "toggle" {
            input.toggles.contains_key(&field.key)
        } else {
            input
                .fields
                .get(&field.key)
                .is_some_and(|value| !value.trim().is_empty())
        };
        if field.required && visible && !filled {
            return Err(format!("Missing required field: {}", field.key));
        }
    }

    let email = input.channel == "email";
    let mut subject = None;
    let mut subject_gloss = None;
    let mut segments = Vec::new();
    let mut missed = Vec::new();
    let politeness = ctx.get("politeness").map_or("casual", String::as_str);

    for slot in &scenario.slot_order {
        if slot == "subject" && !email {
            continue;
        }
        let picked = scenario
            .slots
            .get(slot)
            .into_iter()
            .flatten()
            .chain(shared_slots.get(slot).into_iter().flatten())
            .find(|variant| {
                (variant.skip || variant.text.contains_key(&input.lang))
                    && matches(&variant.when, &ctx)
            });
        let Some(variant) = picked else {
            missed.push(format!("no variant for slot {slot}"));
            continue;
        };
        if variant.skip {
            continue;
        }
        let mut why = vec![if variant.when.is_empty() {
            "unconditional match".into()
        } else {
            variant
                .when
                .keys()
                .map(|key| format!("{key}={}", ctx[key]))
                .collect::<Vec<_>>()
                .join(", ")
        }];
        let text = expand(
            &variant.text[&input.lang],
            scenario,
            input,
            politeness,
            2,
            &mut why,
        );
        let g = gloss(&variant.id, scenario, input);
        if slot == "subject" {
            subject = Some(text);
            subject_gloss = g;
        } else {
            why.append(&mut missed);
            segments.push(Segment {
                slot: slot.clone(),
                text,
                rule_id: variant.id.clone(),
                why: why.join("; "),
                gloss: g,
            });
        }
    }

    let text = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(if email { "\n" } else { "" });
    Ok(ComposeOutput {
        subject,
        subject_gloss,
        segments,
        text,
    })
}

#[derive(Deserialize)]
struct Glosses {
    phrases: BTreeMap<String, BTreeMap<String, String>>,
    variants: BTreeMap<String, String>,
}

/// English gloss for a Japanese variant: `{phrase}` -> phrases[phrase][selected value], `{field}` -> literal.
fn gloss(id: &str, scenario: &Scenario, input: &ComposeInput) -> Option<String> {
    static GLOSSES: OnceLock<Glosses> = OnceLock::new();
    let g = GLOSSES.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/engine-data/glosses.json"
        )))
        .expect("invalid embedded glosses.json")
    });
    if input.lang != "ja" {
        return None;
    }
    let template = g
        .variants
        .get(&format!("{}/{id}", scenario.meta.id))
        .or_else(|| g.variants.get(id))?;
    static PLACEHOLDER: OnceLock<Regex> = OnceLock::new();
    let re = PLACEHOLDER.get_or_init(|| Regex::new(r"\{([^{}]+)\}").unwrap());
    let fill = |t: &str| {
        re.replace_all(t, |cap: &Captures<'_>| {
            let key = &cap[1];
            scenario
                .phrases
                .get(key)
                .and_then(|p| input.fields.get(&p.by))
                .and_then(|value| g.phrases.get(key)?.get(value))
                .cloned()
                .unwrap_or_else(|| input.fields.get(key).cloned().unwrap_or_default())
        })
        .into_owned()
    };
    let text = fill(&fill(template)); // second pass: phrases may contain {field}
    let mut chars = text.chars();
    let first = chars.next()?;
    Some(first.to_uppercase().chain(chars).collect())
}

fn expand(
    template: &str,
    scenario: &Scenario,
    input: &ComposeInput,
    politeness: &str,
    passes: u8,
    why: &mut Vec<String>,
) -> String {
    if passes == 0 {
        return template.into();
    }
    static PLACEHOLDER: OnceLock<Regex> = OnceLock::new();
    PLACEHOLDER
        .get_or_init(|| Regex::new(r"\{([^{}]+)\}").expect("valid placeholder regex"))
        .replace_all(template, |cap: &Captures<'_>| {
            let key = &cap[1];
            if let Some(verb) = key.strip_prefix("verb:") {
                keigo::render(verb, politeness, false, why)
            } else if let Some(verb) = key.strip_prefix("verb_their:") {
                keigo::render(verb, politeness, true, why)
            } else if let Some(phrase) = scenario.phrases.get(key) {
                let Some(text) = input
                    .fields
                    .get(&phrase.by)
                    .and_then(|value| phrase.map.get(value))
                else {
                    return String::new();
                };
                let text = match text {
                    PhraseText::Plain(text) => text,
                    PhraseText::ByPoliteness { formal, casual } => {
                        if politeness == "casual" {
                            casual
                        } else {
                            formal
                        }
                    }
                };
                // Only rule-authored phrases are expanded again; user input stays literal.
                expand(text, scenario, input, politeness, passes - 1, why)
            } else {
                input.fields.get(key).cloned().unwrap_or_default()
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_first_usable_variant_and_traces_missing_slots() {
        let scenario: Scenario = serde_json::from_str(r#"{
          "id":"custom", "label_en":"Custom", "label_ja":"例", "langs":["ja"],
          "fields":[{"key":"reason_other", "label":"Other reason", "kind":"text", "required":true,
                     "show_if":{"reason":"other"}}],
          "slot_order":["subject","missing","greeting","skip","core"],
          "phrases":{"reason_phrase":{"by":"reason","map":{"other":"{reason_other}"}}},
          "slots":{
            "subject":[{"id":"subject", "ja":"件名"}],
            "greeting":[{"id":"other.lang", "en":"Hi"}],
            "skip":[{"id":"skip", "skip":true}],
            "core":[{"id":"want", "when":{"channel":["email","chat"],"ask":"true"}, "ja":"{reason_phrase}"},
                    {"id":"fallback", "ja":"fallback"}]
          }
        }"#).unwrap();
        let shared: Slots = serde_json::from_str(
            r#"{
          "greeting":[{"id":"common.hello", "ja":"こんにちは"}],
          "skip":[{"id":"must.not.render", "ja":"skip did not work"}],
          "core":[{"id":"must.not.win", "ja":"common"}]
        }"#,
        )
        .unwrap();
        let mut input = ComposeInput {
            scenario: "custom".into(),
            recipient: "coworker".into(),
            channel: "chat".into(),
            lang: "ja".into(),
            fields: BTreeMap::from([("reason".into(), "other".into())]),
            toggles: BTreeMap::from([("ask".into(), true)]),
        };
        assert_eq!(
            compose_scenario(&input, &scenario, &shared).unwrap_err(),
            "Missing required field: reason_other"
        );
        input
            .fields
            .insert("reason_other".into(), "理由{literal}".into());
        let output = compose_scenario(&input, &scenario, &shared).unwrap();
        assert!(output.subject.is_none());
        assert_eq!(output.text, "こんにちは理由{literal}");
        assert_eq!(output.segments[0].rule_id, "common.hello");
        assert!(
            output.segments[0]
                .why
                .contains("no variant for slot missing")
        );
        assert_eq!(output.segments[1].rule_id, "want");
        assert_eq!(output.segments[1].why, "ask=true, channel=chat");
        input.fields.insert("reason".into(), "illness".into());
        input.fields.remove("reason_other");
        assert!(compose_scenario(&input, &scenario, &shared).is_ok());
    }
}
