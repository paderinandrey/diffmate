use super::*;

#[tokio::test]
async fn automatic_shutdown_and_timeout_apply_retry_rules_live() {
    for stop in [StopReason::Shutdown, StopReason::Timeout] {
        let fixture = Fixture::new().await;
        fixture.automatic().await;
        fixture.stop(stop).await;
        let result = fixture
            .finish(AgentOutcome::Cancelled {
                metrics: Metrics::default(),
            })
            .await;
        let row = fixture
            .store
            .call(|conn| {
                conn.query_row(
            "SELECT j.state, j.retry_count, j.max_retries, j.next_attempt_at, j.stop_reason,
                r.failure_kind, r.interrupted_cause FROM jobs j JOIN runs r ON r.job_id = j.id",
            [], |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?,
                row.get::<_, Option<u32>>(2)?, row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<String>>(4)?, row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?)),
        )
            })
            .await
            .unwrap();
        assert_eq!(row.0, "queued");
        assert_eq!(row.4, None);
        match stop {
            StopReason::Shutdown => {
                assert_eq!((row.1, row.2, row.3), (0, None, None));
                assert_eq!(
                    (row.5.as_str(), row.6.as_deref()),
                    ("interrupted", Some("shutdown"))
                );
                assert!(matches!(
                    result,
                    ReviewResult::Cancelled {
                        job_end: JobEnd::Requeued {
                            retry: None,
                            next_attempt_at: None
                        },
                        ..
                    }
                ));
            }
            StopReason::Timeout => {
                assert_eq!((row.1, row.2), (1, Some(2)));
                assert!(row.3.is_some());
                assert_eq!((row.5.as_str(), row.6), ("timeout", None));
                assert!(matches!(
                    result,
                    ReviewResult::RunFailed {
                        kind: FailureKind::Timeout,
                        job_end: JobEnd::Requeued {
                            retry: Some((1, 2)),
                            next_attempt_at: Some(_)
                        },
                        ..
                    }
                ));
            }
            StopReason::UserCancel => unreachable!(),
        }
    }
}
