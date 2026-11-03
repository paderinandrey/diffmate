use super::helpers::{Waiting, succeeded};
use super::processes::spawn_review;
use crate::support::sandbox::Sandbox;
use crate::tracer::{review, with_pr};
use std::fs::OpenOptions;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn a_finished_job_is_printed_without_taking_the_held_lock() {
    for success in [true, false] {
        let sandbox = Sandbox::new();
        with_pr(&sandbox);
        let first = review(&sandbox);
        succeeded(&first);
        let lock = OpenOptions::new()
            .write(true)
            .open(sandbox.state_dir().join("runner.lock"))
            .unwrap();
        lock.lock().unwrap();
        let waiting = Waiting::new(spawn_review(&sandbox, 1));
        let db = rusqlite::Connection::open(sandbox.db_path()).unwrap();
        let sql = if success {
            "UPDATE jobs SET state = 'succeeded', result_run_id = 1 WHERE id = 2"
        } else {
            "UPDATE jobs SET state = 'failed', failure_reason = 'fixture failure' WHERE id = 2"
        };
        db.execute(sql, []).unwrap();
        let (send, receive) = mpsc::channel();
        let task = std::thread::spawn(move || send.send(waiting.finish()).unwrap());
        let output = receive.recv_timeout(Duration::from_secs(10));
        drop(lock);
        task.join().unwrap();
        let output = output.expect("a finished job must exit while the runner lock is held");
        if success {
            succeeded(&output);
            assert_eq!(output.stdout, first.stdout);
            assert!(String::from_utf8_lossy(&output.stderr).contains("reused run 1 for head"));
        } else {
            assert_eq!(output.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&output.stderr).contains("fixture failure"));
        }
        assert_eq!(sandbox.claude.run_calls().len(), 1);
    }
}
