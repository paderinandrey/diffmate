use super::*;
use crate::agent::FailureKind;

#[test]
fn terminal_causes_and_manual_failures_never_retry() {
    let state = RetryState {
        retry_count: 1,
        max_retries: Some(2),
    };
    for (origin, failure, text) in [
        (
            JobOrigin::Manual,
            Failure::Shutdown,
            "interrupted: shutdown",
        ),
        (
            JobOrigin::Manual,
            Failure::UserCancel,
            "interrupted: user_cancel",
        ),
        (
            JobOrigin::Automatic,
            Failure::UserCancel,
            "interrupted: user_cancel",
        ),
        (JobOrigin::Manual, Failure::Crash, "interrupted: crash"),
        (JobOrigin::Manual, Failure::Timeout, "timeout"),
        (
            JobOrigin::Automatic,
            Failure::Terminal(FailureKind::AgentError),
            "agent_error",
        ),
        (
            JobOrigin::Automatic,
            Failure::Terminal(FailureKind::CliError),
            "cli_error",
        ),
        (
            JobOrigin::Automatic,
            Failure::Terminal(FailureKind::InvalidOutput),
            "invalid_output",
        ),
    ] {
        assert_eq!(JobOrigin::parse(origin.as_str()).unwrap(), origin);
        assert_eq!(
            after_failure(origin, &failure, state, 9, 0),
            JobNext::Fail {
                reason: text.into(),
                max_retries: Some(2),
            }
        );
    }
}

#[test]
fn zero_budget_fails_immediately_and_shutdown_never_uses_it() {
    let state = RetryState {
        retry_count: 0,
        max_retries: None,
    };
    assert_eq!(
        after_failure(JobOrigin::Automatic, &Failure::Crash, state, 0, 0),
        JobNext::Fail {
            reason: "interrupted: crash; no retries left (0 of 0 used)".into(),
            max_retries: Some(0),
        }
    );
    assert_eq!(
        after_failure(JobOrigin::Automatic, &Failure::Shutdown, state, 0, 0),
        JobNext::Requeue {
            retry_count: 0,
            max_retries: None,
            next_attempt_at: None,
        }
    );
}

#[test]
fn stored_budget_survives_a_config_change() {
    let state = RetryState {
        retry_count: 1,
        max_retries: Some(2),
    };
    assert_eq!(
        after_failure(JobOrigin::Automatic, &Failure::Timeout, state, 0, 17),
        JobNext::Requeue {
            retry_count: 2,
            max_retries: Some(2),
            next_attempt_at: Some(300_017),
        }
    );
    for (number, seconds) in [(1, 60), (2, 300), (7, 300)] {
        assert_eq!(retry_delay(number).as_secs(), seconds);
    }
}
