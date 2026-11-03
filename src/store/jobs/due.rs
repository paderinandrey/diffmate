use crate::error::Error;
use crate::store::{JobId, Store};
use rusqlite::params;

impl Store {
    pub async fn job_delayed(&self, job: JobId, now: i64) -> Result<bool, Error> {
        self.call(move |conn| {
            conn.query_row(
                "SELECT state = 'queued' AND next_attempt_at IS NOT NULL AND next_attempt_at > ?2 FROM jobs WHERE id = ?1",
                params![job, now], |row| row.get(0),
            )
        }).await
    }
}
