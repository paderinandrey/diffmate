use super::Check;
use super::scratch::Scratch;
use super::which::which;
use crate::agent::AgentRequest;
use crate::agent::claude::{AUTH_ENV, production_args};
use crate::agent::probe::{check_flags, check_version};
use crate::agent::schema::Schemas;
use crate::app::AppConfig;
use std::ffi::{OsStr, OsString};

pub(super) async fn check(config: &AppConfig) -> Check {
    let path = config.env.get("PATH").unwrap_or_default();
    if which(path, "claude").await.is_none() {
        let path = path.to_string_lossy();
        return Check::fail(format!("claude: claude was not found in PATH: {path}"));
    }
    let env = config.env.pick(&AUTH_ENV);
    let program = OsStr::new("claude");
    match check_version(program, &env, None).await {
        Ok(Some(())) => {}
        Ok(None) => return Check::fail("claude: probe did not finish"),
        Err(detail) => return Check::fail(format!("claude: {}", detail.message)),
    }
    let scratch = match Scratch::create().await {
        Ok(scratch) => scratch,
        Err(error) => return Check::fail(format!("claude: {error}")),
    };
    let verdict = flags(program, &env, &scratch).await;
    match scratch.remove().await {
        Ok(()) => verdict,
        Err(error) => verdict.with_extra(&error.to_string()),
    }
}

async fn flags(program: &OsStr, env: &[(OsString, OsString)], scratch: &Scratch) -> Check {
    let schemas = match Schemas::load() {
        Ok(schemas) => schemas,
        Err(error) => return Check::fail(format!("claude: {error}")),
    };
    let request = AgentRequest::new(
        scratch.worktree.clone(),
        scratch.context.clone(),
        scratch.prompt.clone(),
    );
    let args = production_args(&request, &schemas.cli_arg);
    match check_flags(program, env, &args, &scratch.worktree, None).await {
        Ok(Some(())) => Check::pass("claude: ok"),
        Ok(None) => Check::fail("claude: probe did not finish"),
        Err(detail) => Check::fail(format!("claude: {}", detail.message)),
    }
}
