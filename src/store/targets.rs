use super::{Store, TargetId, now_ms};
use crate::error::Error;
use crate::pr_url::PrRef;
use rusqlite::{Connection, params};

pub(super) fn clear_ref_cleanup(conn: &Connection, target: TargetId) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM pending_ref_cleanup WHERE target_id = ?1",
        [target],
    )
    .map(drop)
}

impl Store {
    pub async fn clear_ref_cleanup(&self, target: TargetId) -> Result<(), Error> {
        self.call(move |conn| clear_ref_cleanup(conn, target)).await
    }

    pub async fn pending_ref_cleanup(&self) -> Result<Vec<(TargetId, PrRef)>, Error> {
        self.call(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.owner, t.repo, t.number FROM pending_ref_cleanup p
                 JOIN targets t ON t.id = p.target_id ORDER BY t.id",
            )?;
            stmt.query_map([], |row| {
                let number: i64 = row.get(3)?;
                Ok((
                    row.get(0)?,
                    PrRef {
                        owner: row.get(1)?,
                        repo: row.get(2)?,
                        number: u64::try_from(number)
                            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, number))?,
                    },
                ))
            })?
            .collect()
        })
        .await
    }

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
