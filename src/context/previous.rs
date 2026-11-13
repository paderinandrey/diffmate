use super::put;
use crate::error::Error;
use crate::gh::Gh;
use crate::pr_url::PrRef;
use crate::store::previous::{PreviousRun, StoredFinding};
use crate::store::{RunId, Store, TargetId};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const NONE: &str = "no earlier successful run of this target";

fn entry(finding: &StoredFinding, known: &HashMap<String, bool>) -> Value {
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
        "thread_status": status(finding, known),
    })
}

fn status(finding: &StoredFinding, known: &HashMap<String, bool>) -> &'static str {
    if finding.line.is_none() {
        return "none";
    }
    let Some(thread_id) = finding.github_thread_id.as_deref() else {
        return "unknown";
    };
    match known.get(thread_id) {
        Some(true) => "resolved",
        Some(false) => "unresolved",
        None => "unknown",
    }
}

fn document(previous: Option<&PreviousRun>, known: &HashMap<String, bool>) -> Value {
    let Some(previous) = previous else {
        return json!({"available": false, "reason": NONE});
    };
    let findings = previous
        .findings
        .iter()
        .map(|finding| entry(finding, known));
    json!({
        "available": true,
        "run_id": previous.run_id,
        "head_sha": previous.head_sha,
        "findings": findings.collect::<Vec<_>>(),
    })
}

pub(super) async fn write<H: Gh>(
    dir: &Path,
    store: &Store,
    gh: &H,
    pr: &PrRef,
    (target, run): (TargetId, RunId),
) -> Result<HashSet<String>, Error> {
    let previous = store.latest_succeeded_before(target, run).await?;
    let known = thread_map(gh, pr, previous.as_ref()).await;
    let bytes = serde_json::to_vec_pretty(&document(previous.as_ref(), &known))?;
    put(&dir.join("previous.json"), bytes).await?;
    let findings = previous.into_iter().flat_map(|run| run.findings);
    Ok(findings.map(|finding| finding.id).collect())
}

async fn thread_map<H: Gh>(
    gh: &H,
    pr: &PrRef,
    previous: Option<&PreviousRun>,
) -> HashMap<String, bool> {
    let linked = previous.is_some_and(|run| {
        run.findings
            .iter()
            .any(|finding| finding.line.is_some() && finding.github_thread_id.is_some())
    });
    if !linked {
        return HashMap::new();
    }
    let Ok(rows) = gh.thread_resolved(pr).await else {
        return HashMap::new();
    };
    rows.into_iter()
        .map(|row| (row.thread_id, row.resolved))
        .collect()
}
