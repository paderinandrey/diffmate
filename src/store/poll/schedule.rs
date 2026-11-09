use super::tracking;
use super::{Absent, SeenRequest};
use crate::pr_url::PrRef;
use crate::store::TargetId;
use rusqlite::Transaction;

mod rows;
mod write;
use rows::{consider, tracked};
use write::{hold, release};

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
            write::note(tx, id, row.head_sha.as_deref(), when.now)?;
            arm(tx, id, row.is_draft, row.head_sha.as_deref(), &when)?;
        }
    }
    for (pr, kind) in absent {
        let Some(id) = tracking::target_id(tx, pr)? else {
            continue;
        };
        write::note(tx, id, observed(kind), when.now)?;
        if let Absent::Open { is_draft, head_sha } = kind {
            arm(tx, id, *is_draft, head_sha.as_deref(), &when)?;
        }
    }
    Ok(())
}

fn observed(kind: &Absent) -> Option<&str> {
    match kind {
        Absent::Skip => None,
        Absent::Closed { head_sha } | Absent::Open { head_sha, .. } => head_sha.as_deref(),
    }
}

pub(super) fn after_run(
    tx: &Transaction,
    target: TargetId,
    now: i64,
    debounce: i64,
    pid: u32,
    started: i64,
) -> rusqlite::Result<()> {
    if !tracked(tx, target)? {
        return Ok(());
    }
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
    if draft {
        return hold(tx, target, when);
    }
    let Some(head) = head else {
        return Ok(());
    };
    if !tracked(tx, target)? {
        return Ok(());
    }
    release(tx, target, when)?;
    consider(tx, target, head, when)
}
