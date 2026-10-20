#[cfg(test)]
mod tests;

use crate::error::Error;
use std::fs::{File, OpenOptions, TryLockError};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::Duration;

pub(crate) const LOCK_POLL: Duration = Duration::from_millis(500);

pub(crate) struct RunnerLock {
    _file: File,
}

pub(crate) async fn try_acquire(path: &Path) -> Result<Option<RunnerLock>, Error> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<Option<RunnerLock>, Error> {
        let mut options = OpenOptions::new();
        options.create(true).write(true).truncate(false).mode(0o600);
        let file = options
            .open(&path)
            .map_err(Error::io("open the runner lock"))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(RunnerLock { _file: file })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(error)) => Err(Error::io("take the runner lock")(error)),
        }
    })
    .await?
}

pub(crate) async fn acquire(
    path: &Path,
    poll: Duration,
    on_first_wait: impl FnOnce(),
) -> Result<RunnerLock, Error> {
    let mut on_first_wait = Some(on_first_wait);
    loop {
        if let Some(lock) = try_acquire(path).await? {
            return Ok(lock);
        }
        if let Some(notify) = on_first_wait.take() {
            notify();
        }
        tokio::time::sleep(poll).await;
    }
}
