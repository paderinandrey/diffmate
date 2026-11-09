use super::tracking;
use super::{Absent, SeenRequest};
use crate::pr_url::PrRef;
use crate::store::TargetId;
use rusqlite::Transaction;

mod rows;
mod write;
use rows::{consider, tracked};

struct When {
    now: i64,
    debounce: i64,
    pid: u32,
    started: i64,
}

pub(super) fn apply(
    tx: &Transaction,
    seen: &[SeenRequest],
    absent: &[(PrRef, Absent)],
    now: i64,
    debounce: i64,
    pid: u32,
    started: i64,
) -> rusqlite::Result<()> {
    let when = When {
        now,
        debounce,
        pid,
        started,
    };
    for row in seen {
        if let Some(id) = tracking::target_id(tx, &row.pr)? {
            arm(tx, id, row.is_draft, row.head_sha.as_deref(), &when)?;
        }
    }
    for (pr, kind) in absent {
        if let Absent::Open { is_draft, head_sha } = kind
            && let Some(id) = tracking::target_id(tx, pr)?
        {
            arm(tx, id, *is_draft, head_sha.as_deref(), &when)?;
        }
    }
    Ok(())
}

pub(super) fn after_run(
    tx: &Transaction,
    target: TargetId,
    now: i64,
    debounce: i64,
    pid: u32,
    started: i64,
) -> rusqlite::Result<()> {
    let head: Option<String> = tx.query_row(
        "SELECT head_sha FROM targets WHERE id = ?1",
        [target],
        |row| row.get(0),
    )?;
    let Some(head) = head else {
        return Ok(());
    };
    consider(
        tx,
        target,
        &head,
        &When {
            now,
            debounce,
            pid,
            started,
        },
    )
}

fn arm(
    tx: &Transaction,
    target: TargetId,
    draft: bool,
    head: Option<&str>,
    when: &When,
) -> rusqlite::Result<()> {
    let Some(head) = head else {
        return Ok(());
    };
    if draft || !tracked(tx, target)? {
        return Ok(());
    }
    consider(tx, target, head, when)
}
