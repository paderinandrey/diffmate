use super::super::with_pr;
use super::sync::{self, Running};
use super::{config, search_hit};
use crate::support::sandbox::Sandbox;

#[test]
fn a_draft_is_tracked_without_a_job_until_it_is_ready() {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 1\ndebounce_seconds = 1\n",
    );
    with_pr(&sandbox);
    sandbox.gh.route("search prs", &search_hit(true));
    let mut watch = Running::spawn(&sandbox);
    watch.wait_until(|_| {
        sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked' AND request_seen = 1",
        ) == 1
            && sync::query(&sandbox, "SELECT count(*) FROM jobs") == 0
    });
    assert_eq!(sync::calls(&sandbox, "pr"), 0);
    sandbox.gh.replace_route("search prs", &search_hit(false));
    watch.wait_until(|err| err.contains("succeeded (no_findings)"));
    watch.stop();
    assert_eq!(
        sync::query(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND origin = 'automatic'"
        ),
        1
    );
    assert_eq!(sync::query(&sandbox, "SELECT count(*) FROM runs"), 1);
}
