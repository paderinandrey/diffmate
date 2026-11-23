use super::Store;
use crate::error::Error;
use crate::store::TargetId;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(PartialEq, Eq)]
pub(crate) struct DraftFinding {
    pub id: String,
    pub path: Option<String>,
    pub line: Option<i64>,
    pub side: Option<String>,
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
    pub freshness: String,
}

impl DraftFinding {
    pub(crate) fn anchor(&self) -> std::borrow::Cow<'_, str> {
        match (&self.path, self.line) {
            (Some(path), Some(line)) => format!("{path}:{line}").into(),
            (Some(path), None) => path.as_str().into(),
            (None, _) => "(no file)".into(),
        }
    }
}

fn read_finding(row: &rusqlite::Row<'_>) -> rusqlite::Result<DraftFinding> {
    Ok(DraftFinding {
        id: row.get(7)?,
        path: row.get(2)?,
        line: row.get(3)?,
        side: row.get(9)?,
        severity: row.get(1)?,
        title: row.get(4)?,
        body: row.get(5)?,
        decision: row.get(6)?,
        hint: row.get(8)?,
    })
}

pub(crate) fn findings(conn: &Connection, draft: i64) -> rusqlite::Result<Vec<DraftFinding>> {
    let mut stmt = conn.prepare("SELECT ordinal, severity, path, line, title, body, decision, id,
                        (SELECT p.decision FROM findings p WHERE p.id = findings.previous_finding_id), side
                 FROM findings WHERE draft_id = ?1 ORDER BY (line IS NULL OR path IS NULL), ordinal")?;
    stmt.query_map([draft], read_finding)?.collect()
}

impl DraftView {
    pub(crate) fn same_content(&self, other: &Self) -> bool {
        self.id == other.id
            && self.findings == other.findings
            && self.limitation == other.limitation
            && self.future == other.future
            && self.included == other.included
    }
}

pub(crate) fn improvements(value: &str) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })
}

impl Store {
    #[cfg(test)]
    pub(crate) async fn displayed_draft(&self, target: TargetId) -> std::sync::Arc<DraftView> {
        self.latest_draft(target).await.unwrap().unwrap().into()
    }
    pub async fn latest_draft(&self, target: TargetId) -> Result<Option<DraftView>, Error> {
        let now = crate::store::now_ms();
        self.call(move |conn| {
            let sql = "SELECT d.id, d.limitation, d.future_improvements, d.limitation_included,
                              r.head_sha, t.head_sha, t.head_checked_at
                       FROM drafts d
                       JOIN runs r ON r.id = d.run_id
                       JOIN targets t ON t.id = r.target_id
                       WHERE r.target_id = ?1 AND r.state = 'succeeded'
                       ORDER BY r.id DESC LIMIT 1";
            let Some(row) = conn
                .query_row(sql, params![target], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                    ))
                })
                .optional()?
            else {
                return Ok(None);
            };
            let (draft_id, limitation, future, included, run_head, known_head, checked): (
                i64,
                Option<String>,
                String,
                i64,
                String,
                Option<String>,
                Option<i64>,
            ) = row;
            let known = known_head.zip(checked);
            let findings = findings(conn, draft_id)?;
            Ok(Some(DraftView {
                id: draft_id,
                findings,
                limitation,
                future: improvements(&future)?,
                included: included == 1,
                freshness: crate::status::freshness::stored(
                    &run_head,
                    known.as_ref().map(|(head, at)| (head.as_str(), *at)),
                    now,
                ),
            }))
        })
        .await
    }
}
