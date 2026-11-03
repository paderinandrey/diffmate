use super::fixture::Fixture;
use crate::cancel::StopReason;

#[tokio::test]
async fn stored_causes_survive_recovery_and_a_second_recovery_is_inert() {
    for stop in [
        StopReason::Shutdown,
        StopReason::UserCancel,
        StopReason::Timeout,
    ] {
        let fixture = Fixture::new().await;
        let run = fixture.start().await;
        fixture
            .store
            .set_stop_reason(fixture.job.job_id, stop)
            .await
            .unwrap();
        tokio::fs::create_dir_all(fixture.paths.worktree(run))
            .await
            .unwrap();
        let report = fixture.recover().await;
        let row = fixture.row().await;
        let stored = fixture.store.call(|conn| conn.query_row(
            "SELECT state, failure_kind, interrupted_cause, (SELECT stop_reason FROM jobs) FROM runs",
            [], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?, row.get::<_, Option<String>>(3)?)),
        )).await.unwrap();
        assert_eq!(stored.0, "failed");
        let (kind, cause, job_state, retries, budget) = match stop {
            StopReason::Shutdown => ("interrupted", Some("shutdown"), "queued", 0, None),
            StopReason::UserCancel => ("interrupted", Some("user_cancel"), "failed", 0, None),
            StopReason::Timeout => ("timeout", None, "queued", 1, Some(2)),
        };
        assert_eq!((stored.1.as_str(), stored.2.as_deref()), (kind, cause));
        assert_eq!((row.0.as_str(), row.1, row.2), (job_state, retries, budget));
        if stop == StopReason::UserCancel {
            assert_eq!(row.4.as_deref(), Some("interrupted: user_cancel"));
            assert_eq!(stored.3.as_deref(), Some("user_cancel"));
        } else {
            assert_eq!(stored.3, None);
        }
        if stop == StopReason::Shutdown {
            assert_eq!(row.3, None);
        }
        assert_eq!(report.removed_run_dirs, [run]);
        assert!(!fixture.paths.run_dir(run).exists());
        assert_eq!(fixture.git.calls.lock().unwrap().len(), 1);
        let before = fixture.snapshot().await;
        assert_eq!(fixture.recover().await.lines(), Vec::<String>::new());
        assert_eq!(fixture.snapshot().await, before);
    }
}

#[tokio::test]
async fn zero_budget_is_stored_and_never_retried() {
    let fixture = Fixture::new().await;
    fixture.start().await;
    let settings =
        crate::config::parse("max_retries = 0", std::path::Path::new("config.toml")).unwrap();
    super::recover(&fixture.paths, &fixture.store, &fixture.git, &settings)
        .await
        .unwrap();
    let row = fixture.row().await;
    assert_eq!(
        (row.0.as_str(), row.1, row.2, row.3),
        ("failed", 0, Some(0), None)
    );
    assert_eq!(
        row.4.as_deref(),
        Some("interrupted: crash; no retries left (0 of 0 used)")
    );
}
