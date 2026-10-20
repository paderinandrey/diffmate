use super::claude::{ClaudeCli, user_prompt};
use super::probe::{self, Refusal};
use super::{AgentRequest, FailureDetail, gitwrap};
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

pub(super) async fn invoke(cli: &ClaudeCli, request: &AgentRequest) -> Result<Captured, Refusal> {
    let env = environment(cli, request).await.map_err(|e| refused(&e))?;
    let program = OsStr::new(PROGRAM);
    probe::check_version(program, &env).await?;
    let args = cli.args_for(request);
    probe::check_flags(program, &env, &args, &request.worktree).await?;
    start(&env, &args, request).await.map_err(|e| refused(&e))
}

async fn start(env: &Pairs, args: &[OsString], request: &AgentRequest) -> Result<Captured, Error> {
    let mut command = Command::new(PROGRAM);
    command.args(args);
    command.current_dir(&request.worktree).env_clear();
    command.envs(env.iter().map(|(name, value)| (name, value)));
    command.process_group(0).kill_on_drop(true);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn().map_err(Error::io("start claude"))?;
    let prompt = user_prompt(&request.context_dir).into_bytes();
    let limits = Limits {
        stdout: 8 << 20,
        stderr: 64 << 10,
    };
    let captured = Box::pin(proc::finish(child, Some(prompt), limits)).await;
    captured.map_err(Error::io("run claude"))
}
