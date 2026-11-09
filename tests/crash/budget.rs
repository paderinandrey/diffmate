use super::*;
use rusqlite::Connection;
use std::io::BufReader;
use watch_helpers::{debug_enqueue, read_until, spawn_watch};

#[test]
fn crash_every_time_exhausts_the_budget_across_watch_restarts() {
    let sandbox = Sandbox::new();
    concurrency::with_two_prs(&sandbox);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    for retry in 0..=2 {
        let (mut child, mut reader) =
            process::spawn_at(sandbox.diffmate().arg("watch"), "agent-running");
        if retry > 0 {
            let budget = format!("retry {retry} of 2");
            read_until(&mut reader, &format!("job 1 requeued: {budget}"));
            let status = sandbox.run(&["status"]);
            assert!(status.status.success(), "{status:?}");
            let text = String::from_utf8_lossy(&status.stdout);
            assert!(
                text.lines()
                    .any(|line| line.starts_with("o/r#1 ") && line.contains(&budget)),
                "{text}"
            );
            Connection::open(sandbox.db_path())
                .unwrap()
                .execute("UPDATE jobs SET next_attempt_at = 0 WHERE id = 1", [])
                .unwrap();
        }
        read_until(&mut reader, "crash point reached: agent-running");
        process::kill(&mut child);
    }
    let mut last = spawn_watch(&sandbox);
    let mut reader = BufReader::new(last.stderr.take().unwrap());
    let exhausted = "no retries left (2 of 2 used)";
    read_until(
        &mut reader,
        &format!("job 1 failed (interrupted: crash; {exhausted})"),
    );
    assert!(
        debug_enqueue(&sandbox, &concurrency::url(2), &[])
            .status
            .success()
    );
    read_until(&mut reader, "job 2 review o/r#2 succeeded");
    process::stop(&mut last);
    let db = database(&sandbox);
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE target_id = 1"),
        3
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE job_id = 1 AND state = 'failed' AND failure_kind = 'interrupted' AND interrupted_cause = 'crash'"
        ),
        3
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'failed' AND retry_count = 2 AND max_retries = 2 AND next_attempt_at IS NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM (SELECT DISTINCT head_sha, merge_base_sha, base_tip_sha FROM runs WHERE job_id = 1)"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE job_id = 2 AND state = 'succeeded'"
        ),
        1
    );
    let status = sandbox.run(&["status"]);
    let text = String::from_utf8_lossy(&status.stdout);
    assert!(status.status.success(), "{status:?}");
    assert!(
        text.lines()
            .any(|line| line.starts_with("o/r#1 ") && line.contains(exhausted)),
        "{text}"
    );
    assert_clean(&sandbox);
}
