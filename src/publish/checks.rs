use crate::error::Error;
use crate::store::Store;
use rusqlite::Connection;

pub(super) struct Notes {
    pub limitation: Option<String>,
    pub future: Vec<String>,
}

pub(super) struct Unanchored {
    pub path: Option<String>,
    pub title: String,
    pub body: String,
}

pub(super) async fn notes(store: &Store, draft_id: i64) -> Result<Notes, Error> {
    store.call(move |conn| read_notes(conn, draft_id)).await
}

pub(super) async fn unanchored(store: &Store, draft_id: i64) -> Result<Vec<Unanchored>, Error> {
    store
        .call(move |conn| read_unanchored(conn, draft_id))
        .await
}

fn read_notes(conn: &Connection, draft_id: i64) -> rusqlite::Result<Notes> {
    let sql =
        "SELECT limitation, limitation_included, future_improvements FROM drafts WHERE id = ?1";
    conn.query_row(sql, [draft_id], |row| {
        let text: Option<String> = row.get(0)?;
        let included: i64 = row.get(1)?;
        let future: String = row.get(2)?;
        Ok(Notes {
            limitation: (included == 1).then_some(text).flatten(),
            future: serde_json::from_str(&future).map_err(json_err)?,
        })
    })
}

fn read_unanchored(conn: &Connection, draft_id: i64) -> rusqlite::Result<Vec<Unanchored>> {
    let sql = "SELECT path, title, body FROM findings WHERE draft_id = ?1 AND decision IN ('kept', 'edited') AND (path IS NULL OR line IS NULL) ORDER BY ordinal";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([draft_id], |row| {
        Ok(Unanchored {
            path: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
        })
    })?;
    rows.collect()
}

fn json_err(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}
