use super::put;
use crate::error::Error;
use crate::store::previous::{PreviousRun, StoredFinding};
use crate::store::{RunId, Store, TargetId};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::Path;

const NONE: &str = "no earlier successful run of this target";

fn entry(finding: &StoredFinding) -> Value {
    let thread_status = if finding.line.is_some() {
        "unknown"
    } else {
        "none"
    };
    json!({
        "id": finding.id,
        "finding": {
            "severity": finding.severity,
            "path": finding.path,
            "line": finding.line,
            "side": finding.side,
            "title": finding.title,
            "body": finding.body,
        },
        "decision": finding.decision,
        "thread_status": thread_status,
    })
}

fn document(previous: Option<&PreviousRun>) -> Value {
    let Some(previous) = previous else {
        return json!({"available": false, "reason": NONE});
    };
    json!({
        "available": true,
        "run_id": previous.run_id,
        "head_sha": previous.head_sha,
        "findings": previous.findings.iter().map(entry).collect::<Vec<_>>(),
    })
}

pub(super) async fn write(
    dir: &Path,
    store: &Store,
    (target, run): (TargetId, RunId),
) -> Result<HashSet<String>, Error> {
    let previous = store.latest_succeeded_before(target, run).await?;
    let bytes = serde_json::to_vec_pretty(&document(previous.as_ref()))?;
    put(&dir.join("previous.json"), bytes).await?;
    let findings = previous.into_iter().flat_map(|run| run.findings);
    Ok(findings.map(|finding| finding.id).collect())
}
