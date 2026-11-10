#![cfg_attr(not(test), expect(dead_code))]

mod cancel;
mod confirm;
mod insert;
mod rows;
mod save;

pub(crate) use cancel::{CancelQueued, Gate};

use super::Store;
use crate::app::ProcessInfo;
use crate::error::Error;
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
}
