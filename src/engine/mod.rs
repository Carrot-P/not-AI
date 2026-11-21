mod check;
mod compose;
mod keigo;
pub mod types;

pub use check::check;
pub use compose::{compose, list_scenarios};
pub use types::*;

pub fn analyze(_received: &str) -> AnalyzeOutput {
    AnalyzeOutput { guesses: vec![] }
}
