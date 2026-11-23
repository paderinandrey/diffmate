use super::{ENTRY, READY, START, Socket, executable, stream};
use crate::agent::AgentRequest;
use crate::cancel;
use crate::crash::crash_point;
use crate::proc::{self, Captured, Limits};
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Stdio};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Command;

async fn exchange(
    socket: &mut UnixStream,
    review: Option<&AgentRequest>,
) -> io::Result<ExitStatus> {
    if socket.read_u8().await? != READY {
        return Err(io::Error::other(
            "agent supervisor did not acquire the run guard",
        ));
    }
    if review.is_some() {
        crash_point!("agent-guarded");
    }
    socket.write_u8(START).await?;
    let pid = socket.read_u32().await?;
    if let Some(request) = review {
        crash_point!("agent-spawned");
        if let Some(hook) = &request.spawn_hook {
            let _ = hook.send(pid);
        }
    }
    Ok(ExitStatus::from_raw(socket.read_i32().await?))
}

pub(in crate::agent) async fn capture(
    program: &OsStr,
    args: &[OsString],
    env: &[(OsString, OsString)],
    request: Option<&AgentRequest>,
    review: bool,
    limits: Limits,
) -> io::Result<Option<Captured>> {
    let mut cancel = request.and_then(|request| request.cancel.clone());
    if cancel.as_ref().is_some_and(|stop| *stop.borrow()) {
        return Ok(None);
    }
    let (local, remote) = Socket::pair()?;
    let mut socket = stream(local)?;
    let guard = request
        .and_then(|r| r.guard_dir.as_deref())
        .map_or(OsStr::new(""), std::path::Path::as_os_str);
    let review = request.filter(|_| review);
    let mut command = Command::new(executable()?);
    let cwd = review.map_or(OsStr::new(""), |r| r.worktree.as_os_str());
    command
        .arg(ENTRY)
        .arg(guard)
        .arg(cwd)
        .arg(program)
        .args(args);
    command.env_clear().envs(env.iter().map(|(k, v)| (k, v)));
    // The supervisor is this binary too; only its LLVM runtime gets the profile path.
    if let Some(path) = std::env::var_os("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", path);
    }
    command
        .stdin(Stdio::from(OwnedFd::from(remote)))
        .process_group(0);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = command.spawn()?;
    drop(command);
    let control = async move {
        let protocol = exchange(&mut socket, review);
        let result = match cancel.as_mut() {
            Some(rx) => cancel::raced(rx, protocol).await,
            None => Some(protocol.await),
        };
        drop(socket);
        result
    };
    let (status, output) = tokio::join!(control, Box::pin(proc::finish(child, None, limits)));
    let mut output = output?;
    let Some(status) = status else {
        return Ok(None);
    };
    output.status = status.map_err(|error| {
        let detail = proc::excerpt(&output.stderr, 500);
        io::Error::other(format!("agent supervisor: {error}: {detail}"))
    })?;
    Ok(Some(output))
}
