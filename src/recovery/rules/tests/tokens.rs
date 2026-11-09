use super::support::{agent, detail, status};
use super::*;
use crate::agent::FailureKind;

#[test]
fn retryable_statuses_ignore_the_message() {
    for code in [408_u64, 429, 500, 599] {
        let sample = detail(Some(status(code)), "api_error");
        assert!(is_transient(&sample), "{code}");
        assert!(matches!(agent(&sample), Failure::TransientAgent));
    }
    for code in [400_u64, 401, 403, 407, 499, 600] {
        let sample = detail(Some(status(code)), "timeout");
        assert!(!is_transient(&sample), "{code}");
        assert!(matches!(
            agent(&sample),
            Failure::Terminal(FailureKind::AgentError)
        ));
    }
}

#[test]
fn a_missing_status_matches_only_the_fixed_tokens() {
    for token in [
        "overload",
        "RATE LIMIT",
        "rate_limit",
        "Too Many Requests",
        "network",
        "Connection Reset",
        "connection refused",
        "Server Error",
        "timed out",
        "TIMEOUT",
        "econnreset",
        "ECONNREFUSED",
    ] {
        assert!(is_transient(&detail(None, token)), "{token}");
        let null = detail(Some(serde_json::Value::Null), token);
        assert!(is_transient(&null), "{token}");
    }
    assert!(!is_transient(&detail(None, "api_error")));
    assert!(!is_transient(&detail(
        Some(serde_json::json!("429")),
        "timeout"
    )));
    let mut errors = detail(None, "api_error");
    errors.errors = Some(serde_json::json!(["connection refused"]));
    assert!(is_transient(&errors));
    let mut subtype = detail(None, "api_error");
    subtype.subtype = Some("rate_limit".into());
    assert!(is_transient(&subtype));
    let mut terminal = detail(None, "api_error");
    terminal.terminal_reason = Some(serde_json::json!({"why": "server error"}));
    assert!(is_transient(&terminal));
    let mut stderr = detail(None, "api_error");
    stderr.stderr_excerpt = Some("timeout".into());
    assert!(!is_transient(&stderr));
}
