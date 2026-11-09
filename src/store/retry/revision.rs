use crate::error::Error;
use crate::store::{JobId, Store, records::Revision};
use rusqlite::OptionalExtension;

impl Store {
    pub async fn retry_revision(&self, job: JobId) -> Result<Option<Revision>, Error> {
        self.call(move |conn| {
            conn.query_row(
                "SELECT r.head_sha, r.merge_base_sha, r.base_tip_sha, r.base_ref FROM runs r
             JOIN jobs j ON j.id = r.job_id WHERE j.id = ?1 AND j.origin = 'automatic'
             ORDER BY r.id LIMIT 1",
                [job],
                |row| {
                    Ok(Revision {
                        head_sha: row.get(0)?,
                        merge_base_sha: row.get(1)?,
                        base_tip_sha: row.get(2)?,
                        base_ref: row.get(3)?,
                    })
                },
            )
            .optional()
        })
        .await
    }
}
