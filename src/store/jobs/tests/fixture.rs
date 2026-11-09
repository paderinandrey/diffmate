use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::jobs::{EnqueueRequest, Enqueued};
use crate::store::{Store, records::JobOrigin};
use tempfile::TempDir;

pub(super) struct Fixture {
    pub store: Store,
    _tmp: TempDir,
}

pub(super) struct Row {
    pub id: i64,
    pub state: String,
    pub origin: String,
    pub force: bool,
}

impl Fixture {
    pub async fn new(tracked: bool) -> Self {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
        let store = Store::open(&paths).await.unwrap();
        store.call(move |conn| conn.execute(
            "INSERT INTO targets (owner, repo, number, tracking, created_at) VALUES ('o', 'r', 1, ?1, 0)",
            [if tracked { "tracked" } else { "untracked" }],
        ).map(drop)).await.unwrap();
        Self { store, _tmp: tmp }
    }

    pub async fn enqueue(&self, force: bool, automatic: bool) -> Enqueued {
        let pr = PrRef {
            owner: "o".into(),
            repo: "r".into(),
            number: 1,
        };
        let process = ProcessInfo {
            pid: 42,
            started_at_ms: 11,
        };
        let origin = if automatic {
            JobOrigin::Automatic
        } else {
            JobOrigin::Manual
        };
        self.store
            .enqueue_review(&pr, &process, EnqueueRequest { origin, force })
            .await
            .unwrap()
    }

    pub async fn rows(&self) -> Vec<Row> {
        self.store
            .call(|conn| {
                conn.prepare("SELECT id, state, origin, force FROM jobs ORDER BY id")?
                    .query_map([], |row| {
                        Ok(Row {
                            id: row.get(0)?,
                            state: row.get(1)?,
                            origin: row.get(2)?,
                            force: row.get(3)?,
                        })
                    })?
                    .collect()
            })
            .await
            .unwrap()
    }

    pub async fn tracked(&self) -> bool {
        self.store
            .call(|conn| {
                conn.query_row("SELECT tracking = 'tracked' FROM targets", [], |r| r.get(0))
            })
            .await
            .unwrap()
    }
}
