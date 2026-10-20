use super::claude::{ClaudeCli, production_args, user_prompt};
use super::{AgentRequest, gitwrap};
use crate::error::Error;
use crate::proc::{self, Captured, Limits};
use std::process::Stdio;
use tokio::process::Command;

pub(super) async fn invoke(cli: &ClaudeCli, request: &AgentRequest) -> Result<Captured, Error> {
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
    let dirs = std::iter::once(bin).chain(std::env::split_paths(&cli.path));
    let path = std::env::join_paths(dirs).map_err(std::io::Error::other);
    let path = path.map_err(Error::io("build the agent PATH"))?;
    let mut command = Command::new("claude");
    command.args(production_args(request, &cli.schemas.cli_arg));
    command.current_dir(&request.worktree).env_clear();
    command.envs(cli.env.iter().map(|(name, value)| (name, value)));
    command.env("PATH", path);
    command.env("GIT_CONFIG_GLOBAL", "/dev/null");
    command.env("GIT_CONFIG_NOSYSTEM", "1");
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
