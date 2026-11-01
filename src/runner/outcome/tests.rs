mod fixture;
mod preservation;

use super::*;
use crate::store::records::Metrics;
use fixture::Fixture;

#[tokio::test]
async fn the_first_stored_stop_reason_wins() {
    let fixture = Fixture::new().await;
    let (store, job) = (&fixture.store, fixture.job.job_id);
    assert_eq!(store.stop_reason(job).await.unwrap(), None);
    assert!(
        store
            .set_stop_reason(job, StopReason::UserCancel)
            .await
            .unwrap()
    );
    assert!(
        !store
            .set_stop_reason(job, StopReason::Shutdown)
            .await
            .unwrap()
    );
    assert_eq!(
        store.stop_reason(job).await.unwrap(),
        Some(StopReason::UserCancel)
    );
}

#[tokio::test]
async fn a_timeout_cancellation_is_a_timeout_without_an_interrupted_cause() {
    let fixture = Fixture::new().await;
    fixture.stop(StopReason::Timeout).await;
    let outcome = AgentOutcome::Cancelled {
        metrics: Metrics::default(),
    };

    let result = fixture.finish(outcome).await;

    assert!(matches!(
        result,
        ReviewResult::RunFailed {
            kind: FailureKind::Timeout,
            ..
        }
    ));
    let sql = "SELECT state, failure_kind, interrupted_cause, failure_detail FROM runs";
    let row = fixture
        .store
        .call(move |conn| {
            conn.query_row(sql, [], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
        })
        .await
        .unwrap();
    assert_eq!(
        (row.0.as_str(), row.1.as_str(), row.2),
        ("failed", "timeout", None)
    );
    let detail: serde_json::Value = serde_json::from_str(&row.3).unwrap();
    assert_eq!(detail["message"], "stopped by the run timeout");
}
