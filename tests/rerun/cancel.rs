use super::*;
use rustix::process::{Pid, Signal, kill_process};
use std::io::BufReader;

#[test]
fn stopping_a_delayed_attachment_preserves_origin_and_budget() {
    for signal in [Signal::INT, Signal::TERM] {
        let sandbox = Sandbox::new();
        let db = delayed::scheduled(&sandbox);
        let mut child = processes::spawn_review(&sandbox, 1);
        let mut reader = BufReader::new(child.stderr.take().unwrap());
        helpers::read_until(&mut reader, "waiting: job 1 is not due");
        kill_process(
            Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap(),
            signal,
        )
        .unwrap();
        assert_eq!(child.wait().unwrap().code(), Some(1));
        let stopped = helpers::read_until(&mut reader, "Stopped waiting; the job stays queued");
        assert!(!stopped.contains("watch will run it"));
        assert_eq!(
            count(
                &db,
                "SELECT count(*) FROM jobs WHERE id = 1 AND origin = 'automatic' AND state = 'queued' AND retry_count = 1 AND max_retries = 2 AND next_attempt_at = 4102444800000 AND stop_reason IS NULL"
            ),
            1
        );
        assert_eq!(count(&db, "SELECT count(*) FROM runs"), 0);
        assert!(sandbox.claude.calls().is_empty());
    }
}

#[test]
fn recovery_creates_a_retry_delay_before_the_attached_review_can_run() {
    let sandbox = Sandbox::new();
    let db = delayed::scheduled(&sandbox);
    db.execute_batch(
        "UPDATE jobs SET state = 'running', next_attempt_at = NULL, retry_count = 0 WHERE id = 1",
    )
    .unwrap();
    let mut child = processes::spawn_review(&sandbox, 1);
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    helpers::read_until(&mut reader, "waiting: job 1 is not due");
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 0);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'queued' AND retry_count = 1 AND next_attempt_at IS NOT NULL"
        ),
        1
    );
    kill_process(
        Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap(),
        Signal::INT,
    )
    .unwrap();
    assert_eq!(child.wait().unwrap().code(), Some(1));
}
