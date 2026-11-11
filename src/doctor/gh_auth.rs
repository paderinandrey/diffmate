use super::Check;
use super::which::which;
use crate::app::AppConfig;
use crate::gh::GhCli;
use crate::proc::run_tool;

pub(super) async fn check(config: &AppConfig) -> Check {
    let path = config.env.get("PATH").unwrap_or_default();
    if which(path, "gh").await.is_none() {
        let path = path.to_string_lossy();
        return Check::fail(format!("gh auth: gh was not found in PATH: {path}"));
    }
    let command = GhCli::new(&config.env).command(["auth", "status"]);
    match run_tool("gh", command).await {
        Ok(_) => Check::pass("gh auth: ok"),
        Err(error) => Check::fail(format!("gh auth: {error}; run gh auth login")),
    }
}
