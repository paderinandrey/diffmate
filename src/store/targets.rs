use super::{Store, TargetId, now_ms};
use crate::error::Error;
use rusqlite::params;

impl Store {
    pub async fn set_known_head(&self, target: TargetId, head: &str) -> Result<(), Error> {
        let head = head.to_string();
        self.call(move |conn| {
            conn.execute(
                "UPDATE targets SET head_sha = ?1, head_checked_at = ?2 WHERE id = ?3",
                params![head, now_ms(), target],
            )
            .map(drop)
        })
        .await
    }
}
