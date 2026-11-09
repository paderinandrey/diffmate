use super::*;

#[tokio::test]
async fn terminal_errors_and_user_cancel_end_the_job_live() {
    for (automatic, cancelled) in [(true, false), (true, true), (false, true)] {
        let fixture = Fixture::new().await;
        if automatic {
            fixture.automatic().await;
        }
        let outcome = if cancelled {
            fixture.stop(StopReason::UserCancel).await;
            AgentOutcome::Cancelled {
                metrics: Metrics::default(),
            }
        } else {
            AgentOutcome::Failed {
                kind: FailureKind::AgentError,
                detail: FailureDetail::default(),
                metrics: Metrics::default(),
            }
        };
        let result = fixture.finish(outcome).await;
        if cancelled {
            assert!(matches!(
                result,
                ReviewResult::Cancelled {
                    job_end: JobEnd::Failed,
                    ..
                }
            ));
        } else {
            assert!(matches!(
                result,
                ReviewResult::RunFailed {
                    kind: FailureKind::AgentError,
                    job_end: JobEnd::Failed,
                    ..
                }
            ));
        }
        let row = fixture.store.call(|conn| conn.query_row(
            "SELECT state, retry_count, max_retries, next_attempt_at, failure_reason FROM jobs",
            [], |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?,
                row.get::<_, Option<u32>>(2)?, row.get::<_, Option<i64>>(3)?, row.get::<_, String>(4)?)),
        )).await.unwrap();
        assert_eq!(
            (row.0.as_str(), row.1, row.2, row.3),
            ("failed", 0, None, None)
        );
        assert_eq!(
            row.4,
            if cancelled {
                "interrupted: user_cancel"
            } else {
                "agent_error"
            }
        );
    }
}
