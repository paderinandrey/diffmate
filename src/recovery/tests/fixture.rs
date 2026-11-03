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
        let job = store.enqueue_review(&pr, &process).await.unwrap();
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
