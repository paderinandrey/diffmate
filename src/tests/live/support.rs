use super::stored::{StoredRun, latest_run};
use crate::app::{AppConfig, ProcessInfo};
use crate::paths::Paths;
use crate::procenv::InheritedEnv;
use crate::store::now_ms;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

type Stat = Option<(u64, SystemTime)>;

pub(super) struct RealStateGuard {
    paths: Paths,
    before: [Stat; 2],
}

fn stat(path: &Path) -> Stat {
    let meta = fs::metadata(path).ok()?;
    Some((meta.len(), meta.modified().ok()?))
}

impl RealStateGuard {
    pub fn new() -> RealStateGuard {
        let home = std::env::var_os("HOME").unwrap();
        let paths = Paths::from_home(Path::new(&home));
        let before = [stat(&paths.db()), stat(&paths.runs_dir())];
        RealStateGuard { paths, before }
    }

    pub fn assert_untouched(&self) {
        let now = [stat(&self.paths.db()), stat(&self.paths.runs_dir())];
        assert_eq!(self.before, now, "the real diffmate state changed");
    }
}

pub(super) fn live_config(root: &Path) -> AppConfig {
    AppConfig {
        paths: Paths::new(root.join("state"), root.join("cache")),
        env: InheritedEnv::capture(),
        process: ProcessInfo {
            pid: std::process::id(),
            started_at_ms: now_ms(),
        },
    }
}

pub(super) fn load_live_prs() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/live-prs.json");
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

pub(super) async fn review_url(url: &str) -> StoredRun {
    let guard = RealStateGuard::new();
    let dir = tempfile::tempdir().unwrap();
    let config = live_config(dir.path());
    let code = crate::review::run(&config, url).await;
    let run = latest_run(&config.paths.db());
    println!("{url}: exit {code:?}\n{}", run.describe());
    guard.assert_untouched();
    run
}
