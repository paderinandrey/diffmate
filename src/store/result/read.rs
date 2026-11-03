use super::{RunId, StoredRun};
use crate::agent::{FailureDetail, FailureKind};
use crate::store::records::FindingRecord;
use rusqlite::{Connection, Row, types::Type};
use serde::de::DeserializeOwned;

fn json<T: DeserializeOwned>(value: &str, column: usize) -> rusqlite::Result<T> {
    serde_json::from_str(value)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(e)))
}

fn enum_value<T: DeserializeOwned>(row: &Row, column: usize) -> rusqlite::Result<T> {
    let text: String = row.get(column)?;
    serde_json::from_value(serde_json::Value::String(text))
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(e)))
}

fn findings(conn: &Connection, run: RunId) -> rusqlite::Result<Vec<FindingRecord>> {
    let mut statement = conn.prepare(
        "SELECT severity, path, line, side, title, body, previous_finding_id
         FROM findings WHERE run_id = ?1 ORDER BY ordinal",
    )?;
    statement
        .query_map([run], |row| {
            let side: Option<String> = row.get(3)?;
            Ok(FindingRecord {
                severity: enum_value(row, 0)?,
                path: row.get(1)?,
                line: row.get(2)?,
                side: side.map(|_| enum_value(row, 3)).transpose()?,
                title: row.get(4)?,
                body: row.get(5)?,
                previous_finding_id: row.get(6)?,
            })
        })?
        .collect()
}

pub(super) fn run(conn: &Connection, run_id: RunId) -> rusqlite::Result<Option<StoredRun>> {
    conn.query_row(
        "SELECT r.state, r.head_sha, r.warnings, r.failure_kind, r.failure_detail,
         r.interrupted_cause, d.limitation, d.future_improvements
         FROM runs r LEFT JOIN drafts d ON d.run_id = r.id WHERE r.id = ?1",
        [run_id],
        |row| {
            let state: String = row.get(0)?;
            match state.as_str() {
                "succeeded" => Ok(Some(StoredRun::Succeeded {
                    run_id,
                    head_sha: row.get(1)?,
                    findings: findings(conn, run_id)?,
                    limitation: row.get(6)?,
                    future_improvements: json(&row.get::<_, String>(7)?, 7)?,
                    warnings: json(&row.get::<_, String>(2)?, 2)?,
                })),
                "failed" => {
                    let kind: FailureKind = enum_value(row, 3)?;
                    let detail: Option<String> = row.get(4)?;
                    let cause: Option<String> = row.get(5)?;
                    let message = if let Some(detail) = detail {
                        json::<FailureDetail>(&detail, 4)?.message
                    } else {
                        cause.map_or_else(
                            || kind.as_str().to_string(),
                            |c| format!("interrupted: {c}"),
                        )
                    };
                    Ok(Some(StoredRun::Failed {
                        run_id,
                        kind,
                        message,
                    }))
                }
                _ => Ok(None),
            }
        },
    )
}
