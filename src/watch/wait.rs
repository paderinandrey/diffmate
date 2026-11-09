use super::{WATCH_LOCK_POLL, events};
use crate::error::Error;
use crate::lock::{self, RunnerLock};
use std::path::Path;

pub(super) async fn lock(path: &Path) -> Result<RunnerLock, Error> {
    let mut notified = false;
    loop {
        if let Some(lock) = lock::try_acquire(path).await? {
            return Ok(lock);
        }
        if !notified {
            let suffix = lock::holder(path)
                .await
                .map_or_else(String::new, |role| format!(" (held by {})", role.as_str()));
            events::print(&format!("waiting for the runner lock{suffix}"));
            notified = true;
        }
        tokio::time::sleep(WATCH_LOCK_POLL).await;
    }
}
