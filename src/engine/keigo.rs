use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize)]
struct VerbTable {
    verbs: BTreeMap<String, BTreeMap<String, String>>,
}

pub(super) fn render(verb: &str, politeness: &str, their: bool, why: &mut Vec<String>) -> String {
    static TABLE: OnceLock<VerbTable> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/engine-data/keigo_verbs.json"
        )))
        .expect("invalid embedded keigo_verbs.json")
    });
    select_form(table, verb, politeness, their, why)
}

fn select_form(
    table: &VerbTable,
    verb: &str,
    politeness: &str,
    their: bool,
    why: &mut Vec<String>,
) -> String {
    let requested = match (politeness, their) {
        ("formal", false) => "kenjo",
        ("formal", true) => "sonkei",
        ("polite", _) => "teinei",
        _ => "plain",
    };
    let forms = table.verbs.get(verb);
    if let Some(text) = forms.and_then(|f| f.get(requested)) {
        why.push(format!("{verb}→{text} ({requested})"));
        text.clone()
    } else if let Some(text) = forms.and_then(|f| f.get("teinei")) {
        why.push(format!("{verb}→{text} (teinei, fallback from {requested})"));
        text.clone()
    } else {
        why.push(format!(
            "no {requested} or teinei form for {verb}; used dictionary form"
        ));
        verb.split('#').next().unwrap_or(verb).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chooses_forms_by_action_owner_and_traces_fallback() {
        let table: VerbTable = serde_json::from_str(r#"{"verbs":{
            "行く":{"plain":"行く","teinei":"行きます","kenjo":"伺います","sonkei":"いらっしゃいます"},
            "行く#te":{"teinei":"行きまして"}
        }}"#).unwrap();
        let mut trace = Vec::new();
        for (politeness, their, expected) in [
            ("formal", false, "伺います"),
            ("formal", true, "いらっしゃいます"),
            ("polite", false, "行きます"),
            ("polite", true, "行きます"),
            ("casual", false, "行く"),
            ("casual", true, "行く"),
        ] {
            assert_eq!(
                select_form(&table, "行く", politeness, their, &mut trace),
                expected
            );
        }
        assert_eq!(
            select_form(&table, "行く#te", "formal", false, &mut trace),
            "行きまして"
        );
        assert!(
            trace
                .last()
                .unwrap()
                .contains("teinei, fallback from kenjo")
        );
    }
}
