use super::support::{envelope, failure, finding, output, run};
use crate::agent::{AgentOutcome, ReviewStatus};
use serde_json::{Value, json};

fn good() -> Value {
    envelope("success", false, Some(output("findings", &[finding()])))
}

#[test]
fn a_clean_envelope_succeeds() {
    match run(&good(), 0) {
        AgentOutcome::Succeeded { output, .. } => assert_eq!(output.status, ReviewStatus::Findings),
        AgentOutcome::Failed { .. } => panic!("expected success"),
    }
}

#[test]
fn error_flag_with_exit_zero_is_agent_error() {
    let value = envelope("success", true, Some(output("no_findings", &[])));
    let (kind, detail, _) = failure(run(&value, 0));
    assert_eq!(kind, "agent_error");
    assert_eq!(detail.is_error, Some(true));
}

#[test]
fn nonzero_exit_with_a_success_envelope_is_agent_error() {
    let (kind, detail, _) = failure(run(&good(), 1));
    assert_eq!((kind, detail.exit_code), ("agent_error", Some(1)));
}

#[test]
fn missing_structured_output_is_invalid_output() {
    for value in [
        envelope("success", false, None),
        envelope("success", false, Some(Value::Null)),
    ] {
        assert_eq!(failure(run(&value, 0)).0, "invalid_output");
    }
}

#[test]
fn string_structured_output_is_parsed_once() {
    let text = output("findings", &[finding()]).to_string();
    let value = envelope("success", false, Some(Value::String(text.clone())));
    assert!(matches!(run(&value, 0), AgentOutcome::Succeeded { .. }));
    let twice = envelope(
        "success",
        false,
        Some(Value::String(json!(text).to_string())),
    );
    assert_eq!(failure(run(&twice, 0)).0, "invalid_output");
}

#[test]
fn exhausted_structured_output_retries_is_invalid_output() {
    let value = envelope("error_max_structured_output_retries", true, None);
    let (kind, detail, _) = failure(run(&value, 1));
    assert_eq!(kind, "invalid_output");
    assert_eq!(
        detail.subtype.as_deref(),
        Some("error_max_structured_output_retries")
    );
}
