#[path = "concurrency/helpers.rs"]
mod helpers;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{spawn_review, with_two_prs};
use std::io::{BufRead, BufReader, Read};
use support::sandbox::Sandbox;
use tracer::{count, database};

#[test]
fn second_review_waits_for_the_lock_and_status_works() {
    let sandbox = Sandbox::new();
    with_two_prs(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let first = spawn_review(&sandbox, 1);
    hold.wait_started();

    let mut second = spawn_review(&sandbox, 2);
    let mut stderr = BufReader::new(second.stderr.take().unwrap());
    let mut line = String::new();
    stderr.read_line(&mut line).unwrap();
    assert!(line.starts_with("waiting:"), "{line}");
    let status = sandbox.run(&["status"]);

    assert!(status.status.success());
    let text = String::from_utf8(status.stdout).unwrap();
    let rows: Vec<&str> = text.lines().collect();
    assert_eq!(
        rows,
        ["o/r#1  untracked  running", "o/r#2  untracked  queued"]
    );
    assert!(second.try_wait().unwrap().is_none());
    let db = database(&sandbox);
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE target_id = 2"),
        0
    );
    let sql = "SELECT state FROM jobs WHERE target_id = 2";
    let state: String = db.query_row(sql, [], |r| r.get(0)).unwrap();
    assert_eq!(state, "queued");

    hold.release();
    let first = first.wait_with_output().unwrap();
    let mut rest = String::new();
    stderr.read_to_string(&mut rest).unwrap();
    let second = second.wait_with_output().unwrap();
    assert!(first.status.success() && second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).starts_with("o/r#2 at "));
    let sql = "SELECT count(*) FROM runs a JOIN runs b ON a.id = 1 AND b.id = 2
               WHERE a.state = 'succeeded' AND b.state = 'succeeded'
               AND b.created_at >= a.finished_at";
    assert_eq!(count(&db, sql), 1);
}
