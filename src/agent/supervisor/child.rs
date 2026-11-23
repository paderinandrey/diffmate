use super::{READY, START, Socket, group, lock_run, stream};
use crate::agent::claude::{AUTH_ENV, LOCKED_ENV};
use std::ffi::OsString;
use std::io;
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::signal::unix::{SignalKind, signal};

pub(super) async fn run(mut args: impl Iterator<Item = OsString>) -> io::Result<()> {
    let missing = || io::Error::other("incomplete agent supervisor invocation");
    let dir = args.next().ok_or_else(missing)?;
    let cwd = args.next().ok_or_else(missing)?;
    let program = args.next().ok_or_else(missing)?;
    let socket = Socket::from(io::stdin().as_fd().try_clone_to_owned()?);
    socket.peer_addr()?;
    let mut owner = stream(socket)?;
    let _guard = if dir.is_empty() {
        None
    } else {
        Some(lock_run(dir.into()).await?)
    };
    // The runner cannot authorize launch until recovery would have to wait on our lock.
    owner.write_u8(READY).await?;
    if owner.read_u8().await? != START {
        return Err(io::Error::other("agent launch was not authorized"));
    }
    let mut command = Command::new(program);
    command.env_clear();
    for key in AUTH_ENV {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command.envs(LOCKED_ENV.iter().copied());
    command.env("GIT_CONFIG_GLOBAL", "/dev/null");
    command.env("GIT_CONFIG_NOSYSTEM", "1");
    if !cwd.is_empty() {
        command.current_dir(cwd);
    }
    let first = args.next().ok_or_else(missing)?;
    let prompt = if first == "-p" {
        Some(args.next().ok_or_else(missing)?)
    } else {
        None
    };
    command.arg(first).args(&prompt).args(args);
    let stdin = if prompt.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    command.stdin(stdin).process_group(0);
    let changed = signal(SignalKind::child())?;
    #[cfg(target_os = "linux")]
    rustix::process::set_child_subreaper(Some(rustix::process::Pid::INIT))?;
    let mut child = command.spawn()?;
    let pipe = child.stdin.take();
    let feed = async {
        if let (Some(mut pipe), Some(prompt)) = (pipe, prompt) {
            let _ = pipe.write_all(prompt.as_bytes()).await;
        }
    };
    let ((), status) = tokio::join!(feed, group::finish(&mut child, changed, &mut owner));
    owner.write_i32(status?.into_raw()).await
}
