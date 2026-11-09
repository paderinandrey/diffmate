use super::super::{AgentOutcome, Normalized, ReviewResult, Revision, StopReason, Store};
use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::RunId;
use crate::store::jobs::EnqueuedJob;
use tempfile::TempDir;

pub(super) struct Fixture {
    pub store: Store,
    pub job: EnqueuedJob,
    run: RunId,
    revision: Revision,
    _tmp: TempDir,
}

impl Fixture {
    pub async fn automatic(&self) {
        self.store
            .call(|conn| {
                conn.execute("UPDATE jobs SET origin = 'automatic'", [])
                    .map(drop)
            })
            .await
            .unwrap();
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
        store.start_job(job.job_id).await.unwrap();
        let revision = Revision {
            head_sha: "head".into(),
            merge_base_sha: "base".into(),
            base_tip_sha: "tip".into(),
            base_ref: "main".into(),
        };
        let run = store.insert_running_run(&job, &revision).await.unwrap();
        Self {
            store,
            job,
            run,
            revision,
            _tmp: tmp,
        }
    }

    pub async fn stop(&self, reason: StopReason) {
        assert!(
            self.store
                .set_stop_reason(self.job.job_id, reason)
                .await
                .unwrap()
        );
    }

    pub async fn finish(&self, outcome: AgentOutcome) -> ReviewResult {
        super::super::store(
            &self.store,
            (&self.job, self.run, &self.revision),
            (outcome, Normalized::default()),
            Vec::new(),
            &crate::config::Settings::default(),
        )
        .await
    }
}
