use super::support::{NOW, agent, detail, open, status};
use super::*;

#[test]
fn a_rate_limit_uses_the_same_budget_as_a_timeout() {
    let failure = agent(&detail(Some(status(429)), "overload"));
    assert_eq!(
        after_failure(JobOrigin::Automatic, &failure, open(0, None), 2, NOW),
        JobNext::Requeue {
            retry_count: 1,
            max_retries: Some(2),
            next_attempt_at: Some(NOW + 60_000),
        }
    );
    assert_eq!(
        after_failure(JobOrigin::Automatic, &failure, open(1, Some(2)), 9, NOW),
        JobNext::Requeue {
            retry_count: 2,
            max_retries: Some(2),
            next_attempt_at: Some(NOW + 300_000),
        }
    );
    assert_eq!(
        after_failure(JobOrigin::Automatic, &failure, open(2, Some(2)), 9, NOW),
        JobNext::Fail {
            reason: "agent_error; no retries left (2 of 2 used)".into(),
            max_retries: Some(2),
        }
    );
    assert_eq!(
        after_failure(JobOrigin::Manual, &failure, open(0, None), 2, NOW),
        JobNext::Fail {
            reason: "agent_error".into(),
            max_retries: None,
        }
    );
}

#[test]
fn auth_and_other_agent_errors_fail_immediately() {
    for sample in [
        detail(Some(status(401)), "rate limit"),
        detail(Some(status(403)), "timeout"),
        detail(None, "Not logged in"),
        detail(Some(serde_json::Value::Null), "Not logged in"),
        detail(Some(serde_json::json!("429")), "timeout"),
    ] {
        assert_eq!(
            after_failure(JobOrigin::Automatic, &agent(&sample), open(0, None), 2, NOW),
            JobNext::Fail {
                reason: "agent_error".into(),
                max_retries: None,
            }
        );
    }
}

#[test]
fn a_server_error_with_no_budget_names_the_give_up() {
    let failure = agent(&detail(Some(status(500)), "ok"));
    assert_eq!(
        after_failure(JobOrigin::Automatic, &failure, open(0, None), 0, NOW),
        JobNext::Fail {
            reason: "agent_error; no retries left (0 of 0 used)".into(),
            max_retries: Some(0),
        }
    );
}

#[test]
fn shutdown_still_requeues_without_spending_a_retry() {
    let state = open(3, Some(1));
    assert_eq!(
        after_failure(JobOrigin::Automatic, &Failure::Shutdown, state, 0, NOW),
        JobNext::Requeue {
            retry_count: 3,
            max_retries: Some(1),
            next_attempt_at: None,
        }
    );
}
