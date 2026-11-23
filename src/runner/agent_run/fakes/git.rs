use super::Done;
use crate::error::Error;
use crate::git::{Git, TreeEntry};
use std::future::ready;
use std::path::Path;

#[derive(Default)]
pub(crate) struct FakeGit {
    pub preparation: std::time::Duration,
    pulls: std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>,
}

#[allow(refining_impl_trait)]
impl Git for FakeGit {
    fn ensure_clone(&self, _: &Path, _: &str, _: &Path) -> Done<()> {
        ready(Ok(()))
    }
    fn fetch_pr(&self, repo: &Path, number: u64, _: &str) -> Done<()> {
        self.pulls
            .lock()
            .unwrap()
            .insert(repo.join(format!("refs/diffmate/pull/{number}/head")));
        ready(Ok(()))
    }
    fn delete_pull_refs(&self, repo: &Path, number: u64) -> Done<()> {
        self.pulls
            .lock()
            .unwrap()
            .remove(&repo.join(format!("refs/diffmate/pull/{number}/head")));
        ready(Ok(()))
    }
    fn rev_parse(&self, repo: &Path, reference: &str) -> Done<String> {
        if reference.starts_with("refs/diffmate/pull/")
            && !self.pulls.lock().unwrap().contains(&repo.join(reference))
        {
            return ready(Err(Error::Missing(repo.join(reference))));
        }
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
