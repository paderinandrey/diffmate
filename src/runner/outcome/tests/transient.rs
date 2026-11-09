use super::*;
use serde_json::{Value, json};

fn agent_error(status: Option<Value>, message: &str) -> AgentOutcome {
    AgentOutcome::Failed {
        kind: FailureKind::AgentError,
        detail: FailureDetail {
            message: message.into(),
            api_error_status: status,
            ..FailureDetail::default()
        },
        metrics: Metrics::default(),
    }
}

#[tokio::test]
async fn a_rate_limit_requeues_and_auth_failures_do_not() {
    let fixture = Fixture::new().await;
    fixture.automatic().await;
    for (attempt, delay_ms) in [(1_u32, 60_000_i64), (2, 300_000)] {
        let before = crate::store::now_ms();
        let result = fixture
            .finish(agent_error(Some(json!(429)), "rate limit"))
            .await;
        let after = crate::store::now_ms();
        assert!(matches!(
            result,
            ReviewResult::RunFailed {
                kind: FailureKind::AgentError,
                job_end: JobEnd::Requeued {
                    retry: Some((got, 2)),
                    next_attempt_at: Some(at),
                },
                ..
            } if got == attempt && (before + delay_ms..=after + delay_ms).contains(&at)
        ));
    }
    let result = fixture
        .finish(agent_error(Some(json!(429)), "rate limit"))
        .await;
    assert!(matches!(
        result,
        ReviewResult::RunFailed {
            kind: FailureKind::AgentError,
            job_end: JobEnd::Failed,
            ..
        }
    ));
    let reason = fixture
        .store
        .call(|conn| {
            conn.query_row("SELECT failure_reason FROM jobs", [], |row| {
                row.get::<_, String>(0)
            })
        })
        .await
        .unwrap();
    assert!(reason.contains("no retries left"));

    for (status, message) in [(Some(json!(403)), "timeout"), (None, "Not logged in")] {
        let fixture = Fixture::new().await;
        fixture.automatic().await;
        let result = fixture.finish(agent_error(status, message)).await;
        assert!(matches!(
            result,
            ReviewResult::RunFailed {
                kind: FailureKind::AgentError,
                job_end: JobEnd::Failed,
                ..
            }
        ));
        let row = fixture
            .store
            .call(|conn| {
                conn.query_row(
                    "SELECT state, retry_count, failure_reason FROM jobs",
                    [],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, u32>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    },
                )
            })
            .await
            .unwrap();
        assert_eq!(
            (row.0.as_str(), row.1, row.2.as_str()),
            ("failed", 0, "agent_error")
        );
    }
}
