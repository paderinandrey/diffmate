use std::future::{Ready, ready};
type Done<T> = Ready<Result<T, Error>>;

use crate::error::Error;
use crate::git::{Git, TreeEntry};
use std::path::Path;
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct FakeGit(pub Mutex<Vec<&'static str>>);

#[allow(refining_impl_trait)]
impl Git for FakeGit {
    fn ensure_clone(&self, _: &Path, _: &str, _: &Path) -> Done<()> {
        self.0.lock().unwrap().push("clone");
        ready(Ok(()))
    }
    fn fetch_pr(&self, _: &Path, _: u64, _: &str) -> Done<()> {
        self.0.lock().unwrap().push("fetch");
        ready(Ok(()))
    }
    fn rev_parse(&self, _: &Path, reference: &str) -> Done<String> {
        ready(Ok(if reference.contains("pull") {
            "new-head"
        } else {
            "new-tip"
        }
        .into()))
    }
    fn merge_base(&self, _: &Path, _: &str, _: &str) -> Done<String> {
        ready(Ok("new-base".into()))
    }
    fn worktree_add(&self, _: &Path, _: &Path, _: &str) -> Done<()> {
        unreachable!()
    }
    fn diff(&self, _: &Path, _: &str, _: &str) -> Done<Vec<u8>> {
        unreachable!()
    }
    fn worktree_remove(&self, _: &Path, _: &Path) -> Done<()> {
        unreachable!()
    }
    fn worktree_prune(&self, _: &Path) -> Done<()> {
        unreachable!()
    }
    fn tree_entry(&self, _: &Path, _: &str, _: &str) -> Done<Option<TreeEntry>> {
        unreachable!()
    }
    fn blob(&self, _: &Path, _: &str, _: usize) -> Done<Vec<u8>> {
        unreachable!()
    }
}
