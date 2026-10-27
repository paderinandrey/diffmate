use super::{Git, GitCli, TreeEntry, config, nested_refs, os, tree, validate_branch_name};
use crate::error::Error;
use crate::paths::create_private_dir_async;
use crate::proc::run_tool;
use std::path::Path;

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
        let base = format!("refs/diffmate/base/{base_ref}");
        let list = ["for-each-ref", "--format=%(refname)", "refs/diffmate/base/"];
        let listed = GitCli::text(self.command(Some(repo), list)).await?;
        for stale in listed.lines().filter(|have| nested_refs(have, &base)) {
            GitCli::run(self.command(Some(repo), ["update-ref", "-d", stale])).await?;
        }
        let helper = "credential.helper=!gh auth git-credential";
        let mut command = self.command(Some(repo), ["-c", "credential.helper=", "-c", helper]);
        command.args(["fetch", "--no-tags", "--no-recurse-submodules", "origin"]);
        command.arg(format!(
            "+refs/pull/{number}/head:refs/diffmate/pull/{number}/head"
        ));
        command.arg(format!("+refs/heads/{base_ref}:{base}"));
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

    async fn tree_entry(
        &self,
        repo: &Path,
        commit: &str,
        path: &str,
    ) -> Result<Option<TreeEntry>, Error> {
        tree::entry(self, repo, commit, path).await
    }

    async fn blob(&self, repo: &Path, oid: &str, cap: usize) -> Result<Vec<u8>, Error> {
        tree::blob(self, repo, oid, cap).await
    }
}
