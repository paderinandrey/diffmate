use crate::support::fake_claude::{finding_json, output_json, success_envelope};
use crate::support::sandbox::Sandbox;
use crate::tracer::count;
use rusqlite::Connection;

const TABLES: [&str; 5] = ["targets", "jobs", "runs", "drafts", "findings"];

pub fn one_finding(title: &str) -> String {
    let (path, side) = (Some("src/lib.rs"), Some("RIGHT"));
    let finding = finding_json(path, Some(3), side, "high", title, "b");
    success_envelope(&output_json(&[finding], None, &[]))
}

pub fn status_lines(sandbox: &Sandbox) -> Vec<String> {
    let output = sandbox.run(&["status"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    text.lines().map(String::from).collect()
}

pub fn snapshot(db: &Connection) -> Vec<i64> {
    let version = count(db, "PRAGMA user_version");
    let counts = TABLES.map(|t| count(db, &format!("SELECT count(*) FROM {t}")));
    [counts.to_vec(), vec![version]].concat()
}
