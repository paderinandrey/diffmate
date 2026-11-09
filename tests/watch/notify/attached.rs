use super::super::super::{URL, debug_enqueue, with_pr};
use super::super::config;
use super::support;
use crate::support::sandbox::Sandbox;

const REASON: &str = "quota-marker-zz";

fn envelope(reason: &str) -> String {
    format!(
        "{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":true,\"api_error_status\":429,\"duration_ms\":1,\"duration_api_ms\":1,\"num_turns\":1,\"result\":\"{reason}\",\"total_cost_usd\":0,\"session_id\":\"00000000-0000-0000-0000-000000000000\"}}"
    )
}

fn run_review(automatic: bool) -> (String, String) {
    let sandbox = Sandbox::new();
    config(&sandbox, "max_retries = 0\n");
    let script = support::install(&sandbox);
    with_pr(&sandbox);
    sandbox.claude.set_run(&envelope(REASON), 1);
    if automatic {
        assert!(
            debug_enqueue(&sandbox, URL, &["--automatic"])
                .status
                .success()
        );
    }
    let output = sandbox.run(&["review", URL]);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        support::count_sql(&sandbox, "SELECT count(*) FROM jobs WHERE state = 'failed'") == 1,
        "{stderr}"
    );
    (stderr, support::text(&script))
}

#[test]
fn review_notifies_when_it_finishes_an_automatic_give_up() {
    let (stderr, log) = run_review(true);
    let scripts = support::scripts(&log);
    assert_eq!(scripts.len(), 1, "{stderr}");
    assert!(
        scripts[0].contains("diffmate: review failed"),
        "{scripts:?}"
    );
    assert!(scripts[0].contains("o/r#1"), "{scripts:?}");
    assert!(scripts[0].contains(REASON), "{scripts:?}");
    assert!(!stderr.contains("notification-failed"), "{stderr}");
}

#[test]
fn review_stays_silent_when_a_manual_job_fails() {
    let (stderr, log) = run_review(false);
    assert_eq!(support::scripts(&log), Vec::<&str>::new(), "{stderr}");
}
