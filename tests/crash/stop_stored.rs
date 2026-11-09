use super::*;
use rustix::process::Signal;
use std::io::BufReader;
use watch_helpers::{debug_enqueue, read_until, spawn_watch};

#[test]
fn shutdown_cause_survives_a_crash_and_requeues() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    process::crash_after_stop(&sandbox, sandbox.diffmate().arg("watch"), Signal::TERM);
    let mut resumed = spawn_watch(&sandbox);
    let mut reader = BufReader::new(resumed.stderr.take().unwrap());
    let text = read_until(&mut reader, "job 1 review o/r#1 succeeded");
    assert!(
        text.contains("run 1 marked failed (interrupted: shutdown)"),
        "{text}"
    );
    assert!(
        text.contains("job 1 requeued (interrupted: shutdown)"),
        "{text}"
    );
    process::stop(&mut resumed);
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 1);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'succeeded' AND retry_count = 0"
        ),
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM runs WHERE job_id = 1"), 2);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE job_id = 1 AND state = 'failed' AND failure_kind = 'interrupted' AND interrupted_cause = 'shutdown'"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE job_id = 1 AND state = 'succeeded'"
        ),
        1
    );
    assert_clean(&sandbox);
}

#[test]
fn user_cancel_cause_survives_a_crash_and_is_not_retried() {
    let sandbox = Sandbox::new();
    concurrency::with_two_prs(&sandbox);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    process::crash_after_stop(
        &sandbox,
        sandbox.diffmate().args(["review", URL]),
        Signal::INT,
    );
    let mut resumed = spawn_watch(&sandbox);
    let mut reader = BufReader::new(resumed.stderr.take().unwrap());
    let text = read_until(&mut reader, "job 1 failed (interrupted: user_cancel)");
    assert!(
        text.contains("run 1 marked failed (interrupted: user_cancel)"),
        "{text}"
    );
    assert!(
        debug_enqueue(&sandbox, &concurrency::url(2), &[])
            .status
            .success()
    );
    read_until(&mut reader, "job 2 review o/r#2 succeeded");
    process::stop(&mut resumed);
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'failed' AND retry_count = 0 AND next_attempt_at IS NULL"
        ),
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM runs WHERE job_id = 1"), 1);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE job_id = 1 AND state = 'failed' AND failure_kind = 'interrupted' AND interrupted_cause = 'user_cancel'"
        ),
        1
    );
    assert_clean(&sandbox);
}
