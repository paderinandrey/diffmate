mod cli;
mod config;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod isolation_tests;
mod refname;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod worktree_tests;

use crate::error::Error;
use crate::procenv::{InheritedEnv, Pairs};
use refname::validate_branch_name;
use std::ffi::OsStr;
use std::future::Future;
use std::path::Path;
use tokio::process::Command;

pub(crate) trait Git {
    fn ensure_clone(
        &self,
        repo: &Path,
        remote_url: &str,
        hooks_dir: &Path,
    ) -> impl Future<Output = Result<(), Error>> + Send;
    fn fetch_pr(
        &self,
        repo: &Path,
        number: u64,
        base_ref: &str,
    ) -> impl Future<Output = Result<(), Error>> + Send;
    fn rev_parse(
        &self,
        repo: &Path,
        rev: &str,
    ) -> impl Future<Output = Result<String, Error>> + Send;
    fn merge_base(
        &self,
        repo: &Path,
        a: &str,
        b: &str,
    ) -> impl Future<Output = Result<String, Error>> + Send;
    fn worktree_add(
        &self,
        repo: &Path,
        worktree: &Path,
        sha: &str,
    ) -> impl Future<Output = Result<(), Error>> + Send;
    fn diff(
        &self,
        repo: &Path,
        base: &str,
        head: &str,
    ) -> impl Future<Output = Result<Vec<u8>, Error>> + Send;
    fn worktree_remove(
        &self,
        repo: &Path,
        worktree: &Path,
    ) -> impl Future<Output = Result<(), Error>> + Send;
    fn worktree_prune(&self, repo: &Path) -> impl Future<Output = Result<(), Error>> + Send;
}

fn os<S: AsRef<OsStr> + ?Sized>(text: &S) -> &OsStr {
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

    fn command<S: AsRef<OsStr>>(
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
