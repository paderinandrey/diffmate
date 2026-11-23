mod child;
mod client;
mod group;

pub(super) use client::capture;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::os::unix::net::UnixStream as Socket;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::net::UnixStream;

pub(crate) const ENTRY: &str = "__agent-supervisor";
const READY: u8 = 1;
const START: u8 = 2;

pub(crate) async fn lock_run(dir: PathBuf) -> io::Result<File> {
    tokio::task::spawn_blocking(move || {
        let file = File::open(dir)?;
        file.lock()?;
        Ok(file)
    })
    .await
    .map_err(io::Error::other)?
}

fn stream(socket: Socket) -> io::Result<UnixStream> {
    socket.set_nonblocking(true)?;
    UnixStream::from_std(socket)
}

#[cfg(not(test))]
fn executable() -> io::Result<PathBuf> {
    std::env::current_exe()
}

#[cfg(test)]
fn executable() -> io::Result<PathBuf> {
    // Cargo's unit harness is not the CLI. Its layout differs between Cargo versions.
    if let Some(path) = std::env::var_os("NEXTEST_BIN_EXE_diffmate") {
        return Ok(path.into());
    }
    let harness = std::env::current_exe()?;
    let artifacts = harness.ancestors().find(|path| {
        matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some("build" | "deps")
        )
    });
    let profile = artifacts
        .and_then(std::path::Path::parent)
        .ok_or_else(|| io::Error::other("cannot locate the diffmate test binary"))?;
    Ok(profile.join("diffmate"))
}

pub(crate) fn entry(args: impl Iterator<Item = OsString>) -> ExitCode {
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .and_then(|runtime| runtime.block_on(child::run(args)));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("agent supervisor: {error}");
            ExitCode::FAILURE
        }
    }
}
