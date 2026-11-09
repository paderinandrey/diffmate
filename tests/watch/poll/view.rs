use super::super::{count, database};
use super::sync::Running;
use super::{config, search_hit, seed_tracked};
use crate::support::sandbox::Sandbox;

const HEAD: &str = "viewed-head";

#[test]
fn a_ready_search_hit_whose_view_is_draft_records_the_head_and_queues_nothing() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 60\n",
    );
    seed_tracked(&sandbox);
    sandbox.gh.route("search prs", &search_hit(false));
    sandbox
        .gh
        .route("pr view", &super::sync::view(HEAD, "OPEN", true));
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|_| noted(&sandbox) == 1);
    watch.stop();
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 0);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked' AND request_seen = 1"
        ),
        1
    );
}

#[test]
fn a_ready_search_hit_whose_view_is_closed_untracks_and_records_the_head() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 60\n",
    );
    seed_tracked(&sandbox);
    sandbox.gh.route("search prs", &search_hit(false));
    sandbox
        .gh
        .route("pr view", &super::sync::view(HEAD, "CLOSED", false));
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|_| {
        super::sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'untracked'",
        ) == 1
    });
    watch.stop();
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT request_seen FROM targets"), 0);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 0);
    assert_eq!(noted(&sandbox), 1);
}

fn noted(sandbox: &Sandbox) -> i64 {
    super::sync::query(
        sandbox,
        "SELECT count(*) FROM targets WHERE head_sha = 'viewed-head' AND head_checked_at IS NOT NULL",
    )
}
