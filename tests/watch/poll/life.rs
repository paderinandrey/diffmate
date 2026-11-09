use super::super::URL;
use super::head::push;
use super::sync::Running;
use super::{config, search_hit};
use crate::support::sandbox::Sandbox;

fn watch_one(sandbox: &Sandbox, state: &str, draft: bool) -> String {
    config(
        sandbox,
        "poll_interval_seconds = 60\ndebounce_seconds = 1\n",
    );
    let (_, head) = super::super::with_pr(sandbox);
    sandbox.gh.route("search prs", &search_hit(false));
    let hold = sandbox.claude.hold_run(1);
    let mut watch = Running::spawn(sandbox);
    hold.wait_started();
    let moved = push(sandbox, &head);
    sandbox.gh.replace_route(
        &format!("pr view {URL}"),
        &super::sync::view(&moved, state, draft),
    );
    hold.release();
    watch.wait_until(|err| err.contains("succeeded (no_findings)"));
    watch.stop();
    moved
}

#[test]
fn a_draft_seen_at_the_end_of_a_run_holds_the_follow_up() {
    let sandbox = Sandbox::new();
    let moved = watch_one(&sandbox, "OPEN", true);
    assert_eq!(
        super::sync::query(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'queued' AND held = 1 AND origin = 'automatic'"
        ),
        1
    );
    assert_eq!(
        super::sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked'"
        ),
        1
    );
    assert_eq!(
        super::sync::query(
            &sandbox,
            &format!("SELECT count(*) FROM targets WHERE head_sha = '{moved}'")
        ),
        1
    );
}

#[test]
fn a_close_seen_at_the_end_of_a_run_untracks_and_queues_nothing() {
    let sandbox = Sandbox::new();
    let moved = watch_one(&sandbox, "CLOSED", false);
    assert_eq!(
        super::sync::query(&sandbox, "SELECT count(*) FROM jobs WHERE state = 'queued'"),
        0
    );
    assert_eq!(
        super::sync::query(
            &sandbox,
            "SELECT count(*) FROM targets WHERE tracking = 'untracked' AND request_seen = 0"
        ),
        1
    );
    assert_eq!(
        super::sync::query(
            &sandbox,
            &format!("SELECT count(*) FROM targets WHERE head_sha = '{moved}'")
        ),
        1
    );
}
