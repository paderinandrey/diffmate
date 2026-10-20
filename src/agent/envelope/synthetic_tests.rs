use super::classify;
use super::support::{captured, envelope, failure, finding, output, run, schemas};
use serde_json::{Value, json};

#[test]
fn garbage_and_empty_stdout_are_invalid_output() {
    for (stdout, exit) in [("garbage", 1), ("", 0), ("[1]", 0), ("null", 0), ("{", 1)] {
        let (kind, detail, metrics) = failure(classify(&captured(stdout, exit), &schemas()));
        assert_eq!(kind, "invalid_output", "{stdout:?}");
        assert!(detail.message.contains("not a result envelope"));
        assert_eq!(detail.stderr_excerpt.as_deref(), Some("stderr text"));
        assert!(metrics.total_cost_usd.is_none() && metrics.duration_ms.is_none());
    }
}

#[test]
fn padded_stdout_is_still_an_envelope() {
    let good = envelope("success", false, Some(output("no_findings", &[])));
    let padded = format!("\n  {good}\n\n");
    let outcome = classify(&captured(&padded, 0), &schemas());
    assert!(matches!(
        outcome,
        crate::agent::AgentOutcome::Succeeded { .. }
    ));
}

#[test]
fn oversized_stdout_is_invalid_output() {
    let mut big = captured("{}", 0);
    big.stdout_overflow = true;
    let (kind, detail, _) = failure(classify(&big, &schemas()));
    assert_eq!(kind, "invalid_output");
    assert!(detail.message.contains("capture limit"));
}

#[test]
fn schema_violations_are_invalid_output() {
    let mut extra = output("findings", &[finding()]);
    extra["surprise"] = Value::Bool(true);
    let mut bad_line = finding();
    bad_line["line"] = json!(0);
    for structured in [
        extra,
        output("findings", &[bad_line]),
        json!({"status": "findings"}),
    ] {
        let value = envelope("success", false, Some(structured));
        assert_eq!(failure(run(&value, 0)).0, "invalid_output");
    }
}

#[test]
fn status_and_findings_must_agree() {
    for structured in [output("no_findings", &[finding()]), output("findings", &[])] {
        let value = envelope("success", false, Some(structured));
        let (kind, detail, _) = failure(run(&value, 0));
        assert_eq!(kind, "invalid_output");
        assert!(detail.message.contains("status disagrees"));
    }
}

fn logged_out(status: &Value) -> Value {
    json!({"subtype": "success", "is_error": true, "terminal_reason": "api_error",
        "api_error_status": status, "result": "Not logged in · Please run /login"})
}

#[test]
fn only_the_recorded_signal_means_not_authenticated() {
    assert_eq!(failure(run(&logged_out(&Value::Null), 1)).0, "cli_error");
    assert_eq!(failure(run(&logged_out(&json!(404)), 1)).0, "agent_error");
    let mut other_text = logged_out(&Value::Null);
    other_text["result"] = json!("rate limited");
    assert_eq!(failure(run(&other_text, 1)).0, "agent_error");
}

#[test]
fn failure_detail_keeps_the_envelope_fields() {
    let mut value = envelope("error_max_turns", true, None);
    value["errors"] = json!(["Reached maximum number of turns (1)"]);
    value["terminal_reason"] = json!("max_turns");
    value["api_error_status"] = json!(429);
    let (_, detail, _) = failure(run(&value, 1));
    assert_eq!(detail.subtype.as_deref(), Some("error_max_turns"));
    assert_eq!(
        detail.errors,
        Some(json!(["Reached maximum number of turns (1)"]))
    );
    assert_eq!(detail.terminal_reason, Some(json!("max_turns")));
    assert_eq!(detail.api_error_status, Some(json!(429)));
    assert!(detail.message.contains("Reached maximum number of turns"));
}
