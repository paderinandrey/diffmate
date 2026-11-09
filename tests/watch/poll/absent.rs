use super::super::{URL, count, database, with_pr};
use super::sync::{self, Running};
use super::{config, search_hit, seed_tracked};
use crate::support::sandbox::Sandbox;

const LATER: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn an_open_absence_keeps_one_delayed_job_for_the_new_head() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 60\n",
    );
    let (_, head) = with_pr(&sandbox);
    assert_ne!(head, LATER);
    sandbox.gh.route("search prs", &search_hit(false));
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|_| sync::query(&sandbox, "SELECT request_seen FROM targets") == 1);
    let seen = sync::calls(&sandbox, "search");
    sandbox.gh.replace_route("search prs", "[]");
    sandbox
        .gh
        .replace_route(&format!("pr view {URL}"), &sync::view(LATER, "OPEN", false));
    watch.wait_until(|_| {
        sync::calls(&sandbox, "search") > seen
            && sync::query(&sandbox, "SELECT request_seen FROM targets") == 0
    });
    watch.stop();
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked'"
        ),
        1
    );
    assert_eq!(count(&db, "SELECT request_seen FROM targets"), 0);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE origin = 'automatic' AND state = 'queued' AND next_attempt_at IS NOT NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE queued_head = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'"
        ),
        1
    );
}

#[test]
fn a_closed_absence_untracks_and_inserts_no_job() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 60\n",
    );
    seed_tracked(&sandbox);
    sandbox.gh.route("search prs", "[]");
    sandbox
        .gh
        .route("pr view", &super::sync::view(LATER, "MERGED", false));
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|_| {
        sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'untracked'",
        ) == 1
    });
    watch.stop();
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT request_seen FROM targets"), 0);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 0);
}
