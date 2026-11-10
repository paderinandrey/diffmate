use super::Outcome;
use super::rows::{Draft, Finding};
use rusqlite::{Transaction, params};
use serde_json::{Value, json};

pub(super) fn snapshot(
    event: &str,
    draft: &Draft,
    findings: &[Finding],
) -> rusqlite::Result<String> {
    let future: Value = serde_json::from_str(&draft.future).map_err(json_err)?;
    let kept: Vec<Value> = findings
        .iter()
        .filter(|finding| matches!(finding.decision.as_str(), "kept" | "edited"))
        .map(finding_json)
        .collect();
    let limitation = if draft.included {
        draft.limitation.clone()
    } else {
        None
    };
    Ok(json!({
        "event": event,
        "findings": kept,
        "limitation": limitation,
        "future_improvements": future,
    })
    .to_string())
}

fn finding_json(finding: &Finding) -> Value {
    json!({
        "id": finding.id,
        "decision": finding.decision,
        "severity": finding.severity,
        "path": finding.path,
        "line": finding.line,
        "side": finding.side,
        "title": finding.title,
        "body": finding.body,
    })
}

fn json_err(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

pub(super) fn publication(
    tx: &Transaction,
    draft: &Draft,
    draft_id: i64,
    event: &str,
    confirmed_sha: &str,
    snapshot: &str,
    now: i64,
) -> rusqlite::Result<String> {
    let sql = "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at) VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, 'pending', ?4, ?5, ?6, ?7, ?7)";
    tx.execute(
        sql,
        params![
            draft.target_id,
            draft_id,
            draft.run_id,
            event,
            confirmed_sha,
            snapshot,
            now
        ],
    )?;
    tx.query_row(
        "SELECT id FROM publications WHERE rowid = last_insert_rowid()",
        [],
        |row| row.get(0),
    )
}

pub(super) fn job(
    tx: &Transaction,
    target_id: i64,
    pid: i64,
    started: i64,
    now: i64,
    publication_id: &str,
) -> rusqlite::Result<Outcome> {
    let sql = "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, origin, publication_id) VALUES ('publish', ?1, 'queued', ?2, ?3, ?4, ?4, 'manual', ?5)";
    tx.execute(sql, params![target_id, pid, started, now, publication_id])?;
    Ok(Outcome::Job(tx.last_insert_rowid()))
}
