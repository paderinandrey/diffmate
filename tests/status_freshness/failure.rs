use super::{Sandbox, URL, pr_view_json, review, status, with_pr};

#[test]
fn status_reports_a_failed_live_check() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    assert!(review(&sandbox).status.success());
    rusqlite::Connection::open(sandbox.db_path())
        .unwrap()
        .execute(
            "UPDATE targets SET head_sha = NULL, head_checked_at = NULL",
            [],
        )
        .unwrap();
    sandbox.gh.replace_route_fail(
        &format!("pr view {URL}"),
        "off\u{1b}\u{202e}line\ntry\tagain",
        1,
    );
    let output = sandbox.run(&["status"]);
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .ends_with("  freshness unknown: head never checked (live check failed)\n")
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("warning: live head check for o/r#1 failed")
    );
    let warning = String::from_utf8_lossy(&output.stderr);
    assert_eq!(warning.lines().count(), 1);
    assert!(!warning.contains(['\u{1b}', '\u{202e}', '\t']));
    let json = pr_view_json("o", "r", 1, &head, "main", false);
    sandbox.gh.replace_route(&format!("pr view {URL}"), &json);
    assert!(status(&sandbox).contains("current: head"));
}

#[test]
fn status_falls_back_to_the_stored_head() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    assert!(review(&sandbox).status.success());
    sandbox
        .gh
        .replace_route_fail(&format!("pr view {URL}"), "offline", 1);
    assert!(status(&sandbox).ends_with(&format!(
        "  current: head {}, checked just now (live check failed)\n",
        &head[..12]
    )));
}
