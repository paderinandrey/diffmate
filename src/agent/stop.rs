#[cfg(test)]
mod tests;

use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group};
use std::future::Future;
use std::io::{self, ErrorKind};
use std::pin::pin;
use std::time::Duration;
use tokio::process::Child;

pub(crate) const AGENT_STOP_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopSignal {
    Term,
    Kill,
}

pub(crate) trait GroupSignaller {
    fn signal(&self, pgid: u32, signal: StopSignal) -> io::Result<()>;
}

pub(crate) struct RustixSignaller;

fn checked(pgid: u32) -> io::Result<Pid> {
    let raw = i32::try_from(pgid).ok().filter(|id| *id > 1);
    raw.and_then(Pid::from_raw).ok_or_else(|| {
        let message = format!("refusing to signal process group {pgid}");
        io::Error::new(ErrorKind::InvalidInput, message)
    })
}

impl GroupSignaller for RustixSignaller {
    fn signal(&self, pgid: u32, signal: StopSignal) -> io::Result<()> {
        let signal = match signal {
            StopSignal::Term => Signal::TERM,
            StopSignal::Kill => Signal::KILL,
        };
        match kill_process_group(checked(pgid)?, signal) {
            Err(Errno::SRCH) => Ok(()),
            other => other.map_err(io::Error::from),
        }
    }
}

pub(crate) async fn stop_group<S: GroupSignaller, W: Future<Output = ()>>(
    signaller: &S,
    pgid: u32,
    grace: Duration,
    wait: W,
) -> io::Result<()> {
    checked(pgid)?;
    signaller.signal(pgid, StopSignal::Term)?;
    let mut wait = pin!(wait);
    if tokio::time::timeout(grace, wait.as_mut()).await.is_err() {
        signaller.signal(pgid, StopSignal::Kill)?;
        wait.await;
    }
    Ok(())
}

pub(super) async fn halt(child: &mut Child, pgid: Option<u32>) {
    let stopped = match pgid {
        Some(pgid) => {
            let wait = async {
                let _ = child.wait().await;
            };
            stop_group(&RustixSignaller, pgid, AGENT_STOP_GRACE, wait).await
        }
        None => Err(io::Error::other("the agent has no process id")),
    };
    if stopped.is_err() {
        let _ = child.start_kill();
        let _ = child.wait().await;
    }
}
