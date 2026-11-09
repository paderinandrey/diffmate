use super::*;
use crate::agent::ReviewStatus;

#[tokio::test]
async fn a_successful_result_that_finished_first_keeps_its_draft() {
    let fixture = Fixture::new().await;
    fixture.stop(StopReason::UserCancel).await;
    let output = ReviewOutput {
        status: ReviewStatus::NoFindings,
        limitation: None,
        findings: Vec::new(),
        future_improvements: Vec::new(),
    };
    let outcome = AgentOutcome::Succeeded {
        output,
        raw: serde_json::json!({"status": "no_findings"}),
        metrics: Metrics::default(),
    };

    assert!(matches!(
        fixture.finish(outcome).await,
        ReviewResult::Succeeded { .. }
    ));

    let row = fixture
        .store
        .call(|conn| {
            let sql = "SELECT state, result, (SELECT count(*) FROM drafts),
                   (SELECT state FROM jobs), (SELECT stop_reason FROM jobs) FROM runs";
            conn.query_row(sql, [], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
        })
        .await
        .unwrap();
    assert_eq!(
        row,
        (
            "succeeded".into(),
            "no_findings".into(),
            1,
            "succeeded".into(),
            "user_cancel".into()
        )
    );
}

#[tokio::test]
async fn a_failed_result_that_finished_first_keeps_its_original_failure() {
    let fixture = Fixture::new().await;
    fixture.stop(StopReason::UserCancel).await;
    let outcome = AgentOutcome::Failed {
        kind: FailureKind::AgentError,
        detail: FailureDetail {
            message: "agent failed".into(),
            ..FailureDetail::default()
        },
        metrics: Metrics::default(),
    };

    assert!(matches!(
        fixture.finish(outcome).await,
        ReviewResult::RunFailed {
            kind: FailureKind::AgentError,
            ..
        }
    ));

    let row = fixture
        .store
        .call(|conn| {
            conn.query_row(
                "SELECT failure_kind, interrupted_cause FROM runs",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
            )
        })
        .await
        .unwrap();
    assert_eq!(row, ("agent_error".into(), None));
}
