use super::{Env, MIN_VERSION, Refusal, dotted, interpret_help, parse_version, refuse};
use crate::agent::{AgentRequest, supervisor};
use crate::proc::{Captured, Limits, excerpt};
use std::ffi::{OsStr, OsString};

async fn capture(
    program: &OsStr,
    args: &[OsString],
    env: &Env,
    request: Option<&AgentRequest>,
) -> Result<Option<Captured>, Refusal> {
    let failed = |error: std::io::Error| refuse(format!("run grok: {error}"), None, &[]);
    let limits = Limits {
        stdout: 64 << 10,
        stderr: 64 << 10,
    };
    Box::pin(supervisor::capture(
        program, args, env, request, false, limits,
    ))
    .await
    .map_err(failed)
}

pub(crate) async fn check_version(
    program: &OsStr,
    env: &Env,
    request: Option<&AgentRequest>,
) -> Result<Option<()>, Refusal> {
    let args = [OsString::from("--version")];
    let Some(done) = capture(program, &args, env, request).await? else {
        return Ok(None);
    };
    let text = String::from_utf8_lossy(&done.stdout);
    let code = done.status.code();
    let Some(found) = parse_version(&text).filter(|_| done.status.success()) else {
        let shown = excerpt(text.as_bytes(), 100);
        let message = format!("cannot read the Grok CLI version from {shown:?}");
        return Err(refuse(message, code, &done.stderr));
    };
    if found < MIN_VERSION {
        let (found, minimum) = (dotted(found), dotted(MIN_VERSION));
        let message = format!("Grok CLI {found} is older than the minimum {minimum}");
        return Err(refuse(message, code, &done.stderr));
    }
    Ok(Some(()))
}

pub(crate) async fn check_flags(
    program: &OsStr,
    env: &Env,
    request: Option<&AgentRequest>,
) -> Result<Option<()>, Refusal> {
    let args = [OsString::from("--help")];
    let Some(done) = capture(program, &args, env, request).await? else {
        return Ok(None);
    };
    interpret_help(&done.stdout, done.status.code()).map(Some)
}
