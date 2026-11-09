#[path = "attach/finished.rs"]
mod finished;
#[path = "attach/forced.rs"]
mod forced;
#[path = "attach/helpers.rs"]
mod helpers;
#[path = "concurrency/helpers.rs"]
mod processes;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{Waiting, succeeded};
use processes::{spawn_review, with_two_prs};
use rustix::process::{Pid, Signal, kill_process};
use support::sandbox::Sandbox;
use tracer::{count, database};

#[test]
fn two_reviews_of_one_url_produce_one_run() {
    let sandbox = Sandbox::new();
    with_two_prs(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let first = spawn_review(&sandbox, 1);
    hold.wait_started();
    let second = Waiting::new(spawn_review(&sandbox, 1));
    hold.release();
    let first = first.wait_with_output().unwrap();
    let second = second.finish();
    succeeded(&first);
    succeeded(&second);
    assert!(String::from_utf8_lossy(&second.stderr).contains("attached to job 1"));
    assert_eq!(first.stdout, second.stdout);
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 1);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    assert_eq!(sandbox.claude.run_calls().len(), 1);
}

#[test]
fn stopping_a_waiting_review_leaves_its_job_queued() {
    for signal in [Signal::INT, Signal::TERM] {
        let sandbox = Sandbox::new();
        with_two_prs(&sandbox);
        let hold = sandbox.claude.hold_run(1);
        let first = spawn_review(&sandbox, 1);
        hold.wait_started();
        let second = Waiting::new(spawn_review(&sandbox, 2));
        kill_process(
            Pid::from_raw(i32::try_from(second.child.id()).unwrap()).unwrap(),
            signal,
        )
        .unwrap();
        let second = second.finish();
        hold.release();
        succeeded(&first.wait_with_output().unwrap());
        assert_eq!(second.status.code(), Some(1), "{second:?}");
        assert!(
            String::from_utf8_lossy(&second.stderr)
                .contains("o/r#2: Stopped waiting; the job stays queued")
        );
        assert_eq!(
            count(
                &database(&sandbox),
                "SELECT count(*) FROM jobs WHERE id = 2 AND state = 'queued' AND stop_reason IS NULL"
            ),
            1
        );
    }
}
