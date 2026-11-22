use not_ai::engine::{CheckContext, ComposeInput, analyze, check, compose, list_scenarios};
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

fn day_two_input(scenario: &str, recipient: &str, channel: &str) -> ComposeInput {
    ComposeInput {
        scenario: scenario.into(),
        recipient: recipient.into(),
        channel: channel.into(),
        lang: "ja".into(),
        fields: [
            ("sender_name", "山田"),
            ("recipient_name", "佐藤"),
            ("event", "情報科学"),
            ("topic", "仕様書"),
            ("meeting_day", "specific_date"),
            ("meeting_date", "11月25日"),
            ("meeting_time", "14時"),
            ("client_company", "佐藤商事"),
            ("sender_company", "山田企画"),
            ("task", "報告書"),
            ("new_date", "11月27日"),
            ("meeting", "定例会議"),
            ("orig_date", "11月25日"),
            ("new_options", "11月28日"),
            ("occasion", "advice"),
            (
                "reason",
                if scenario == "lateness" {
                    "train_delay"
                } else {
                    "illness"
                },
            ),
            ("day", "today"),
            ("minutes", "15"),
            ("incident", "資料の送付が遅れたこと"),
            ("next_step", "資料を送り直す"),
            ("previous_contact_date", "11月21日"),
        ]
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect(),
        toggles: Default::default(),
    }
}

#[test]
fn day_two_scenarios_cover_all_recipients_and_channels() {
    let scenarios = list_scenarios();
    for (scenario, expected) in [
        ("meeting_request", "11月25日の14時"),
        ("extension", "11月27日まで"),
        ("apology", "資料を送り直す"),
        ("follow_up", "11月21日に"),
        ("reschedule", "11月28日"),
        ("thanks", "ありがとうございました"),
    ] {
        assert!(
            scenarios
                .iter()
                .any(|s| s.id == scenario && s.langs.iter().any(|lang| lang == "ja"))
        );
        for recipient in ["professor", "boss", "coworker", "client"] {
            for channel in ["email", "chat"] {
                let input = day_two_input(scenario, recipient, channel);
                let label = format!("{scenario}/{recipient}/{channel}");
                let out = compose(&input).unwrap_or_else(|err| panic!("{label}: {err}"));
                assert_eq!(out.subject.is_some(), channel == "email", "{label}");
                let core = out
                    .segments
                    .iter()
                    .find(|seg| seg.slot == "core")
                    .expect(&label);
                assert!(out.text.contains(expected), "{label}: {}", out.text);
                assert!(!core.why.is_empty(), "{label}");
                if recipient == "client" && channel == "email" {
                    assert!(out.text.starts_with("佐藤商事\n佐藤様"), "{label}");
                    assert!(out.text.contains("お世話になっております"), "{label}");
                }
            }
        }
    }
}

#[test]
fn explicit_date_and_season_drive_the_rules() {
    let mut meeting = day_two_input("meeting_request", "client", "email");
    meeting.fields.remove("meeting_date");
    assert_eq!(
        compose(&meeting).unwrap_err(),
        "Missing required field: meeting_date"
    );
    meeting.fields.insert("meeting_day".into(), "today".into());
    assert!(compose(&meeting).unwrap().text.contains("本日の14時"));
    meeting.recipient = "coworker".into();
    meeting.channel = "chat".into();
    assert!(compose(&meeting).unwrap().text.contains("今日の14時"));

    let mut follow_up = day_two_input("follow_up", "client", "email");
    let has_season = |input: &ComposeInput| {
        compose(input)
            .unwrap()
            .segments
            .into_iter()
            .any(|s| s.slot == "seasonal_greeting")
    };
    assert!(!has_season(&follow_up));
    for season in ["spring", "summer", "autumn", "winter"] {
        follow_up.fields.insert("season".into(), season.into());
        let out = compose(&follow_up).unwrap();
        let seg = out
            .segments
            .iter()
            .find(|segment| segment.slot == "seasonal_greeting")
            .unwrap();
        assert_eq!(seg.rule_id, format!("greeting.season.{season}"));
        assert!(seg.why.contains(&format!("season={season}")));
        assert_eq!(
            serde_json::to_vec(&out).unwrap(),
            serde_json::to_vec(&compose(&follow_up).unwrap()).unwrap()
        );
    }
    follow_up.channel = "chat".into();
    assert!(!has_season(&follow_up));
    follow_up.channel = "email".into();
    follow_up.recipient = "boss".into();
    assert!(!has_season(&follow_up));
}

#[test]
fn client_uses_external_language_in_day_one_scenarios_and_checker() {
    for (scenario, expected) in [
        ("lateness", "到着いたします"),
        ("absence", "出席できなくなりました"),
    ] {
        let out = compose(&day_two_input(scenario, "client", "email")).unwrap();
        assert!(out.text.starts_with("佐藤商事\n佐藤様"));
        assert!(out.text.contains(expected));
    }
    let issues = check(
        "了解しました",
        &CheckContext {
            recipient: "client".into(),
            channel: "email".into(),
            lang: "ja".into(),
        },
    );
    assert!(issues.iter().any(|i| i.rule_id == "ryoukai.superior"));
}

#[test]
fn analyzer_scenario_guesses_are_available_to_compose() {
    for (received, expected) in [
        ("提出期限は11月25日です。", "extension"),
        ("打ち合わせの候補日はございますか。", "reschedule"),
        ("資料を添付いたします。", "thanks"),
    ] {
        let guess = analyze(received)
            .guesses
            .into_iter()
            .find(|guess| guess.field == "scenario")
            .unwrap();
        assert_eq!(guess.value, expected);
        assert!(compose(&day_two_input(&guess.value, "client", "email")).is_ok());
    }
}
