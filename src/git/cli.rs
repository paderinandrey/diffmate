use super::{Git, GitCli, config, os, validate_branch_name};
use crate::error::Error;
use crate::paths::create_private_dir_async;
use crate::proc::run_tool;
use std::path::Path;
use tokio::process::Command;

impl GitCli {
    pub(super) async fn run(command: Command) -> Result<(), Error> {
        Box::pin(run_tool("git", command)).await.map(drop)
    }

    pub(super) async fn text(command: Command) -> Result<String, Error> {
        let stdout = Box::pin(run_tool("git", command)).await?;
        Ok(String::from_utf8_lossy(&stdout).trim().to_string())
    }
}

impl Git for GitCli {
    async fn ensure_clone(
        &self,
        repo: &Path,
        remote_url: &str,
        hooks_dir: &Path,
    ) -> Result<(), Error> {
        for dir in [repo.parent().unwrap_or(repo), hooks_dir] {
            let made = create_private_dir_async(dir.to_path_buf()).await;
            made.map_err(Error::io("create git directory"))?;
        }
        if !tokio::fs::try_exists(repo)
            .await
            .map_err(Error::io("check clone"))?
        {
            let init = self.command(
                None,
                [os("init"), os("--bare"), os("--template="), os(repo)],
            );
            GitCli::run(init).await?;
        }
        config::write(self, repo, remote_url, hooks_dir).await
    }

    async fn fetch_pr(&self, repo: &Path, number: u64, base_ref: &str) -> Result<(), Error> {
        validate_branch_name(base_ref)?;
        let helper = "credential.helper=!gh auth git-credential";
        let mut command = self.command(Some(repo), ["-c", "credential.helper=", "-c", helper]);
        command.args(["fetch", "--no-tags", "--no-recurse-submodules", "origin"]);
        command.arg(format!(
            "+refs/pull/{number}/head:refs/diffmate/pull/{number}/head"
        ));
        command.arg(format!(
            "+refs/heads/{base_ref}:refs/diffmate/base/{base_ref}"
        ));
        GitCli::run(command).await
    }

    async fn rev_parse(&self, repo: &Path, rev: &str) -> Result<String, Error> {
        let rev = format!("{rev}^{{commit}}");
        GitCli::text(self.command(Some(repo), ["rev-parse", "--verify", &rev])).await
    }

    async fn merge_base(&self, repo: &Path, a: &str, b: &str) -> Result<String, Error> {
        GitCli::text(self.command(Some(repo), ["merge-base", a, b])).await
    }

    async fn worktree_add(&self, repo: &Path, worktree: &Path, sha: &str) -> Result<(), Error> {
        let args = [
            os("worktree"),
            os("add"),
            os("--detach"),
            os(worktree),
            os(sha),
        ];
        GitCli::run(self.command(Some(repo), args)).await
    }

    async fn diff(&self, repo: &Path, base: &str, head: &str) -> Result<Vec<u8>, Error> {
        let args = ["diff", "--no-ext-diff", "--no-textconv", base, head];
        Box::pin(run_tool("git", self.command(Some(repo), args))).await
    }

    async fn worktree_remove(&self, repo: &Path, worktree: &Path) -> Result<(), Error> {
        let args = [os("worktree"), os("remove"), os("--force"), os(worktree)];
        GitCli::run(self.command(Some(repo), args)).await
    }

    async fn worktree_prune(&self, repo: &Path) -> Result<(), Error> {
        GitCli::run(self.command(Some(repo), ["worktree", "prune"])).await
    }
}
