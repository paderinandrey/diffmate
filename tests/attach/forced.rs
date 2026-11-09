use super::helpers::{Waiting, succeeded};
use super::processes::{spawn_review, spawn_review_with, with_two_prs};
use crate::support::sandbox::Sandbox;
use crate::tracer::{count, database};

#[test]
fn force_during_a_running_job_produces_a_second_run() {
    let sandbox = Sandbox::new();
    with_two_prs(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let first = spawn_review(&sandbox, 1);
    hold.wait_started();
    let second = Waiting::new(spawn_review_with(&sandbox, 1, &["--force"]));
    hold.release();
    succeeded(&first.wait_with_output().unwrap());
    let second = second.finish();
    succeeded(&second);
    assert!(!String::from_utf8_lossy(&second.stderr).contains("attached to"));
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 2);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 2);
    assert_eq!(count(&db, "SELECT force FROM jobs WHERE id = 2"), 1);
    let head: String = db
        .query_row("SELECT head_sha FROM runs WHERE id = 2", [], |r| r.get(0))
        .unwrap();
    assert!(
        String::from_utf8_lossy(&second.stdout).starts_with(&format!("o/r#1 at {}:", &head[..12]))
    );
}

#[test]
fn a_plain_review_attaches_to_the_queued_forced_job() {
    let sandbox = Sandbox::new();
    with_two_prs(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let first = spawn_review(&sandbox, 1);
    hold.wait_started();
    let second = Waiting::new(spawn_review_with(&sandbox, 1, &["--force"]));
    let third = Waiting::new(spawn_review(&sandbox, 1));
    hold.release();
    succeeded(&first.wait_with_output().unwrap());
    let second = second.finish();
    let third = third.finish();
    succeeded(&second);
    succeeded(&third);
    assert!(String::from_utf8_lossy(&third.stderr).contains("attached to job 2"));
    assert_eq!(second.stdout, third.stdout);
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 2);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 2);
    assert_eq!(sandbox.claude.run_calls().len(), 2);
}
