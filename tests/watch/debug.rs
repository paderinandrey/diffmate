use super::*;

#[test]
fn enqueue_ignores_config_and_lock_and_attaches_without_running() {
    let sandbox = Sandbox::new();
    let config = sandbox.home.join(".config/diffmate");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("config.toml"), "broken = [").unwrap();
    let queued = debug_enqueue(&sandbox, URL, &[]);
    assert!(queued.status.success(), "{queued:?}");
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(sandbox.state_dir().join("runner.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let attached = debug_enqueue(&sandbox, URL, &["--automatic", "--force"]);
    assert!(attached.status.success(), "{attached:?}");
    assert_eq!(
        String::from_utf8_lossy(&attached.stdout),
        "attached to job 1\n"
    );
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE origin = 'manual' AND force = 1 AND state = 'queued'"
        ),
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 0);
    assert!(sandbox.claude.calls().is_empty());
    assert_eq!(sandbox.gh.calls(), [] as [Vec<String>; 0]);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE tracking = 'untracked'"
        ),
        1
    );
}

#[test]
fn enqueue_rejects_invalid_urls_before_opening_the_store() {
    let sandbox = Sandbox::new();
    let output = debug_enqueue(&sandbox, "https://example.com/pull/1", &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!sandbox.db_path().exists());
}

#[test]
fn watch_rejects_invalid_configuration_before_opening_the_store() {
    let sandbox = Sandbox::new();
    let config = sandbox.home.join(".config/diffmate");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("config.toml"), "run_timeout_minutes = 0").unwrap();
    let output = sandbox.run(&["watch"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("run_timeout_minutes"));
    assert!(!sandbox.db_path().exists());
    assert_eq!(sandbox.gh.calls(), [] as [Vec<String>; 0]);
}
