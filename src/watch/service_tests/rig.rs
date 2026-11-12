use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use tokio::sync::watch;

use super::remote::{Bot, Remote};
use crate::app::{AppConfig, ProcessInfo};
use crate::config::Settings;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::procenv::InheritedEnv;
use crate::runner::{Deps, FakeGit};
use crate::store::Store;
use crate::store::jobs::EnqueueRequest;
use crate::store::records::JobOrigin;

const SEED: &str = "\
INSERT INTO targets (id, owner, repo, number, tracking, created_at) VALUES (1, 'o', 'r', 1, 'tracked', 1);
INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', 'abc123', 'base', 'base', 'main', 0);
INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{}', 0);
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('f1', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Title', 'Body text', 'kept');";

pub(super) struct Rig {
    _dir: tempfile::TempDir,
    pub config: AppConfig,
    pub store: Store,
    pub posts: Arc<AtomicUsize>,
    gate: watch::Sender<bool>,
    pub seen: watch::Receiver<bool>,
    pub review: i64,
}

impl Rig {
    pub async fn open() -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let paths = Paths::new(dir.path().join("state"), dir.path().join("cache"));
        let store = Store::open(&paths).await.unwrap();
        let sql = SEED.to_string();
        store
            .call(move |conn| conn.execute_batch(&sql))
            .await
            .unwrap();
        let pr = PrRef {
            owner: "o".into(),
            repo: "r".into(),
            number: 1,
        };
        let process = ProcessInfo {
            pid: 1,
            started_at_ms: 0,
        };
        let queued = store
            .enqueue_review(
                &pr,
                &process,
                EnqueueRequest {
                    origin: JobOrigin::Manual,
                    force: false,
                },
            )
            .await
            .unwrap();
        let (gate, seen) = watch::channel(false);
        let config = AppConfig {
            paths,
            env: InheritedEnv::from_pairs(Vec::new()),
            process,
            config_file: PathBuf::from("config.toml"),
            settings: Settings::default(),
        };
        Self {
            _dir: dir,
            config,
            store,
            posts: Arc::new(AtomicUsize::new(0)),
            gate,
            seen,
            review: queued.job.job_id,
        }
    }

    pub fn deps(&self, hold: bool) -> Deps<FakeGit, Remote, Bot> {
        Deps {
            git: FakeGit::default(),
            gh: Remote {
                posts: Arc::clone(&self.posts),
                hang: false,
            },
            agent: Bot(hold.then(|| self.gate.clone())),
            settings: self.config.settings.clone(),
        }
    }

    pub fn deps_hanging(&self) -> Deps<FakeGit, Remote, Bot> {
        let mut deps = self.deps(true);
        deps.gh.hang = true;
        deps
    }
}
pub(super) fn end_watch() {
    let signal = rustix::process::Signal::TERM;
    rustix::process::kill_process(rustix::process::getpid(), signal).unwrap();
}
