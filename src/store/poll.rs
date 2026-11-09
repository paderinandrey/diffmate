use super::Store;
use crate::app::ProcessInfo;
use crate::error::Error;
use crate::pr_url::PrRef;
use rusqlite::TransactionBehavior;

mod known;
mod schedule;
#[cfg(test)]
mod tests;
mod tracking;

pub(crate) struct SeenRequest {
    pub pr: PrRef,
    pub is_draft: bool,
    pub head_sha: Option<String>,
}

pub(crate) enum Absent {
    Skip,
    Closed,
    Open {
        is_draft: bool,
        head_sha: Option<String>,
    },
}

pub(crate) struct KnownTarget {
    pub pr: PrRef,
    pub tracking: String,
    pub request_seen: bool,
    #[cfg_attr(not(test), allow(dead_code))]
    pub queued_head: Option<String>,
}

impl Store {
    pub async fn known_targets(&self) -> Result<Vec<KnownTarget>, Error> {
        self.call(known::known).await
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn discard(&self, pr: &PrRef) -> Result<(), Error> {
        let pr = pr.clone();
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tracking::discard(&tx, &pr, super::now_ms())?;
            tx.commit()
        })
        .await
    }

    pub async fn apply_poll(
        &self,
        seen: Vec<SeenRequest>,
        absent: Vec<(PrRef, Absent)>,
        now_ms: i64,
        debounce_ms: i64,
        process: &ProcessInfo,
    ) -> Result<(), Error> {
        let (pid, started) = (process.pid, process.started_at_ms);
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tracking::apply(&tx, &seen, &absent, now_ms)?;
            schedule::apply(&tx, &seen, &absent, now_ms, debounce_ms, pid, started)?;
            tx.commit()
        })
        .await
    }

    pub async fn schedule_after_run(
        &self,
        target: super::TargetId,
        now_ms: i64,
        debounce_ms: i64,
        process: &ProcessInfo,
    ) -> Result<(), Error> {
        let (pid, started) = (process.pid, process.started_at_ms);
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            schedule::after_run(&tx, target, now_ms, debounce_ms, pid, started)?;
            tx.commit()
        })
        .await
    }
}
