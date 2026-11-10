use super::Store;
use crate::error::Error;
use crate::store::TargetId;
use rusqlite::{OptionalExtension, params};

pub(crate) struct DraftFinding {
    pub id: String,
    pub anchor: String,
    pub severity: String,
    pub title: String,
    pub body: String,
    pub decision: String,
    pub hint: Option<String>,
}

pub(crate) struct DraftView {
    pub id: i64,
    pub findings: Vec<DraftFinding>,
    pub limitation: Option<String>,
    pub future: Vec<String>,
    pub included: bool,
}

fn anchor(path: Option<String>, line: Option<i64>) -> String {
    match (path, line) {
        (Some(path), Some(line)) => format!("{path}:{line}"),
        (Some(path), None) => path,
        (None, _) => "(no file)".to_string(),
    }
}

fn read_finding(row: &rusqlite::Row<'_>) -> rusqlite::Result<(bool, DraftFinding)> {
    let path: Option<String> = row.get(2)?;
    let line: Option<i64> = row.get(3)?;
    let anchored = path.is_some() && line.is_some();
    Ok((
        anchored,
        DraftFinding {
            id: row.get(7)?,
            anchor: anchor(path, line),
            severity: row.get(1)?,
            title: row.get(4)?,
            body: row.get(5)?,
            decision: row.get(6)?,
            hint: row.get(8)?,
        },
    ))
}

fn improvements(value: &str) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })
}

impl Store {
    pub async fn latest_draft(&self, target: TargetId) -> Result<Option<DraftView>, Error> {
        self.call(move |conn| {
            let sql = "SELECT d.id, d.limitation, d.future_improvements, d.limitation_included
                       FROM drafts d JOIN runs r ON r.id = d.run_id
                       WHERE r.target_id = ?1 AND r.state = 'succeeded'
                       ORDER BY r.id DESC LIMIT 1";
            let Some(row) = conn
                .query_row(sql, params![target], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })
                .optional()?
            else {
                return Ok(None);
            };
            let (draft_id, limitation, future, included): (i64, Option<String>, String, i64) = row;
            let mut statement = conn.prepare(
                "SELECT ordinal, severity, path, line, title, body, decision, id,
                        (SELECT p.decision FROM findings p WHERE p.id = findings.previous_finding_id)
                 FROM findings WHERE draft_id = ?1 ORDER BY ordinal",
            )?;
            let loaded: Vec<_> = statement
                .query_map(params![draft_id], read_finding)?
                .collect::<Result<_, _>>()?;
            let (anchored, rest): (Vec<_>, Vec<_>) =
                loaded.into_iter().partition(|(anchored, _)| *anchored);
            let findings = anchored
                .into_iter()
                .chain(rest)
                .map(|(_, finding)| finding)
                .collect();
            Ok(Some(DraftView {
                id: draft_id,
                findings,
                limitation,
                future: improvements(&future)?,
                included: included == 1,
            }))
        })
        .await
    }
}
