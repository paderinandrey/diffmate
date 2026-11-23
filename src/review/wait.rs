use super::signals::Signals;
use crate::cancel::StopReason;
use crate::error::Error;
use crate::lock::{self, LOCK_POLL, RunnerLock};
use crate::store::{JobId, Store, jobs::JobProgress};
use std::path::Path;
#[cfg(test)]
mod tests;

pub(super) enum Waited {
    Locked(RunnerLock),
    Finished,
    Held,
    Stopped,
}

pub(super) async fn until_lock_or_end(
    path: &Path,
    store: &Store,
    job: JobId,
    label: &str,
    signals: &mut Signals,
) -> Result<Waited, Error> {
    let polling = async {
        let mut notified = false;
        loop {
            let delayed = match store.job_progress(job, crate::store::now_ms()).await? {
                JobProgress::Finished => return Ok(Waited::Finished),
                JobProgress::Held => return Ok(Waited::Held),
                JobProgress::Delayed => true,
                JobProgress::Active => false,
            };
            if !delayed && let Some(lock) = lock::try_acquire(path).await? {
                return Ok(Waited::Locked(lock));
            }
            if !notified {
                if delayed {
                    eprintln!("waiting: job {job} is not due; waiting for its scheduled time");
                } else {
                    eprintln!(
                        "waiting: another diffmate process holds the runner lock; job {job} stays queued until it is released"
                    );
                }
                notified = true;
            }
            tokio::time::sleep(LOCK_POLL).await;
        }
    };
    tokio::select! {
        biased;
        _ = signals.next(StopReason::UserCancel) => Ok(stopped(path, label).await),
        result = polling => result,
    }
}

async fn stopped(path: &Path, label: &str) -> Waited {
    let suffix = if lock::holder(path).await == Some(lock::Role::Watch) {
        " and watch will run it"
    } else {
        ""
    };
    eprintln!("{label}: Stopped waiting; the job stays queued{suffix}");
    Waited::Stopped
}
