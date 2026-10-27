use super::{Env, MIN_VERSION, Refusal, dotted, interpret_probe, parse_version, refuse};
use crate::agent::stop::supervise;
use crate::cancel::CancelRx;
use crate::proc::{Captured, Limits, excerpt};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

pub(super) async fn capture(
    program: &OsStr,
    args: &[OsString],
    env: &Env,
    cwd: Option<&Path>,
    cancel: Option<CancelRx>,
) -> Result<Option<Captured>, Refusal> {
    let mut command = Command::new(program);
    command.args(args).env_clear();
    command.envs(env.iter().map(|(name, value)| (name, value)));
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command.process_group(0).kill_on_drop(true);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let failed =
        |what: &str, error: std::io::Error| refuse(format!("{what} claude: {error}"), None, &[]);
    let child = command.spawn().map_err(|error| failed("start", error))?;
    let limits = Limits {
        stdout: 64 << 10,
        stderr: 64 << 10,
    };
    let done = Box::pin(supervise(child, None, limits, cancel)).await;
    done.map_err(|error| failed("run", error))
}

pub(crate) async fn check_version(
    program: &OsStr,
    env: &Env,
    cancel: Option<CancelRx>,
) -> Result<Option<()>, Refusal> {
    let args = [OsString::from("--version")];
    let Some(done) = capture(program, &args, env, None, cancel).await? else {
        return Ok(None);
    };
    let text = String::from_utf8_lossy(&done.stdout);
    let code = done.status.code();
    let Some(found) = parse_version(&text).filter(|_| done.status.success()) else {
        let shown = excerpt(text.as_bytes(), 100);
        let message = format!("cannot read the Claude CLI version from {shown:?}");
        return Err(refuse(message, code, &done.stderr));
    };
    if found < MIN_VERSION {
        let (found, minimum) = (dotted(found), dotted(MIN_VERSION));
        let message = format!("Claude CLI {found} is older than the minimum {minimum}");
        return Err(refuse(message, code, &done.stderr));
    }
    Ok(Some(()))
}

pub(crate) async fn check_flags(
    program: &OsStr,
    env: &Env,
    args: &[OsString],
    cwd: &Path,
    cancel: Option<CancelRx>,
) -> Result<Option<()>, Refusal> {
    let Some(done) = capture(program, args, env, Some(cwd), cancel).await? else {
        return Ok(None);
    };
    interpret_probe(&done.stdout, &done.stderr, done.status.code()).map(Some)
}
