use super::super::super::{URL, debug_enqueue};
use super::super::config;
use super::support;
use crate::support::sandbox::Sandbox;

const OTHER: &str = "https://github.com/o/r/pull/2";

#[test]
fn review_notifies_an_automatic_give_up_found_by_recovery() {
    let sandbox = Sandbox::new();
    config(&sandbox, "max_retries = 0\n");
    let script = support::install(&sandbox);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    assert!(debug_enqueue(&sandbox, OTHER, &[]).status.success());
    let db = rusqlite::Connection::open(sandbox.db_path()).unwrap();
    db.execute_batch(
        "UPDATE jobs SET state = 'running';
         INSERT INTO runs (target_id, job_id, state, head_sha, merge_base_sha,
             base_tip_sha, base_ref, created_at)
         SELECT target_id, id, 'running', 'abc', 'abc', 'abc', 'main', 0 FROM jobs;",
    )
    .unwrap();
    drop(db);
    let output = sandbox.run(&["review", URL]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let log = support::text(&script);
    let scripts = support::scripts(&log);
    assert_eq!(scripts.len(), 1, "{stderr}");
    assert!(
        scripts[0].contains("diffmate: review failed"),
        "{scripts:?}"
    );
    assert!(scripts[0].contains("o/r#1:"), "{scripts:?}");
    assert!(scripts[0].contains("interrupted: crash"), "{scripts:?}");
    assert!(!scripts[0].contains("o/r#2"), "{scripts:?}");
    assert!(!stderr.contains("notification-failed"), "{stderr}");
    assert_eq!(
        support::count_sql(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'failed' AND origin = 'automatic'"
        ),
        1
    );
    assert_eq!(
        support::count_sql(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE id = 2 AND state = 'failed' AND origin = 'manual'"
        ),
        1
    );
}
