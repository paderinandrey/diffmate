use super::stored::{StoredRun, latest_run};
use crate::app::{AppConfig, ProcessInfo};
use crate::gh::{GhCli, verified_view};
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
        config_file: root.join("config.toml"),
        settings: crate::config::Settings::default(),
    }
}

pub(super) fn load_live_prs() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/live-prs.json");
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

pub(super) async fn review_fixture(fixture: &Value) -> StoredRun {
    let url = fixture["url"].as_str().unwrap();
    let expected = fixture["head_sha"].as_str().unwrap();
    let guard = RealStateGuard::new();
    let dir = tempfile::tempdir().unwrap();
    let config = live_config(dir.path());
    let pr = crate::pr_url::parse(url).unwrap();
    let head = verified_view(&GhCli::new(&config.env), &pr)
        .await
        .unwrap_or_else(|err| panic!("cannot check fixture HEAD for {url}: {err}"))
        .head_sha;
    assert_eq!(
        head, expected,
        "fixture HEAD changed for {url}; review skipped"
    );
    let code = crate::review::run(&config, url, true).await;
    let run = latest_run(&config.paths.db());
    println!("{url}: exit {code:?}\n{}", run.describe());
    guard.assert_untouched();
    assert_eq!(
        run.head_sha, expected,
        "reviewed a different fixture revision for {url}"
    );
    run
}
