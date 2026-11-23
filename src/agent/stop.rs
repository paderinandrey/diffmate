#[cfg(test)]
mod tests;

use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};
use std::io::{self, ErrorKind};
use std::time::Duration;

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
            // XNU reports EPERM for a group containing only our unreaped zombie.
            // Only ESRCH, checked separately after reaping, proves the group is gone.
            #[cfg(target_vendor = "apple")]
            Err(Errno::PERM) => Ok(()),
            other => other.map_err(io::Error::from),
        }
    }

    fn alive(&self, pgid: u32) -> io::Result<bool> {
        match test_kill_process_group(checked(pgid)?) {
            Ok(()) | Err(Errno::PERM) => Ok(true),
            Err(Errno::SRCH) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

pub(super) async fn drained<S: GroupSignaller>(signaller: &S, pgid: u32) -> io::Result<()> {
    while signaller.alive(pgid)? {
        tokio::time::sleep(GROUP_POLL).await;
    }
    Ok(())
}

pub(super) async fn stop_group<S: GroupSignaller>(
    signaller: &S,
    pgid: u32,
    grace: Duration,
) -> io::Result<()> {
    checked(pgid)?;
    signaller.signal(pgid, StopSignal::Term)?;
    match tokio::time::timeout(grace, drained(signaller, pgid)).await {
        Ok(result) => result,
        Err(_) => signaller.signal(pgid, StopSignal::Kill),
    }
}
