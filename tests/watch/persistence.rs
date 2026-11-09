use super::*;

#[test]
fn a_failed_signal_write_keeps_the_owned_agent_until_the_retry() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    let db = rusqlite::Connection::open(sandbox.db_path()).unwrap();
    db.execute_batch("CREATE TRIGGER reject_signal BEFORE UPDATE OF stop_reason ON jobs WHEN NEW.stop_reason IS NOT NULL BEGIN SELECT RAISE(ABORT, 'injected signal write failure'); END;").unwrap();
    let hold = sandbox.claude.hold_run(1);
    let mut child = spawn_watch(&sandbox);
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    hold.wait_started();
    let agent = sandbox.claude.run_calls()[0].pid.unwrap();
    kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    let first = read_until(&mut reader, "storing stop reason failed");
    assert!(process::running(agent));
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE state = 'running' AND stop_reason IS NULL"
        ),
        1
    );
    kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    db.execute_batch("DROP TRIGGER reject_signal").unwrap();
    assert!(child.wait().unwrap().success());
    let mut rest = String::new();
    reader.read_to_string(&mut rest).unwrap();
    let text = first + &rest;
    assert_eq!(text.matches("storing stop reason failed").count(), 1);
    assert!(
        text.contains("failed (interrupted: shutdown), requeued"),
        "{text}"
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE state = 'queued' AND stop_reason IS NULL AND retry_count = 0"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE interrupted_cause = 'shutdown'"
        ),
        1
    );
    process::wait_until_gone(agent);
}
