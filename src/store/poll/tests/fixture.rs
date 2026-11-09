use super::super::{Absent, SeenRequest};
use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::Store;
use rusqlite::OptionalExtension;
use tempfile::TempDir;

pub(super) struct Fix {
    pub store: Store,
    _tmp: TempDir,
}

pub(super) struct Target {
    pub tracking: String,
    pub seen: bool,
    pub queued_head: Option<String>,
    pub head: Option<String>,
    pub checked: Option<i64>,
}

pub(super) fn pr(number: u64) -> PrRef {
    PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number,
    }
}

pub(super) const fn process() -> ProcessInfo {
    ProcessInfo {
        pid: 7,
        started_at_ms: 11,
    }
}

pub(super) fn seen(number: u64, draft: bool, head: Option<&str>) -> SeenRequest {
    SeenRequest {
        pr: pr(number),
        is_draft: draft,
        head_sha: head.map(str::to_string),
    }
}

pub(super) fn absent_open(draft: bool, head: Option<&str>) -> Absent {
    Absent::Open {
        is_draft: draft,
        head_sha: head.map(str::to_string),
    }
}

pub(super) fn closed(head: Option<&str>) -> Absent {
    Absent::Closed {
        head_sha: head.map(str::to_string),
    }
}

impl Fix {
    pub(super) async fn open() -> Self {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
        Self {
            store: Store::open(&paths).await.unwrap(),
            _tmp: tmp,
        }
    }

    pub(super) async fn seed(&self, number: u64, tracked: bool, seen: bool) {
        let tracking = if tracked { "tracked" } else { "untracked" };
        self.store
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO targets (owner, repo, number, tracking, request_seen, created_at)
                     VALUES ('o', 'r', ?1, ?2, ?3, 0)",
                    rusqlite::params![i64::try_from(number).unwrap(), tracking, i64::from(seen)],
                )
                .map(drop)
            })
            .await
            .unwrap();
    }

    pub(super) async fn target(&self, number: u64) -> Option<Target> {
        self.store
            .call(move |conn| {
                conn.query_row(
                    "SELECT tracking, request_seen, queued_head, head_sha, head_checked_at
                     FROM targets WHERE owner = 'o' AND repo = 'r' AND number = ?1",
                    [i64::try_from(number).unwrap()],
                    |row| {
                        Ok(Target {
                            tracking: row.get(0)?,
                            seen: row.get::<_, i64>(1)? == 1,
                            queued_head: row.get(2)?,
                            head: row.get(3)?,
                            checked: row.get(4)?,
                        })
                    },
                )
                .optional()
            })
            .await
            .unwrap()
    }
}
