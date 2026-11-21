use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComposeInput {
    pub scenario: String,
    pub recipient: String,
    pub channel: String,
    pub lang: String,
    pub fields: BTreeMap<String, String>,
    pub toggles: BTreeMap<String, bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Segment {
    pub slot: String,
    pub text: String,
    pub rule_id: String,
    pub why: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComposeOutput {
    pub subject: Option<String>,
    pub segments: Vec<Segment>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CheckContext {
    pub recipient: String,
    pub channel: String,
    pub lang: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Issue {
    pub rule_id: String,
    pub severity: String,
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Guess {
    pub field: String,
    pub value: String,
    pub evidence: String,
    pub rule_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnalyzeOutput {
    pub guesses: Vec<Guess>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldMeta {
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub show_if: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScenarioMeta {
    pub id: String,
    pub label_en: String,
    pub label_ja: String,
    pub langs: Vec<String>,
    pub fields: Vec<FieldMeta>,
}
