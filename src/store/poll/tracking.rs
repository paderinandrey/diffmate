use super::{Absent, SeenRequest};
use crate::pr_url::PrRef;
use crate::store::TargetId;
use rusqlite::{OptionalExtension, Transaction, params};

pub(super) fn discard(tx: &Transaction, pr: &PrRef, now: i64) -> rusqlite::Result<()> {
    let id = upsert(tx, pr, now)?;
    tx.execute(
        "UPDATE targets SET tracking = 'untracked', request_seen = 1 WHERE id = ?1",
        [id],
    )?;
    tx.execute(
        "DELETE FROM jobs WHERE target_id = ?1 AND kind = 'review' AND origin = 'automatic'
         AND state = 'queued' AND retry_count = 0
         AND NOT EXISTS (SELECT 1 FROM runs WHERE job_id = jobs.id)",
        [id],
    )?;
    Ok(())
}

pub(super) fn apply(
    tx: &Transaction,
    seen: &[SeenRequest],
    absent: &[(PrRef, Absent)],
    now: i64,
) -> rusqlite::Result<()> {
    for row in seen {
        see(tx, &row.pr, now)?;
    }
    for (pr, kind) in absent {
        miss(tx, pr, kind, now)?;
    }
    Ok(())
}

pub(super) fn target_id(tx: &Transaction, pr: &PrRef) -> rusqlite::Result<Option<TargetId>> {
    let number = number(pr)?;
    tx.query_row(
        "SELECT id FROM targets WHERE owner = ?1 AND repo = ?2 AND number = ?3",
        params![pr.owner, pr.repo, number],
        |row| row.get(0),
    )
    .optional()
}

fn see(tx: &Transaction, pr: &PrRef, now: i64) -> rusqlite::Result<()> {
    let id = upsert(tx, pr, now)?;
    tx.execute(
        "UPDATE targets SET tracking = 'tracked', request_seen = 1
         WHERE id = ?1 AND request_seen = 0",
        [id],
    )
    .map(drop)
}

fn miss(tx: &Transaction, pr: &PrRef, kind: &Absent, now: i64) -> rusqlite::Result<()> {
    let Some(id) = target_id(tx, pr)? else {
        return Ok(());
    };
    match kind {
        Absent::Skip => Ok(()),
        Absent::Closed => {
            tx.execute(
                "UPDATE targets SET tracking = 'untracked', request_seen = 0
                 WHERE id = ?1 AND tracking = 'tracked'",
                [id],
            )?;
            tx.execute(
                "UPDATE jobs SET state = 'cancelled', updated_at = ?1
                 WHERE target_id = ?2 AND kind = 'review' AND origin = 'automatic'
                 AND state = 'queued'",
                params![now, id],
            )?;
            Ok(())
        }
        Absent::Open { .. } => tx
            .execute("UPDATE targets SET request_seen = 0 WHERE id = ?1", [id])
            .map(drop),
    }
}

fn upsert(tx: &Transaction, pr: &PrRef, now: i64) -> rusqlite::Result<TargetId> {
    let number = number(pr)?;
    tx.execute(
        "INSERT INTO targets (owner, repo, number, created_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (owner, repo, number) DO NOTHING",
        params![pr.owner, pr.repo, number, now],
    )?;
    target_id(tx, pr)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

fn number(pr: &PrRef) -> rusqlite::Result<i64> {
    i64::try_from(pr.number).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}
