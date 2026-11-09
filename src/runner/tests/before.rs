use super::*;

pub(super) async fn cancelled_before_run(stop: Option<StopReason>, automatic: bool) {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let process = ProcessInfo {
        pid: 1,
        started_at_ms: 0,
    };
    let job = store
        .enqueue_review(
            &pr,
            &process,
            crate::store::jobs::EnqueueRequest {
                origin: crate::store::records::JobOrigin::Manual,
                force: false,
            },
        )
        .await
        .unwrap()
        .job;
    if automatic {
        store
            .call(|conn| {
                conn.execute("UPDATE jobs SET origin = 'automatic'", [])
                    .map(drop)
            })
            .await
            .unwrap();
    }
    if let Some(reason) = stop {
        store.start_job(job.job_id).await.unwrap();
        assert!(store.set_stop_reason(job.job_id, reason).await.unwrap());
    }
    let env = InheritedEnv::from_pairs(vec![("PATH".into(), std::env::var_os("PATH").unwrap())]);
    let deps = Deps {
        git: GitCli::new(&env),
        gh: GhCli::new(&env),
        agent: ClaudeCli::new(&env).unwrap(),
        settings: crate::config::Settings::default(),
    };
    let (_stop, cancel) = watch::channel(true);

    let result = execute_review_job(&paths, &store, &deps, &pr, &job, &cancel).await;

    assert!(matches!(
        result,
        ReviewResult::Cancelled { run_id: None, .. }
    ));
    assert!(!paths.runs_dir().exists());
    assert!(!paths.repos_dir().exists());
    let sql = "SELECT state, failure_reason, (SELECT count(*) FROM runs) FROM jobs";
    let row = store
        .call(move |conn| {
            conn.query_row(sql, [], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
        })
        .await
        .unwrap();
    let reason = "interrupted before the run started";
    let reason = stop.map_or_else(
        || reason.to_string(),
        |stop| format!("{reason} ({})", stop.as_str()),
    );
    if automatic {
        assert_eq!(row, ("queued".into(), None, 0));
        assert_eq!(store.stop_reason(job.job_id).await.unwrap(), None);
        assert!(matches!(
            result,
            ReviewResult::Cancelled {
                job_end: crate::runner::JobEnd::Requeued {
                    retry: None,
                    next_attempt_at: None
                },
                ..
            }
        ));
    } else {
        assert_eq!(row, ("failed".into(), Some(reason), 0));
        assert!(matches!(
            result,
            ReviewResult::Cancelled {
                job_end: crate::runner::JobEnd::Failed,
                ..
            }
        ));
    }
}
