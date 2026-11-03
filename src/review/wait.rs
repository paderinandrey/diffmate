use crate::error::Error;
use crate::lock::{self, LOCK_POLL, RunnerLock};
use crate::store::{JobId, Store, jobs::JobProgress};
use std::path::Path;
use tokio::signal::unix::{SignalKind, signal};

pub(super) enum Waited {
    Locked(RunnerLock),
    Finished,
    Stopped,
}

pub(super) async fn until_lock_or_end(
    path: &Path,
    store: &Store,
    job: JobId,
    label: &str,
) -> Result<Waited, Error> {
    let mut interrupts = signal(SignalKind::interrupt()).map_err(Error::io("listen for SIGINT"))?;
    let mut terminate = signal(SignalKind::terminate()).map_err(Error::io("listen for SIGTERM"))?;
    let polling = async {
        let mut notified = false;
        loop {
            if matches!(store.job_progress(job).await?, JobProgress::Finished) {
                return Ok(Waited::Finished);
            }
            if let Some(lock) = lock::try_acquire(path).await? {
                return Ok(Waited::Locked(lock));
            }
            if !notified {
                eprintln!(
                    "waiting: another diffmate process holds the runner lock; job {job} stays queued until it is released"
                );
                notified = true;
            }
            tokio::time::sleep(LOCK_POLL).await;
        }
    };
    tokio::select! {
        result = polling => result,
        _ = interrupts.recv() => Ok(stopped(label)),
        _ = terminate.recv() => Ok(stopped(label)),
    }
}

fn stopped(label: &str) -> Waited {
    eprintln!("{label}: Stopped waiting; the job stays queued");
    Waited::Stopped
}
