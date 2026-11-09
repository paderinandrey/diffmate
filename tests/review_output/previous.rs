use super::helpers::anchored;
use crate::support::fake_claude::{output_json, success_envelope};
use crate::support::fake_gh::pr_view_json;
use crate::support::sandbox::Sandbox;
use crate::tracer::{URL, count, database, with_pr};

#[test]
fn previous_id_from_another_target_is_nulled() {
    check(2, "1.1");
}

#[test]
fn unknown_id_on_a_forced_rerun_is_nulled() {
    check(1, "1.9");
}

fn check(first_number: u64, previous: &str) {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let first_url = format!("https://github.com/o/r/pull/{first_number}");
    if first_number != 1 {
        let upstream = sandbox.upstream("o", "r");
        let head = upstream.pr_head(
            first_number,
            "main",
            &[("src/lib.rs", crate::tracer::HEAD_LIB)],
        );
        let meta = pr_view_json("o", "r", first_number, &head, "main", false);
        sandbox.gh.route(&format!("pr view {first_url}"), &meta);
    }
    let finding = anchored(3, "RIGHT", "low", "First");
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[finding], None, &[])), 0);
    let first = sandbox.run(&["review", &first_url]);
    assert!(first.status.success(), "{first:?}");
    let linked = anchored(3, "RIGHT", "high", "Again").replace(
        "\"previous_finding_id\":null",
        &format!("\"previous_finding_id\":\"{previous}\""),
    );
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[linked], None, &[])), 0);
    let second = sandbox.run(&["review", URL, "--force"]);
    assert!(second.status.success(), "{second:?}");
    let warning = format!(
        "finding 1: previous_finding_id {previous} does not refer to a finding in previous.json; treated as new"
    );
    assert!(String::from_utf8_lossy(&second.stderr).contains(&format!("warning: {warning}")));
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM findings WHERE id = '2.1' AND previous_finding_id IS NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE id = 2 AND state = 'succeeded'"
        ),
        1
    );
    let warnings: String = db
        .query_row("SELECT warnings FROM runs WHERE id = 2", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Vec<String>>(&warnings).unwrap(),
        [warning]
    );
}
