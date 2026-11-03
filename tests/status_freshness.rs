#[path = "status_freshness/failure.rs"]
mod failure;
mod support;
#[allow(dead_code)]
mod tracer;

use support::fake_gh::pr_view_json;
use support::sandbox::Sandbox;
use tracer::{URL, database, review, with_pr};

fn status(sandbox: &Sandbox) -> String {
    let calls = sandbox.gh.calls().len();
    let output = sandbox.run(&["status"]);
    assert!(output.status.success());
    let after = sandbox.gh.calls();
    assert_eq!(after.len(), calls + 1);
    assert_eq!(&after[calls][..3], &["pr", "view", URL]);
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn status_shows_an_unmoved_head_as_current() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    assert!(review(&sandbox).status.success());
    assert!(status(&sandbox).ends_with(&format!(
        "  current: head {}, checked just now\n",
        &head[..12]
    )));
}

#[test]
fn status_shows_a_moved_head_as_stale() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    assert!(review(&sandbox).status.success());
    let moved = "9f3c2a1b4e5d0000000000000000000000000000";
    let json = pr_view_json("o", "r", 1, moved, "main", false);
    sandbox.gh.replace_route(&format!("pr view {URL}"), &json);
    assert!(status(&sandbox).ends_with("  stale: head 9f3c2a1b4e5d, checked just now\n"));
    let db = database(&sandbox);
    let stored: String = db
        .query_row("SELECT head_sha FROM runs", [], |r| r.get(0))
        .unwrap();
    let known: Option<String> = db
        .query_row("SELECT head_sha FROM targets", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, head);
    assert_eq!(known.as_deref(), Some(head.as_str()));
}

#[test]
fn a_moved_base_branch_keeps_the_draft_current() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    assert!(review(&sandbox).status.success());
    let upstream = sandbox.upstream("o", "r");
    upstream.commit(&[("base-only.txt", "base change")], "advance base");
    upstream.push_branch("main");
    assert!(status(&sandbox).ends_with(&format!(
        "  current: head {}, checked just now\n",
        &head[..12]
    )));
}
