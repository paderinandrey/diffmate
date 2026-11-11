#![cfg_attr(not(test), expect(dead_code))]

mod cancel;
mod confirm;
mod insert;
mod rows;
mod save;
mod sync;

pub(crate) use cancel::{CancelQueued, Gate};
pub(crate) use sync::insert as insert_sync_job;

use super::Store;
use crate::app::ProcessInfo;
use crate::error::Error;
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ConfirmError {
    #[error("undecided findings")]
    Undecided(Vec<String>),
    #[error("this draft is already submitted")]
    DraftSubmitted,
    #[error("a publication is already pending, sending, or unknown")]
    Blocked,
    #[error(transparent)]
    Database(#[from] Error),
}

pub(super) enum Outcome {
    Job(i64),
    Undecided(Vec<String>),
    DraftSubmitted,
    Blocked,
}

pub(crate) struct ServiceJob {
    pub id: i64,
    pub kind: String,
}

impl Store {
    pub async fn confirm_publication(
        &self,
        draft_id: i64,
        event: &str,
        confirmed_sha: &str,
        process: &ProcessInfo,
    ) -> Result<i64, ConfirmError> {
        let event = event.to_string();
        let confirmed_sha = confirmed_sha.to_string();
        let (pid, started) = (i64::from(process.pid), process.started_at_ms);
        let outcome = self
            .call(move |conn| insert::write(conn, draft_id, &event, &confirmed_sha, pid, started))
            .await;
        match outcome {
            Ok(Outcome::Job(id)) => Ok(id),
            Ok(Outcome::Undecided(ids)) => Err(ConfirmError::Undecided(ids)),
            Ok(Outcome::DraftSubmitted) => Err(ConfirmError::DraftSubmitted),
            Ok(Outcome::Blocked) => Err(ConfirmError::Blocked),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn publish_gate(&self, target: i64) -> Result<Gate, Error> {
        self.call(move |conn| cancel::gate(conn, target)).await
    }

    pub async fn cancel_queued(&self, target: i64) -> Result<CancelQueued, Error> {
        self.call(move |conn| cancel::write(conn, target)).await
    }

    pub async fn mark_needs_confirmation(&self, job_id: i64, seen_sha: &str) -> Result<(), Error> {
        let seen_sha = seen_sha.to_string();
        self.call(move |conn| confirm::needs_confirmation(conn, job_id, &seen_sha))
            .await
    }

    pub async fn reconfirm(&self, publication_id: &str, new_sha: &str) -> Result<(), Error> {
        let publication_id = publication_id.to_string();
        let new_sha = new_sha.to_string();
        self.call(move |conn| confirm::reconfirm(conn, &publication_id, &new_sha))
            .await
    }

    pub async fn decline(&self, publication_id: &str) -> Result<(), Error> {
        let publication_id = publication_id.to_string();
        self.call(move |conn| confirm::decline(conn, &publication_id))
            .await
    }

    pub async fn unknown_publication(&self, target: i64) -> Result<Option<String>, Error> {
        self.call(move |conn| {
            let sql = "SELECT id FROM publications WHERE target_id = ?1 AND state = 'unknown' ORDER BY created_at DESC LIMIT 1";
            conn.query_row(sql, [target], |row| row.get(0)).optional()
        })
        .await
    }

    pub async fn enqueue_reconcile(
        &self,
        publication_id: &str,
        process: &ProcessInfo,
    ) -> Result<(), Error> {
        let publication_id = publication_id.to_string();
        let pid = i64::from(process.pid);
        let started = process.started_at_ms;
        self.call(move |conn| {
            let now = crate::store::now_ms();
            let sql = "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, origin, publication_id) SELECT 'reconcile', p.target_id, 'queued', ?2, ?3, ?4, ?4, 'manual', p.id FROM publications p WHERE p.id = ?1 AND p.state = 'unknown' AND NOT EXISTS (SELECT 1 FROM jobs j WHERE j.publication_id = p.id AND j.kind = 'reconcile' AND j.state IN ('queued', 'running'))";
            conn.execute(sql, params![publication_id, pid, started, now])?;
            Ok(())
        })
        .await
    }

    pub async fn abandon_unknown(&self, publication_id: &str) -> Result<(), Error> {
        let publication_id = publication_id.to_string();
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = crate::store::now_ms();
            let sql = "UPDATE publications SET state = 'abandoned', reason = 'confirmed by me', updated_at = ?2 WHERE id = ?1 AND state = 'unknown'";
            if tx.execute(sql, params![publication_id, now])? != 1 {
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                    Some("publication is not unknown".into()),
                ));
            }
            tx.commit()?;
            Ok(())
        })
        .await
    }

    pub async fn abandon_pending(
        &self,
        publication_id: &str,
        job_id: i64,
        reason: &str,
    ) -> Result<(), Error> {
        let publication_id = publication_id.to_string();
        let reason = reason.to_string();
        self.call(move |conn| confirm::abandon(conn, &publication_id, job_id, &reason))
            .await
    }

    pub async fn next_service_job(&self, now: i64) -> Result<Option<ServiceJob>, Error> {
        self.call(move |conn| {
            let sql = "SELECT id, kind FROM jobs WHERE kind IN ('publish', 'reconcile', 'sync') \
                       AND state = 'queued' AND held = 0 \
                       AND (next_attempt_at IS NULL OR next_attempt_at <= ?1) \
                       ORDER BY id LIMIT 1";
            conn.query_row(sql, [now], |row| {
                Ok(ServiceJob {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                })
            })
            .optional()
        })
        .await
    }

    pub async fn enqueue_sync(&self, publication_id: &str) -> Result<(), Error> {
        let publication_id = publication_id.to_string();
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            sync::insert(&tx, &publication_id, crate::store::now_ms())?;
            tx.commit()?;
            Ok(())
        })
        .await
    }
}
