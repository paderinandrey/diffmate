use super::classify;
use super::support::{captured, envelope, failure, finding, output, run, schemas};
use crate::agent::AgentOutcome;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude")
}

enum Expect {
    Success {
        findings: usize,
    },
    Failed {
        kind: &'static str,
        subtype: &'static str,
    },
}

fn expectation(case: &str) -> Expect {
    let failed = |kind, subtype| Expect::Failed { kind, subtype };
    match case {
        "success-findings" => Expect::Success { findings: 1 },
        "success-no-findings" => Expect::Success { findings: 0 },
        "error-max-turns" => failed("agent_error", "error_max_turns"),
        "error-unknown-model" => failed("agent_error", "success"),
        "error-not-logged-in" => failed("cli_error", "success"),
        "error-max-budget" => failed("agent_error", "error_max_budget_usd"),
        other => panic!("cases.json has {other} but the golden table has no expectation"),
    }
}

#[test]
fn every_recorded_envelope_is_classified_as_d01_says() {
    let cases = fs::read_to_string(fixtures().join("cases.json")).unwrap();
    let cases: Value = serde_json::from_str(&cases).unwrap();
    for (name, case) in cases.as_object().unwrap() {
        let file = fixtures().join(case["file"].as_str().unwrap());
        let stdout = fs::read_to_string(file).unwrap();
        let exit = i32::try_from(case["exit_code"].as_i64().unwrap()).unwrap();
        let outcome = classify(&captured(&stdout, exit), &schemas());
        match (expectation(name), outcome) {
            (Expect::Success { findings }, AgentOutcome::Succeeded { output, .. }) => {
                assert_eq!(output.findings.len(), findings, "{name}");
            }
            (
                Expect::Failed { kind, subtype },
                AgentOutcome::Failed {
                    kind: got, detail, ..
                },
            ) => {
                assert_eq!(got.as_str(), kind, "{name}");
                assert_eq!(detail.subtype.as_deref(), Some(subtype), "{name}");
            }
            _ => panic!("{name}: success and failure disagree with the expectation"),
        }
    }
}

#[test]
fn a_recorded_error_envelope_stays_failed_with_exit_zero() {
    let stdout = fs::read_to_string(fixtures().join("error-max-turns.stdout")).unwrap();
    let (kind, detail, metrics) = failure(classify(&captured(&stdout, 0), &schemas()));
    assert_eq!(kind, "agent_error");
    assert_eq!(detail.subtype.as_deref(), Some("error_max_turns"));
    assert_eq!(detail.terminal_reason, Some(json!("max_turns")));
    assert_eq!(metrics.total_cost_usd, Some(0.022_223));
    assert_eq!(metrics.duration_ms, Some(3635));
}

#[test]
fn cost_and_duration_are_stored_exactly() {
    let mut ok = envelope("success", false, Some(output("findings", &[finding()])));
    ok["total_cost_usd"] = json!(0.123_456_789);
    ok["duration_ms"] = json!(98765);
    let AgentOutcome::Succeeded { metrics, .. } = run(&ok, 0) else {
        panic!("expected success");
    };
    assert_eq!(
        (metrics.total_cost_usd, metrics.duration_ms),
        (Some(0.123_456_789), Some(98765))
    );
    let mut bad = envelope("error_max_turns", true, None);
    bad["total_cost_usd"] = json!(0.123_456_789);
    bad["duration_ms"] = json!(98765);
    let metrics = failure(run(&bad, 1)).2;
    assert_eq!(
        (metrics.total_cost_usd, metrics.duration_ms),
        (Some(0.123_456_789), Some(98765))
    );
}
