use crate::store::{JobId, TargetId};
use rusqlite::{OptionalExtension, Transaction, params};

pub(super) fn find(
    tx: &Transaction,
    target: TargetId,
    force: bool,
    now: i64,
) -> rusqlite::Result<Option<JobId>> {
    let job = tx
        .query_row(
            "SELECT id FROM jobs WHERE target_id = ?1 AND kind = 'review' AND held = 0
         AND (state = 'queued' OR (state = 'running' AND (?2 = 0 OR force = 1)))
         ORDER BY (state = 'queued') DESC, id LIMIT 1",
            params![target, force],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(job) = job.filter(|_| force) {
        tx.execute(
            "UPDATE jobs SET force = 1, updated_at = ?1 WHERE id = ?2 AND state = 'queued'",
            params![now, job],
        )?;
    }
    Ok(job)
}
