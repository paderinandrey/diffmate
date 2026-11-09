use super::KnownTarget;
use crate::pr_url::PrRef;
use rusqlite::Connection;

pub(super) fn known(conn: &mut Connection) -> rusqlite::Result<Vec<KnownTarget>> {
    let mut statement = conn.prepare(
        "SELECT owner, repo, number, tracking, request_seen, queued_head
         FROM targets ORDER BY id",
    )?;
    let rows = statement.query_map([], |row| {
        let number: i64 = row.get(2)?;
        Ok(KnownTarget {
            pr: PrRef {
                owner: row.get(0)?,
                repo: row.get(1)?,
                number: u64::try_from(number)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, number))?,
            },
            tracking: row.get(3)?,
            request_seen: row.get::<_, i64>(4)? == 1,
            queued_head: row.get(5)?,
        })
    })?;
    rows.collect()
}
