use super::claude::ClaudeCli;
use super::probe::{self, Refusal};
use super::{AgentRequest, FailureDetail, gitwrap, supervisor};
use crate::error::Error;
use crate::proc::{Captured, Limits};
use crate::procenv::Pairs;
use std::ffi::OsStr;

const PROGRAM: &str = "grok";

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
    let Some(()) = probe::check_version(program, &env, Some(request)).await? else {
        return Ok(None);
    };
    let Some(()) = probe::check_flags(program, &env, Some(request)).await? else {
        return Ok(None);
    };
    let rules = tokio::fs::read_to_string(&request.prompt_file).await;
    let rules = rules
        .map_err(Error::io("read the review prompt"))
        .map_err(|e| refused(&e))?;
    let args = cli.args_for(request, &rules);
    let limits = Limits {
        stdout: 8 << 20,
        stderr: 64 << 10,
    };
    supervisor::capture(program, &args, &env, Some(request), true, limits)
        .await
        .map_err(Error::io("run grok"))
        .map_err(|e| refused(&e))
}
