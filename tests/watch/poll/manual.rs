use super::super::{URL, count, database, debug_enqueue, with_pr};
use super::sync::{self, Running};
use super::{config, search_hit};
use crate::support::fake_gh::pr_view_json;
use crate::support::sandbox::Sandbox;

#[test]
fn a_manual_review_stays_untracked_until_search_contains_it() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 60\n",
    );
    let (_, head) = with_pr(&sandbox);
    sandbox.gh.route("search prs", "[]");
    assert!(debug_enqueue(&sandbox, URL, &[]).status.success());
    assert_eq!(sync::query(&sandbox, "SELECT request_seen FROM targets"), 0);
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|err| err.contains("succeeded (no_findings)"));
    assert_eq!(
        sync::query(&sandbox, "SELECT count(*) FROM runs"),
        1,
        "only the queued manual run"
    );
    assert_eq!(
        sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'untracked' AND request_seen = 0"
        ),
        1
    );
    let upstream = sandbox.upstream("o", "r");
    upstream.git(&["branch", "feature", &head]);
    let moved = upstream.pr_head(1, "feature", &[("src/lib.rs", "fn changed() {}\n")]);
    sandbox.gh.replace_route(
        &format!("pr view {URL}"),
        &pr_view_json("o", "r", 1, &moved, "main", false),
    );
    let searches = sync::calls(&sandbox, "search");
    watch.wait_until(|_| sync::calls(&sandbox, "search") >= searches + 2);
    assert_eq!(
        sync::query(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE origin = 'automatic'"
        ),
        0
    );
    sandbox.gh.replace_route("search prs", &search_hit(false));
    watch.wait_until(|_| {
        sync::query(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE origin = 'automatic'",
        ) == 1
    });
    watch.stop();
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE origin = 'manual' AND state = 'succeeded'"
        ),
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    let (next, queued) = db
        .query_row(
            "SELECT j.next_attempt_at IS NOT NULL, t.queued_head FROM jobs j
             JOIN targets t ON t.id = j.target_id WHERE j.origin = 'automatic'",
            [],
            |row| Ok((row.get::<_, bool>(0)?, row.get::<_, String>(1)?)),
        )
        .unwrap();
    assert!(next);
    assert_eq!(queued, moved);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked' AND request_seen = 1"
        ),
        1
    );
}
