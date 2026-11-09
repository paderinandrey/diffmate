use super::super::with_pr;
use super::head::{self, push};
use super::sync::{self, Running};
use super::{config, search_hit};
use crate::support::sandbox::Sandbox;

#[test]
fn a_head_change_during_a_run_leaves_a_follow_up_job() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 60\ndebounce_seconds = 1\n",
    );
    let (_, head) = with_pr(&sandbox);
    sandbox.gh.route("search prs", &search_hit(false));
    let hold = sandbox.claude.hold_run(1);
    let mut watch = Running::spawn(&sandbox);
    hold.wait_started();
    let moved = push(&sandbox, &head);
    head::show(&sandbox, &moved);
    hold.release();
    watch.wait_until(|err| err.contains("succeeded (no_findings)"));
    assert_eq!(
        sync::query(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded'"
        ),
        1
    );
    assert_eq!(
        sync::query(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'cancelled'"
        ),
        0
    );
    let db = rusqlite::Connection::open_with_flags(
        sandbox.db_path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    db.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
    let (due, queued) = db
        .query_row(
            "SELECT j.next_attempt_at IS NOT NULL, t.queued_head FROM jobs j
             JOIN targets t ON t.id = j.target_id WHERE j.state = 'queued'",
            [],
            |row| Ok((row.get::<_, bool>(0)?, row.get::<_, String>(1)?)),
        )
        .unwrap();
    assert!(due);
    assert_eq!(queued, moved);
    watch.stop();
}
