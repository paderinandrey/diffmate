use crate::error::Error;
use crate::gh::InlineComment;
use crate::store::Store;
use serde_json::Value;

pub(super) struct JobRow {
    pub publication_id: String,
    pub draft_id: i64,
    pub event: String,
    pub head_sha: String,
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

pub(super) async fn load_job(store: &Store, job_id: i64) -> Result<JobRow, Error> {
    store
        .call(move |conn| {
            let sql = "SELECT p.id, p.draft_id, p.snapshot, r.head_sha, t.owner, t.repo, t.number FROM jobs j JOIN publications p ON p.id = j.publication_id JOIN runs r ON r.id = p.run_id JOIN targets t ON t.id = p.target_id WHERE j.id = ?1 AND j.kind = 'publish'";
            conn.query_row(sql, [job_id], |row| {
                let number: i64 = row.get(6)?;
                Ok(JobRow {
                    publication_id: row.get(0)?,
                    draft_id: row.get(1)?,
                    event: event_of(&row.get::<_, String>(2)?)?,
                    head_sha: row.get(3)?,
                    owner: row.get(4)?,
                    repo: row.get(5)?,
                    number: u64::try_from(number).map_err(sql_err)?,
                })
            })
        })
        .await
}

fn event_of(snapshot: &str) -> rusqlite::Result<String> {
    let value: Value = serde_json::from_str(snapshot).map_err(json_err)?;
    let Some(event) = value.get("event").and_then(Value::as_str) else {
        return Err(text_err(std::io::Error::other(
            "publication snapshot has no event",
        )));
    };
    Ok(event.to_string())
}

pub(super) async fn comments(store: &Store, draft_id: i64) -> Result<Vec<InlineComment>, Error> {
    store
        .call(move |conn| {
            let sql = "SELECT id, title, body, path, line, side FROM findings WHERE draft_id = ?1 AND decision IN ('kept', 'edited') AND path IS NOT NULL AND line IS NOT NULL ORDER BY ordinal";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([draft_id], read_comment)?;
            rows.collect()
        })
        .await
}

fn read_comment(row: &rusqlite::Row<'_>) -> rusqlite::Result<InlineComment> {
    let line: i64 = row.get(4)?;
    Ok(InlineComment {
        path: row.get(3)?,
        line: u32::try_from(line).map_err(sql_err)?,
        side: row.get(5)?,
        body: super::body::inline(
            &row.get::<_, String>(0)?,
            &row.get::<_, String>(1)?,
            &row.get::<_, String>(2)?,
        ),
    })
}

fn json_err(error: serde_json::Error) -> rusqlite::Error {
    text_err(error)
}

fn text_err(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn sql_err(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}
