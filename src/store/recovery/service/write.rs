use crate::store::JobId;
use rusqlite::{Transaction, params};

pub(super) fn requeue(tx: &Transaction, id: JobId, now: i64) -> rusqlite::Result<()> {
    let sql = "UPDATE jobs SET state = 'queued', failure_reason = NULL, stop_reason = NULL, \
               updated_at = ?2 WHERE id = ?1";
    changed(tx, sql, params![id, now], "job is not running")
}

pub(super) fn fail(tx: &Transaction, id: JobId, reason: &str, now: i64) -> rusqlite::Result<()> {
    let sql =
        "UPDATE jobs SET state = 'failed', failure_reason = ?2, updated_at = ?3 WHERE id = ?1";
    changed(tx, sql, params![id, reason, now], "job is not running")
}

pub(super) fn succeed(tx: &Transaction, id: JobId, now: i64) -> rusqlite::Result<()> {
    let sql = "UPDATE jobs SET state = 'succeeded', updated_at = ?2 WHERE id = ?1";
    changed(tx, sql, params![id, now], "job is not running")
}

pub(super) fn queue_unknown(tx: &Transaction, now: i64) -> rusqlite::Result<()> {
    let pid = i64::from(std::process::id());
    let sql = "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, \
               created_at, updated_at, origin, publication_id) \
               SELECT 'reconcile', p.target_id, 'queued', ?1, ?2, ?2, ?2, 'manual', p.id \
               FROM publications p WHERE p.state = 'unknown' AND NOT EXISTS ( \
               SELECT 1 FROM jobs j WHERE j.publication_id = p.id AND j.kind = 'reconcile' \
               AND j.state IN ('queued', 'running'))";
    tx.execute(sql, params![pid, now])?;
    Ok(())
}

pub(super) fn changed(
    tx: &Transaction,
    sql: &str,
    values: impl rusqlite::Params,
    detail: &str,
) -> rusqlite::Result<()> {
    if tx.execute(sql, values)? == 1 {
        Ok(())
    } else {
        Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            Some(detail.into()),
        ))
    }
}
