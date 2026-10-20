mod build;
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
mod tree;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod worktree_tests;

use crate::error::Error;
use build::os;
use refname::validate_branch_name;
use std::future::Future;
use std::path::Path;

pub(crate) use build::GitCli;
pub(crate) use tree::TreeEntry;

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
    fn tree_entry(
        &self,
        repo: &Path,
        commit: &str,
        path: &str,
    ) -> impl Future<Output = Result<Option<TreeEntry>, Error>> + Send;
    fn blob(
        &self,
        repo: &Path,
        oid: &str,
        cap: usize,
    ) -> impl Future<Output = Result<Vec<u8>, Error>> + Send;
}
