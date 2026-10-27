#[cfg(test)]
mod tests;

use crate::cancel::{self, CancelRx};
use crate::proc::{self, Captured, Limits};
use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};
use std::future::Future;
use std::io::{self, ErrorKind};
use std::pin::pin;
use std::time::Duration;
use tokio::process::Child;

pub(crate) const AGENT_STOP_GRACE: Duration = Duration::from_secs(5);
const GROUP_POLL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopSignal {
    Term,
    Kill,
}

pub(crate) trait GroupSignaller {
    fn signal(&self, pgid: u32, signal: StopSignal) -> io::Result<()>;
    fn alive(&self, pgid: u32) -> io::Result<bool>;
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

    fn alive(&self, pgid: u32) -> io::Result<bool> {
        match test_kill_process_group(checked(pgid)?) {
            Ok(()) => Ok(true),
            Err(Errno::SRCH) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

async fn drained<S: GroupSignaller>(signaller: &S, pgid: u32) -> io::Result<()> {
    while signaller.alive(pgid)? {
        tokio::time::sleep(GROUP_POLL).await;
    }
    Ok(())
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
    let mut reaped = false;
    let stopped = async {
        wait.as_mut().await;
        reaped = true;
        drained(signaller, pgid).await
    };
    if let Ok(stopped) = tokio::time::timeout(grace, stopped).await {
        return stopped;
    }
    signaller.signal(pgid, StopSignal::Kill)?;
    if !reaped {
        wait.await;
    }
    let killed = tokio::time::timeout(grace, drained(signaller, pgid)).await;
    killed.unwrap_or(Ok(()))
}

pub(super) async fn supervise(
    mut child: Child,
    stdin: Option<Vec<u8>>,
    limits: Limits,
    cancel: Option<CancelRx>,
) -> io::Result<Option<Captured>> {
    let pgid = child.id();
    let finished = Box::pin(proc::finish(&mut child, stdin, limits));
    let captured = match cancel {
        Some(mut stop) => cancel::raced(&mut stop, finished).await,
        None => Some(finished.await),
    };
    let Some(captured) = captured else {
        halt(&mut child, pgid).await;
        return Ok(None);
    };
    captured.map(Some)
}

async fn halt(child: &mut Child, pgid: Option<u32>) {
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
