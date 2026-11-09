use super::When;
use super::write::{insert, reset};
use crate::store::{JobId, TargetId};
use rusqlite::{OptionalExtension, Transaction, params};

struct Live {
    id: JobId,
    running: bool,
    automatic: bool,
    next_at: Option<i64>,
}

pub(super) fn consider(
    tx: &Transaction,
    target: TargetId,
    head: &str,
    when: &When,
) -> rusqlite::Result<()> {
    if let Some(job) = live(tx, target)? {
        let blocked = job.running || job.next_at.is_none() || !job.automatic;
        if !blocked && waiting(tx, target)?.as_deref() != Some(head) {
            reset(tx, job.id, target, head, when)?;
        }
        return Ok(());
    }
    if run_exists(tx, target, head)? {
        return Ok(());
    }
    insert(tx, target, head, when)
}

pub(super) fn tracked(tx: &Transaction, id: TargetId) -> rusqlite::Result<bool> {
    tx.query_row(
        "SELECT tracking = 'tracked' FROM targets WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
}

fn live(tx: &Transaction, target: TargetId) -> rusqlite::Result<Option<Live>> {
    tx.query_row(
        "SELECT id, state = 'running', origin = 'automatic', next_attempt_at FROM jobs
         WHERE target_id = ?1 AND kind = 'review' AND state IN ('queued', 'running')
         ORDER BY state = 'running' DESC, next_attempt_at IS NULL DESC, id LIMIT 1",
        [target],
        |row| {
            Ok(Live {
                id: row.get(0)?,
                running: row.get(1)?,
                automatic: row.get(2)?,
                next_at: row.get(3)?,
            })
        },
    )
    .optional()
}

fn waiting(tx: &Transaction, target: TargetId) -> rusqlite::Result<Option<String>> {
    let queued: Option<String> = tx.query_row(
        "SELECT queued_head FROM targets WHERE id = ?1",
        [target],
        |row| row.get(0),
    )?;
    if queued.is_some() {
        return Ok(queued);
    }
    tx.query_row(
        "SELECT head_sha FROM runs WHERE target_id = ?1 ORDER BY id DESC LIMIT 1",
        [target],
        |row| row.get(0),
    )
    .optional()
}

fn run_exists(tx: &Transaction, target: TargetId, head: &str) -> rusqlite::Result<bool> {
    tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM runs WHERE target_id = ?1 AND head_sha = ?2)",
        params![target, head],
        |row| row.get(0),
    )
}
