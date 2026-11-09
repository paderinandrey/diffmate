use super::*;
use crate::store::jobs::EnqueuedJob;

pub(super) struct Fixture {
    pub paths: Paths,
    pub store: Store,
    pub job: EnqueuedJob,
    pub git: FakeGit,
    _tmp: TempDir,
}

impl Fixture {
    pub async fn recover(&self) -> super::super::RecoveryReport {
        recover(
            &self.paths,
            &self.store,
            &self.git,
            &crate::config::Settings::default(),
        )
        .await
        .unwrap()
    }

    pub async fn snapshot(&self) -> String {
        self.store
            .call(|conn| {
                conn.query_row(
                    "SELECT json_object('job', json_group_array(json_array(state, retry_count,
                max_retries, next_attempt_at, stop_reason, failure_reason, updated_at)),
                'runs', (SELECT json_group_array(json_array(state, failure_kind,
                    interrupted_cause, finished_at)) FROM runs)) FROM jobs",
                    [],
                    |row| row.get(0),
                )
            })
            .await
            .unwrap()
    }

    pub async fn new() -> Self {
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
        store
            .call(|conn| {
                conn.execute("UPDATE jobs SET origin = 'automatic'", [])
                    .map(drop)
            })
            .await
            .unwrap();
        Self {
            paths,
            store,
            job,
            git: FakeGit::default(),
            _tmp: tmp,
        }
    }

    pub async fn start(&self) -> i64 {
        self.store.start_job(self.job.job_id).await.unwrap();
        self.store
            .insert_running_run(&self.job, &revision())
            .await
            .unwrap()
    }

    pub async fn row(&self) -> (String, u32, Option<u32>, Option<i64>, Option<String>) {
        self.store
            .call(|conn| {
                conn.query_row(
            "SELECT state, retry_count, max_retries, next_attempt_at, failure_reason FROM jobs",
            [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
            })
            .await
            .unwrap()
    }
}
