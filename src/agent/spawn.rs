use super::claude::{ClaudeCli, user_prompt};
use super::probe::{self, Refusal};
use super::stop::halt;
use super::{AgentRequest, FailureDetail, gitwrap};
use crate::cancel;
use crate::error::Error;
use crate::proc::{self, Captured, Limits};
use crate::procenv::Pairs;
use std::ffi::{OsStr, OsString};
use std::process::Stdio;
use tokio::process::Command;

const PROGRAM: &str = "claude";

fn refused(error: &Error) -> Refusal {
    Box::new(FailureDetail {
        message: error.to_string(),
        ..FailureDetail::default()
    })
}

async fn environment(cli: &ClaudeCli, request: &AgentRequest) -> Result<Pairs, Error> {
    let inputs = [
        &request.worktree,
        &request.context_dir,
        &request.prompt_file,
    ];
    for input in inputs {
        let found = tokio::fs::try_exists(input).await;
        if !found.map_err(Error::io("check agent input"))? {
            return Err(Error::Missing(input.clone()));
        }
    }
    let bin = gitwrap::install(&request.prompt_file, &cli.real_git).await?;
    cli.run_env(&bin)
}

type Invoked = Result<Option<Captured>, Refusal>;

pub(super) async fn invoke(cli: &ClaudeCli, request: &AgentRequest) -> Invoked {
    let env = environment(cli, request).await.map_err(|e| refused(&e))?;
    let program = OsStr::new(PROGRAM);
    probe::check_version(program, &env).await?;
    let args = cli.args_for(request);
    probe::check_flags(program, &env, &args, &request.worktree).await?;
    start(&env, &args, request).await.map_err(|e| refused(&e))
}

async fn start(
    env: &Pairs,
    args: &[OsString],
    request: &AgentRequest,
) -> Result<Option<Captured>, Error> {
    let mut cancel = request.cancel.clone();
    if cancel.as_ref().is_some_and(|stop| *stop.borrow()) {
        return Ok(None);
    }
    let mut command = Command::new(PROGRAM);
    command.args(args);
    command.current_dir(&request.worktree).env_clear();
    command.envs(env.iter().map(|(name, value)| (name, value)));
    command.process_group(0).kill_on_drop(true);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(Error::io("start claude"))?;
    let pgid = child.id();
    if let (Some(hook), Some(pid)) = (&request.spawn_hook, pgid) {
        let _ = hook.send(pid);
    }
    let prompt = user_prompt(&request.context_dir).into_bytes();
    let limits = Limits {
        stdout: 8 << 20,
        stderr: 64 << 10,
    };
    let finished = Box::pin(proc::finish(&mut child, Some(prompt), limits));
    let captured = match cancel.as_mut() {
        Some(stop) => cancel::raced(stop, finished).await,
        None => Some(finished.await),
    };
    let Some(captured) = captured else {
        halt(&mut child, pgid).await;
        return Ok(None);
    };
    captured.map(Some).map_err(Error::io("run claude"))
}
