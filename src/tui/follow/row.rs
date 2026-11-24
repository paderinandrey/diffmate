use crate::error::Error;
use crate::store::Store;
use rusqlite::{OptionalExtension, params};

pub(super) struct Publication {
    pub id: String,
    pub draft: i64,
    pub state: String,
    pub reason: Option<String>,
    pub job: String,
    pub seen: Option<String>,
    pub run: String,
    pub reconcile: Option<String>,
}

pub(super) async fn load(
    store: &Store,
    target: i64,
    draft: Option<i64>,
) -> Result<Option<Publication>, Error> {
    store
        .call(move |conn| {
            let sql = "SELECT p.id, p.draft_id, p.state, p.reason, j.state, j.head_sha, r.head_sha,
                          c.state
                   FROM publications p JOIN runs r ON r.id = p.run_id
                   JOIN jobs j ON j.publication_id = p.id AND j.kind = 'publish'
                   LEFT JOIN jobs c ON c.id = (SELECT max(id) FROM jobs
                         WHERE publication_id = p.id AND kind = 'reconcile')
                   WHERE p.target_id = ?1 AND (?2 IS NULL OR p.draft_id = ?2
                          OR p.state IN ('pending','sending','unknown'))
                   ORDER BY p.state IN ('pending','sending','unknown') DESC, j.id DESC LIMIT 1";
            conn.query_row(sql, params![target, draft], |row| {
                Ok(Publication {
                    id: row.get(0)?,
                    draft: row.get(1)?,
                    state: row.get(2)?,
                    reason: row.get(3)?,
                    job: row.get(4)?,
                    seen: row.get(5)?,
                    run: row.get(6)?,
                    reconcile: row.get(7)?,
                })
            })
            .optional()
        })
        .await
}
