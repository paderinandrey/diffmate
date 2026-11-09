use super::When;
use crate::store::{JobId, TargetId};
use rusqlite::{Transaction, params};

pub(super) fn note(
    tx: &Transaction,
    target: TargetId,
    head: Option<&str>,
    now: i64,
) -> rusqlite::Result<()> {
    let Some(head) = head else {
        return Ok(());
    };
    tx.execute(
        "UPDATE targets SET head_sha = ?1, head_checked_at = ?2 WHERE id = ?3",
        params![head, now, target],
    )
    .map(drop)
}

pub(super) fn reset(
    tx: &Transaction,
    job: JobId,
    target: TargetId,
    head: &str,
    when: &When,
) -> rusqlite::Result<()> {
    let at = when.now.saturating_add(when.debounce);
    tx.execute(
        "UPDATE jobs SET retry_count = 0, next_attempt_at = ?1, updated_at = ?2, head_sha = ?3
         WHERE id = ?4",
        params![at, when.now, head, job],
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
         created_at, updated_at, origin, force, next_attempt_at, retry_count, head_sha)
         VALUES ('review', ?1, 'queued', ?2, ?3, ?4, ?4, 'automatic', 0, ?5, 0, ?6)",
        params![target, when.pid, when.started, when.now, at, head],
    )?;
    tx.execute(
        "UPDATE targets SET queued_head = ?1 WHERE id = ?2",
        params![head, target],
    )
    .map(drop)
}

pub(super) fn cancel(tx: &Transaction, job: JobId, when: &When) -> rusqlite::Result<()> {
    tx.execute(
        "UPDATE jobs SET state = 'cancelled', updated_at = ?1 WHERE id = ?2",
        params![when.now, job],
    )
    .map(drop)
}

pub(super) fn hold(tx: &Transaction, target: TargetId, when: &When) -> rusqlite::Result<()> {
    tx.execute(
        "UPDATE jobs SET held = 1, updated_at = ?1 WHERE target_id = ?2 AND kind = 'review'
         AND origin = 'automatic' AND state = 'queued'",
        params![when.now, target],
    )
    .map(drop)
}

pub(super) fn release(tx: &Transaction, target: TargetId, when: &When) -> rusqlite::Result<()> {
    tx.execute(
        "UPDATE jobs SET held = 0, updated_at = ?1 WHERE target_id = ?2 AND kind = 'review'
         AND origin = 'automatic' AND state = 'queued' AND held = 1",
        params![when.now, target],
    )
    .map(drop)
}
