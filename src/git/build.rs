use crate::error::Error;
use crate::proc::run_tool;
use crate::procenv::{InheritedEnv, Pairs};
use std::ffi::OsStr;
use std::path::Path;
use tokio::process::Command;

pub(super) fn os<S: AsRef<OsStr> + ?Sized>(text: &S) -> &OsStr {
    text.as_ref()
}

pub(crate) struct GitCli {
    env: Pairs,
}

impl GitCli {
    pub fn new(env: &InheritedEnv) -> GitCli {
        let mut vars = env.tool_env();
        vars.push(("GIT_TERMINAL_PROMPT".into(), "0".into()));
        vars.push(("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()));
        vars.push(("GIT_CONFIG_NOSYSTEM".into(), "1".into()));
        GitCli { env: vars }
    }

    pub(super) fn command<S: AsRef<OsStr>>(
        &self,
        repo: Option<&Path>,
        args: impl IntoIterator<Item = S>,
    ) -> Command {
        let mut command = Command::new("git");
        command
            .env_clear()
            .envs(self.env.iter().map(|(name, value)| (name, value)));
        if let Some(repo) = repo {
            command.arg("-C").arg(repo);
        }
        command.args(args);
        command
    }
}

impl GitCli {
    pub(super) async fn run(command: Command) -> Result<(), Error> {
        Box::pin(run_tool("git", command)).await.map(drop)
    }

    pub(super) async fn text(command: Command) -> Result<String, Error> {
        let stdout = Box::pin(run_tool("git", command)).await?;
        Ok(String::from_utf8_lossy(&stdout).trim().to_string())
    }
}
