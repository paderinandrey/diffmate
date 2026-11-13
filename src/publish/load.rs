use crate::error::Error;
use crate::store::Store;
use serde_json::Value;

pub(super) struct JobRow {
    pub publication_id: String,
    pub snapshot: String,
    pub event: String,
    pub confirmed_sha: String,
    pub publication_state: String,
    pub head_sha: String,
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

pub(super) async fn load_job(store: &Store, job_id: i64) -> Result<JobRow, Error> {
    store
        .call(move |conn| {
            let sql = "SELECT p.id, p.snapshot, p.confirmed_sha, p.state, r.head_sha, t.owner, t.repo, t.number FROM jobs j JOIN publications p ON p.id = j.publication_id JOIN runs r ON r.id = p.run_id JOIN targets t ON t.id = p.target_id WHERE j.id = ?1 AND j.kind = 'publish'";
            conn.query_row(sql, [job_id], |row| {
                let snapshot: String = row.get(1)?;
                let number: i64 = row.get(7)?;
                Ok(JobRow {
                    publication_id: row.get(0)?,
                    event: event_of(&snapshot)?,
                    snapshot,
                    confirmed_sha: row.get(2)?,
                    publication_state: row.get(3)?,
                    head_sha: row.get(4)?,
                    owner: row.get(5)?,
                    repo: row.get(6)?,
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

fn json_err(error: serde_json::Error) -> rusqlite::Error {
    text_err(error)
}

fn text_err(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn sql_err(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}
