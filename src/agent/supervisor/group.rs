use crate::agent::stop::{
    AGENT_STOP_GRACE, GroupSignaller, RustixSignaller, StopSignal, drained, stop_group,
};
use rustix::io::Errno;
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
use std::io;
use std::process::ExitStatus;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Child;
use tokio::signal::unix::Signal;

async fn exited(pid: Pid, changed: &mut Signal) -> io::Result<()> {
    // WNOWAIT pins the group leader's PID until after the last group signal.
    let options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG;
    loop {
        match waitid(WaitId::Pid(pid), options) {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {
                let _ = changed.recv().await;
            }
            Err(Errno::INTR) => (),
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(target_os = "linux")]
async fn reap_adopted(pid: Pid, changed: &mut Signal) -> io::Result<()> {
    use rustix::process::{WaitOptions, waitpgid};
    loop {
        match waitpgid(pid, WaitOptions::NOHANG) {
            Ok(Some(_)) | Err(Errno::INTR) => (),
            Ok(None) => {
                let _ = changed.recv().await;
            }
            Err(Errno::CHILD) => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
}

pub(super) async fn finish(
    child: &mut Child,
    mut changed: Signal,
    owner: &mut UnixStream,
) -> io::Result<ExitStatus> {
    let pgid = child
        .id()
        .ok_or_else(|| io::Error::other("missing Grok PID"))?;
    let leader = i32::try_from(pgid)
        .ok()
        .and_then(Pid::from_raw)
        .ok_or_else(|| io::Error::other("invalid Grok PID"))?;
    let exited = if owner.write_u32(pgid).await.is_ok() {
        tokio::select! {
            status = exited(leader, &mut changed) => Some(status),
            _ = owner.read_u8() => None,
        }
    } else {
        None
    };
    let signalled = match exited {
        None => stop_group(&RustixSignaller, pgid, AGENT_STOP_GRACE).await,
        Some(Ok(())) => RustixSignaller.signal(pgid, StopSignal::Kill),
        Some(Err(error)) => Err(error),
    };
    let status = child.wait().await;
    #[cfg(target_os = "linux")]
    let reaped = reap_adopted(leader, &mut changed).await;
    drained(&RustixSignaller, pgid).await?;
    #[cfg(target_os = "linux")]
    reaped?;
    signalled?;
    status
}
