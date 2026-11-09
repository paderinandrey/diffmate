use super::*;
use crate::agent::{AgentOutcome, FailureKind};
use crate::cancel::StopReason;
use crate::runner::ReviewResult;
use crate::store::records::Metrics;

#[tokio::test]
async fn a_retry_that_fails_before_its_run_replaces_the_earlier_timeout() {
    let fixture = Fixture::new().await;
    fixture.automatic().await;
    fixture.stop(StopReason::Timeout).await;
    fixture
        .finish(AgentOutcome::Cancelled {
            metrics: Metrics::default(),
        })
        .await;
    fixture
        .store
        .call(|conn| {
            conn.execute("UPDATE runs SET finished_at = 1", [])
                .map(drop)
        })
        .await
        .unwrap();
    let reason = "prepare: gh auth invalid";
    fixture
        .store
        .fail_job(fixture.job.job_id, reason.into())
        .await
        .unwrap();
    let stored = fixture
        .store
        .stored_result(fixture.job.job_id)
        .await
        .unwrap();
    assert!(matches!(
        ReviewResult::from_stored(stored, false),
        ReviewResult::JobFailed { reason: got } if got == reason
    ));
}

#[tokio::test]
async fn a_terminal_run_failure_stays_the_stored_result() {
    let fixture = Fixture::new().await;
    let result = fixture
        .finish(AgentOutcome::Failed {
            kind: FailureKind::AgentError,
            detail: crate::agent::FailureDetail {
                message: "boom".into(),
                ..crate::agent::FailureDetail::default()
            },
            metrics: Metrics::default(),
        })
        .await;
    assert!(matches!(result, ReviewResult::RunFailed { .. }));
    let stored = fixture
        .store
        .stored_result(fixture.job.job_id)
        .await
        .unwrap();
    assert!(matches!(
        ReviewResult::from_stored(stored, false),
        ReviewResult::RunFailed {
            kind: FailureKind::AgentError,
            ref message,
            ..
        } if message == "boom"
    ));
}
