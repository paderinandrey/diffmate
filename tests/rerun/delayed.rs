use super::*;
use std::io::BufReader;

pub(super) fn scheduled(sandbox: &Sandbox) -> rusqlite::Connection {
    processes::with_two_prs(sandbox);
    assert!(
        debug_enqueue(sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    assert!(
        debug_enqueue(sandbox, &processes::url(2), &[])
            .status
            .success()
    );
    let db = rusqlite::Connection::open(sandbox.db_path()).unwrap();
    db.execute_batch("UPDATE jobs SET retry_count = 1, max_retries = 2, next_attempt_at = 4102444800000 WHERE id = 1;").unwrap();
    db
}

#[test]
fn an_attached_retry_waits_until_due_and_runs_only_its_job() {
    let sandbox = Sandbox::new();
    let db = scheduled(&sandbox);
    let mut child = processes::spawn_review(&sandbox, 1);
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    helpers::read_until(&mut reader, "waiting: job 1 is not due");
    assert!(child.try_wait().unwrap().is_none());
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 0);
    assert!(sandbox.claude.calls().is_empty());
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND origin = 'automatic' AND retry_count = 1 AND max_retries = 2"
        ),
        1
    );
    db.execute_batch("UPDATE jobs SET next_attempt_at = 0 WHERE id = 1")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'succeeded' AND retry_count = 1 AND max_retries = 2"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 2 AND state = 'queued'"
        ),
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    assert_eq!(sandbox.claude.run_calls().len(), 1);
}
