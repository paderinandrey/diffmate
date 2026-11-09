use super::When;
use crate::store::{JobId, TargetId};
use rusqlite::{Transaction, params};

pub(super) fn reset(
    tx: &Transaction,
    job: JobId,
    target: TargetId,
    head: &str,
    when: &When,
) -> rusqlite::Result<()> {
    let at = when.now.saturating_add(when.debounce);
    tx.execute(
        "UPDATE jobs SET retry_count = 0, next_attempt_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![at, when.now, job],
    )?;
    tx.execute(
        "UPDATE targets SET queued_head = ?1 WHERE id = ?2",
        params![head, target],
    )
    .map(drop)
}

pub(super) fn insert(
    tx: &Transaction,
    target: TargetId,
    head: &str,
    when: &When,
) -> rusqlite::Result<()> {
    let at = when.now.saturating_add(when.debounce);
    tx.execute(
        "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at,
         created_at, updated_at, origin, force, next_attempt_at, retry_count)
         VALUES ('review', ?1, 'queued', ?2, ?3, ?4, ?4, 'automatic', 0, ?5, 0)",
        params![target, when.pid, when.started, when.now, at],
    )?;
    tx.execute(
        "UPDATE targets SET queued_head = ?1 WHERE id = ?2",
        params![head, target],
    )
    .map(drop)
}
