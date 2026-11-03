mod support;
#[allow(dead_code)]
mod tracer;
#[path = "attach/helpers.rs"]
mod waiting;

use std::process::Stdio;
use support::fake_gh::pr_view_json;
use support::sandbox::Sandbox;
use tracer::{URL, database, with_pr};

#[test]
fn push_during_run_makes_the_draft_stale() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let child = sandbox
        .diffmate()
        .args(["review", URL])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    hold.wait_started();
    let attached = waiting::Waiting::new(
        sandbox
            .diffmate()
            .args(["review", URL])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let upstream = sandbox.upstream("o", "r");
    upstream.git(&["branch", "feature", &head]);
    let moved = upstream.pr_head(1, "feature", &[("src/lib.rs", "fn changed() {}\n")]);
    let json = pr_view_json("o", "r", 1, &moved, "main", false);
    sandbox.gh.replace_route(&format!("pr view {URL}"), &json);
    hold.release();
    let output = child.wait_with_output().unwrap();
    waiting::succeeded(&output);
    let attached = attached.finish();
    waiting::succeeded(&attached);
    assert_eq!(output.stdout, attached.stdout);
    assert!(String::from_utf8_lossy(&attached.stderr).contains("attached to job 1"));
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        text.lines().next().unwrap(),
        format!(
            "o/r#1 at {}: succeeded (no_findings), stale: PR head is now {}",
            &head[..12],
            &moved[..12]
        )
    );
    let stored: (String, String, bool) = database(&sandbox)
        .query_row(
            "SELECT t.head_sha, r.head_sha, t.head_checked_at >= r.created_at
             FROM targets t JOIN runs r ON r.target_id = t.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(stored, (moved, head, true));
}
