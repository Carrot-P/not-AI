use not_ai::engine::{ComposeInput, compose};
use serde::Deserialize;
use std::{fs, io::ErrorKind, path::Path};

#[derive(Deserialize)]
struct Case {
    input: ComposeInput,
    must_contain: Vec<String>,
    must_not_contain: Vec<String>,
    expect_subject: bool,
}

#[test]
fn golden_cases() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!("No tests/golden/ directory yet; no golden cases run");
            return;
        }
        Err(err) => panic!("Could not read tests/golden/: {err}"),
    };
    let mut paths: Vec<_> = entries
        .map(|entry| entry.expect("read tests/golden/ entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "tests/golden/ has no JSON cases");
    for path in paths {
        let name = path.display();
        let json = fs::read_to_string(&path).unwrap_or_else(|err| panic!("{name}: {err}"));
        let case: Case = serde_json::from_str(&json).unwrap_or_else(|err| panic!("{name}: {err}"));
        let output = compose(&case.input).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(
            output.subject.is_some(),
            case.expect_subject,
            "{name}: subject presence"
        );
        let full_text = match &output.subject {
            Some(subject) => format!("{subject}\n{}", output.text),
            None => output.text.clone(),
        };
        for expected in case.must_contain {
            assert!(
                full_text.contains(&expected),
                "{name}: missing {expected:?} in {full_text:?}"
            );
        }
        for forbidden in case.must_not_contain {
            assert!(
                !full_text.contains(&forbidden),
                "{name}: found {forbidden:?} in {full_text:?}"
            );
        }
        assert!(
            output
                .segments
                .iter()
                .all(|s| !s.rule_id.is_empty() && !s.why.is_empty()),
            "{name}: missing rule trace"
        );
        let repeated = compose(&case.input).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(
            serde_json::to_vec(&output).unwrap(),
            serde_json::to_vec(&repeated).unwrap(),
            "{name}: output is not deterministic"
        );
    }
}

#[test]
fn professor_lateness_email_smoke_test() {
    let input: ComposeInput = serde_json::from_str(r#"{
        "scenario":"lateness", "recipient":"professor", "channel":"email", "lang":"ja",
        "fields":{"sender_name":"山田", "recipient_name":"佐藤", "reason":"train_delay", "minutes":"15", "event":"データ構造"},
        "toggles":{"ask_handouts":true}
    }"#).unwrap();
    let output = compose(&input).unwrap();
    assert!(output.subject.is_some());
    assert!(output.text.contains("佐藤先生"));
    assert!(output.text.contains("到着いたします"));
    let core = output
        .segments
        .iter()
        .find(|segment| segment.slot == "core")
        .unwrap();
    assert!(!core.rule_id.is_empty());
    assert!(core.why.contains("到着いたします (kenjo)"));
    assert_eq!(
        serde_json::to_vec(&output).unwrap(),
        serde_json::to_vec(&compose(&input).unwrap()).unwrap()
    );
}
