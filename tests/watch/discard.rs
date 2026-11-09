use super::poll::sync::{self, Running};
use super::poll::{config, search_hit};
use super::*;
use std::process::Output;

fn debug_discard(sandbox: &Sandbox, url: &str) -> Output {
    sandbox
        .diffmate()
        .args(["debug", "discard", url])
        .output()
        .unwrap()
}

fn untracked_seen(sandbox: &Sandbox) -> i64 {
    count(
        &database(sandbox),
        "SELECT count(*) FROM targets WHERE tracking = 'untracked' AND request_seen = 1",
    )
}

#[test]
fn discard_marks_the_row_untracked_without_gh_config_or_the_lock() {
    let sandbox = Sandbox::new();
    let config = sandbox.home.join(".config/diffmate");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("config.toml"), "broken = [").unwrap();
    let first = debug_discard(&sandbox, URL);
    assert!(first.status.success(), "{first:?}");
    assert_eq!(String::from_utf8_lossy(&first.stdout), "discarded o/r#1\n");
    assert_eq!(untracked_seen(&sandbox), 1);
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(sandbox.state_dir().join("runner.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let second = debug_discard(&sandbox, URL);
    assert!(second.status.success(), "{second:?}");
    assert_eq!(String::from_utf8_lossy(&second.stdout), "discarded o/r#1\n");
    assert_eq!(untracked_seen(&sandbox), 1);
    assert_eq!(sandbox.gh.calls(), [] as [Vec<String>; 0]);
}

#[test]
fn discard_rejects_the_same_invalid_url_as_enqueue() {
    let sandbox = Sandbox::new();
    let url = "https://example.com/pull/1";
    let discard = debug_discard(&sandbox, url);
    let enqueue = debug_enqueue(&sandbox, url, &[]);
    assert_eq!(discard.status.code(), Some(2));
    assert_eq!(discard.stderr, enqueue.stderr);
    assert!(!sandbox.db_path().exists());
    assert_eq!(sandbox.gh.calls(), [] as [Vec<String>; 0]);
}

#[test]
fn debug_stays_hidden_when_discard_exists() {
    let output = Sandbox::new().run(&["--help"]);
    let help = String::from_utf8_lossy(&output.stdout);
    for command in ["review", "status", "watch"] {
        assert!(help.contains(command), "{help}");
    }
    assert!(!help.contains("debug"));
}

#[test]
fn a_still_visible_request_stays_untracked_until_it_returns() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 60\n",
    );
    with_pr(&sandbox);
    let hit = search_hit(false);
    sandbox.gh.route("search prs", &hit);
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|_| {
        sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked' AND request_seen = 1",
        ) == 1
    });
    let discarded = debug_discard(&sandbox, URL);
    assert!(discarded.status.success(), "{discarded:?}");
    let searches = sync::calls(&sandbox, "search");
    watch.wait_until(|_| sync::calls(&sandbox, "search") > searches);
    assert_eq!(untracked_seen(&sandbox), 1);
    assert_eq!(
        sync::query(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE origin = 'automatic' AND state = 'queued'",
        ),
        0
    );
    sandbox.gh.replace_route("search prs", "[]");
    watch.wait_until(|_| {
        sync::query(&sandbox, "SELECT request_seen FROM targets") == 0
            && sync::query(
                &sandbox,
                "SELECT count(*) FROM targets WHERE tracking = 'untracked'",
            ) == 1
    });
    sandbox.gh.replace_route("search prs", &hit);
    watch.wait_until(|_| {
        sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked' AND request_seen = 1",
        ) == 1
            && sync::query(
                &sandbox,
                "SELECT count(*) FROM jobs WHERE origin = 'automatic' AND state = 'queued' AND next_attempt_at IS NOT NULL",
            ) == 1
    });
    watch.stop();
}
