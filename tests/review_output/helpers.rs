use crate::support::fake_claude::{finding_json, output_json, success_envelope};
use crate::support::sandbox::Sandbox;
use crate::tracer::{database, review};
use std::process::Output;

pub const MOVED: &str =
    "finding 1: src/lib.rs:50 (RIGHT) is not inside a hunk of diff.patch; moved to the review body";

pub type Row = (Option<String>, Option<i64>, Option<String>, Option<String>);

pub fn finding_rows(sandbox: &Sandbox) -> Vec<Row> {
    let db = database(sandbox);
    let sql = "SELECT path, line, side, previous_finding_id FROM findings ORDER BY ordinal";
    let mut statement = db.prepare(sql).unwrap();
    let row = |r: &rusqlite::Row| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?));
    let rows = statement.query_map([], row).unwrap();
    rows.map(Result::unwrap).collect()
}

pub fn run_warnings(sandbox: &Sandbox) -> Vec<String> {
    let sql = "SELECT warnings FROM runs";
    let json: String = database(sandbox).query_row(sql, [], |r| r.get(0)).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn review_with(sandbox: &Sandbox, output: &str) -> Output {
    sandbox.claude.set_run(&success_envelope(output), 0);
    review(sandbox)
}

pub fn texts(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

pub fn anchored(line: u32, side: &str, severity: &str, title: &str) -> String {
    let path = Some("src/lib.rs");
    finding_json(path, Some(line), Some(side), severity, title, "Body.")
}

pub fn row(line: Option<i64>, side: Option<&str>) -> Row {
    (Some("src/lib.rs".into()), line, side.map(Into::into), None)
}

pub fn review_one(sandbox: &Sandbox, finding: String) -> Output {
    review_with(sandbox, &output_json(&[finding], None, &[]))
}
