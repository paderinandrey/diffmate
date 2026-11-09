use super::Done;
use crate::error::Error;
use crate::git::{Git, TreeEntry};
use std::future::ready;
use std::path::Path;

#[derive(Default)]
pub(crate) struct FakeGit {
    pub preparation: std::time::Duration,
}

#[allow(refining_impl_trait)]
impl Git for FakeGit {
    fn ensure_clone(&self, _: &Path, _: &str, _: &Path) -> Done<()> {
        ready(Ok(()))
    }
    fn fetch_pr(&self, _: &Path, _: u64, _: &str) -> Done<()> {
        ready(Ok(()))
    }
    fn rev_parse(&self, _: &Path, reference: &str) -> Done<String> {
        ready(Ok(
            if reference.contains("pull") { "h" } else { "b" }.repeat(40)
        ))
    }
    fn merge_base(&self, _: &Path, _: &str, _: &str) -> Done<String> {
        ready(Ok("b".repeat(40)))
    }
    async fn worktree_add(&self, _: &Path, worktree: &Path, _: &str) -> Result<(), Error> {
        tokio::fs::create_dir_all(worktree)
            .await
            .map_err(Error::io("create fake worktree"))
    }
    async fn diff(&self, _: &Path, _: &str, _: &str) -> Result<Vec<u8>, Error> {
        if !self.preparation.is_zero() {
            tokio::time::sleep(self.preparation).await;
        }
        Ok(Vec::new())
    }
    async fn worktree_remove(&self, _: &Path, worktree: &Path) -> Result<(), Error> {
        tokio::fs::remove_dir_all(worktree)
            .await
            .map_err(Error::io("remove fake worktree"))
    }
    fn worktree_prune(&self, _: &Path) -> Done<()> {
        ready(Ok(()))
    }
    fn tree_entry(&self, _: &Path, _: &str, _: &str) -> Done<Option<TreeEntry>> {
        ready(Ok(None))
    }
    fn blob(&self, _: &Path, _: &str, _: usize) -> Done<Vec<u8>> {
        unreachable!()
    }
}
