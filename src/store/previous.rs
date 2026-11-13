use super::{RunId, Store, TargetId};
use crate::error::Error;
use rusqlite::{OptionalExtension, params};
use serde_json::Value;

pub(crate) struct StoredFinding {
    pub id: String,
    pub severity: String,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub side: Option<String>,
    pub title: String,
    pub body: String,
    pub decision: String,
    pub github_thread_id: Option<String>,
}

pub(crate) struct PreviousRun {
    pub run_id: RunId,
    pub head_sha: String,
    pub findings: Vec<StoredFinding>,
}

impl Store {
    pub async fn latest_succeeded_before(
        &self,
        target: TargetId,
        run: RunId,
    ) -> Result<Option<PreviousRun>, Error> {
        self.call(move |conn| {
            let sql = "SELECT id, head_sha FROM runs
                       WHERE target_id = ?1 AND state = 'succeeded' AND id < ?2
                       ORDER BY id DESC LIMIT 1";
            let row = |r: &rusqlite::Row| Ok((r.get::<_, RunId>(0)?, r.get::<_, String>(1)?));
            let Some((run_id, head_sha)) =
                conn.query_row(sql, params![target, run], row).optional()?
            else {
                return Ok(None);
            };
            let raw: Option<String> = conn
                .query_row(
                    "SELECT original_output FROM drafts WHERE run_id = ?1",
                    [run_id],
                    |row| row.get(0),
                )
                .optional()?;
            let originals = raw.as_deref().map(original_texts).unwrap_or_default();
            let sql = "SELECT id, severity, path, line, side, title, body, decision,
                       github_thread_id, ordinal FROM findings WHERE run_id = ?1 ORDER BY ordinal";
            let mut statement = conn.prepare(sql)?;
            let stored = |row: &rusqlite::Row| {
                let ordinal: i64 = row.get(9)?;
                let mut title: String = row.get(5)?;
                let mut body: String = row.get(6)?;
                if let Some((original_title, original_body)) = text_at(&originals, ordinal) {
                    title = original_title;
                    body = original_body;
                }
                Ok(StoredFinding {
                    id: row.get(0)?,
                    severity: row.get(1)?,
                    path: row.get(2)?,
                    line: row.get(3)?,
                    side: row.get(4)?,
                    title,
                    body,
                    decision: row.get(7)?,
                    github_thread_id: row.get(8)?,
                })
            };
            let findings = statement
                .query_map(params![run_id], stored)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(PreviousRun {
                run_id,
                head_sha,
                findings,
            }))
        })
        .await
    }
}

fn original_texts(raw: &str) -> Vec<(String, String)> {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    let Some(items) = value.get("findings").and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|item| (json_str(item, "title"), json_str(item, "body")))
        .collect()
}

fn json_str(item: &Value, name: &str) -> String {
    item.get(name)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn text_at(originals: &[(String, String)], ordinal: i64) -> Option<(String, String)> {
    let index = usize::try_from(ordinal).ok()?.checked_sub(1)?;
    let (title, body) = originals.get(index)?;
    (!title.is_empty() && !body.is_empty()).then(|| (title.clone(), body.clone()))
}
