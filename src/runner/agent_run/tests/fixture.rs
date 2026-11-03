use super::*;
use crate::app::ProcessInfo;
use crate::config;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::runner::{Deps, execute_review_job};
use crate::store::jobs::{EnqueueRequest, EnqueuedJob};
use crate::store::{RunId, Store, records::Revision};
use std::path::Path;
use tempfile::TempDir;

pub(super) struct Fixture {
    _dir: TempDir,
    pub paths: Paths,
    pub store: Store,
    pub pr: PrRef,
    pub job: EnqueuedJob,
    pub deps: Deps<FakeGit, FakeGh, FakeAgent>,
}

impl Fixture {
    pub async fn new(origin: JobOrigin) -> Self {
        let dir = TempDir::new().unwrap();
        let paths = Paths::from_home(dir.path());
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
        let request = EnqueueRequest {
            origin,
            force: false,
        };
        let job = store
            .enqueue_review(&pr, &process, request)
            .await
            .unwrap()
            .job;
        let settings = config::parse("run_timeout_minutes = 1", Path::new("config.toml")).unwrap();
        let deps = Deps {
            git: FakeGit::default(),
            gh: FakeGh,
            agent: FakeAgent::WaitForCancel,
            settings,
        };
        Self {
            _dir: dir,
            paths,
            store,
            pr,
            job,
            deps,
        }
    }

    pub async fn run(&self, cancel: &crate::cancel::CancelRx) -> ReviewResult {
        execute_review_job(
            &self.paths,
            &self.store,
            &self.deps,
            &self.pr,
            &self.job,
            cancel,
        )
        .await
    }

    pub async fn seed_run(&self) -> RunId {
        self.store.start_job(self.job.job_id).await.unwrap();
        let revision = Revision {
            head_sha: "h".repeat(40),
            merge_base_sha: "b".repeat(40),
            base_tip_sha: "b".repeat(40),
            base_ref: "main".into(),
        };
        self.store
            .insert_running_run(&self.job, &revision)
            .await
            .unwrap()
    }
}
