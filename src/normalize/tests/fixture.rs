use crate::agent::ReviewOutput;
use crate::diff::{DiffIndex, parse};
use serde_json::{Value, json};

const DIFF: &str = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,2 +1,2 @@\n a\n-b\n+c\n";

pub fn finding(
    path: Option<&str>,
    line: Option<u32>,
    side: Option<&str>,
    previous: Option<&str>,
) -> Value {
    json!({"severity": "low", "path": path, "line": line, "side": side, "title": "T", "body": "B", "previous_finding_id": previous})
}

pub fn output(findings: &[Value]) -> ReviewOutput {
    let status = if findings.is_empty() {
        "no_findings"
    } else {
        "findings"
    };
    let value = json!({"status": status, "limitation": null, "findings": findings, "future_improvements": []});
    serde_json::from_value(value).unwrap()
}

pub fn index() -> DiffIndex {
    parse(DIFF.as_bytes())
}
