use super::super::{count, database, with_pr};
use super::head::{self, push};
use super::sync::{self, Running};
use super::{config, search_hit};
use crate::support::sandbox::Sandbox;

#[test]
fn two_heads_before_the_job_starts_leave_one_later_run() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 3\n",
    );
    let (_, head) = with_pr(&sandbox);
    sandbox.gh.route("search prs", &search_hit(false));
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|_| {
        sync::query(&sandbox, "SELECT count(*) FROM jobs WHERE state = 'queued'") == 1
    });
    let moved = push(&sandbox, &head);
    head::show(&sandbox, &moved);
    watch.wait_until(|err| err.contains("succeeded (no_findings)"));
    watch.stop();
    let db = database(&sandbox);
    assert_eq!(
        count(&db, "SELECT count(*) FROM jobs WHERE kind = 'review'"),
        1
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE state = 'succeeded'"),
        1
    );
    assert_eq!(
        count(
            &db,
            &format!("SELECT count(*) FROM runs WHERE head_sha = '{moved}'")
        ),
        1
    );
}
