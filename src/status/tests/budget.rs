use super::super::budget;
use crate::store::status::RetryView;

fn retry() -> RetryView {
    RetryView {
        job_state: Some("queued".into()),
        origin: Some("automatic".into()),
        retry_count: 1,
        max_retries: Some(2),
        next_attempt_at: Some(250_000),
        has_failed_run: true,
    }
}

#[test]
fn queued_failures_show_retry_count_and_remaining_delay() {
    let mut job = retry();
    assert_eq!(
        budget::describe(&job, 10_000).as_deref(),
        Some("retry 1 of 2 in 4m")
    );
    job.next_attempt_at = Some(9_999);
    assert_eq!(
        budget::describe(&job, 10_000).as_deref(),
        Some("retry 1 of 2 due")
    );
    job.next_attempt_at = Some(10_000);
    assert_eq!(
        budget::describe(&job, 10_000).as_deref(),
        Some("retry 1 of 2 due")
    );
    job.next_attempt_at = None;
    assert_eq!(budget::describe(&job, 10_000).as_deref(), Some("requeued"));
}

#[test]
fn automatic_failures_report_exhausted_and_zero_budgets() {
    let mut job = retry();
    job.job_state = Some("failed".into());
    for count in [2, 0] {
        job.retry_count = count;
        job.max_retries = Some(count);
        assert_eq!(
            budget::describe(&job, 10_000),
            Some(format!("no retries left ({count} of {count} used)"))
        );
    }
}

#[test]
fn other_job_states_and_unused_budgets_have_no_retry_text() {
    let mut job = retry();
    for state in ["running", "succeeded"] {
        job.job_state = Some(state.into());
        assert_eq!(budget::describe(&job, 10_000), None);
    }
    job.job_state = Some("failed".into());
    assert_eq!(budget::describe(&job, 10_000), None);
    job.retry_count = 2;
    job.origin = Some("manual".into());
    assert_eq!(budget::describe(&job, 10_000), None);
    job.job_state = Some("queued".into());
    job.has_failed_run = false;
    assert_eq!(budget::describe(&job, 10_000), None);
}
