use not_ai::engine::{
    self, AnalyzeOutput, CheckContext, ComposeInput, ComposeOutput, DecodeOutput, Issue,
    ScenarioMeta,
};

#[tauri::command]
pub fn list_scenarios() -> Vec<ScenarioMeta> {
    engine::list_scenarios()
}

#[tauri::command]
pub fn compose(input: ComposeInput) -> Result<ComposeOutput, String> {
    engine::compose(&input)
}

#[tauri::command]
pub fn check(text: String, ctx: CheckContext) -> Vec<Issue> {
    engine::check(&text, &ctx)
}

#[tauri::command]
pub fn analyze(received: String) -> AnalyzeOutput {
    engine::analyze(&received)
}

#[tauri::command]
pub fn decode(received: String) -> DecodeOutput {
    engine::decode(&received)
}
