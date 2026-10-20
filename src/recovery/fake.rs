use crate::error::Error;
use crate::git::Git;
use std::future::{Future, ready};
use std::path::Path;
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct FakeGit {
    pub calls: Mutex<Vec<String>>,
}

type Done<T> = std::future::Ready<Result<T, Error>>;

impl FakeGit {
    fn note<T>(&self, call: String, value: T) -> Done<T> {
        self.calls.lock().unwrap().push(call);
        ready(Ok(value))
    }
}

impl Git for FakeGit {
    fn ensure_clone(
        &self,
        _: &Path,
        _: &str,
        _: &Path,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        self.note("ensure_clone".into(), ())
    }

    fn fetch_pr(
        &self,
        _: &Path,
        _: u64,
        _: &str,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        self.note("fetch_pr".into(), ())
    }

    fn rev_parse(&self, _: &Path, _: &str) -> impl Future<Output = Result<String, Error>> + Send {
        self.note("rev_parse".into(), String::new())
    }

    fn merge_base(
        &self,
        _: &Path,
        _: &str,
        _: &str,
    ) -> impl Future<Output = Result<String, Error>> + Send {
        self.note("merge_base".into(), String::new())
    }

    fn worktree_add(
        &self,
        _: &Path,
        _: &Path,
        _: &str,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        self.note("worktree_add".into(), ())
    }

    fn diff(
        &self,
        _: &Path,
        _: &str,
        _: &str,
    ) -> impl Future<Output = Result<Vec<u8>, Error>> + Send {
        self.note("diff".into(), Vec::new())
    }

    fn worktree_remove(
        &self,
        repo: &Path,
        worktree: &Path,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        self.note(
            format!("remove {} {}", repo.display(), worktree.display()),
            (),
        )
    }

    fn worktree_prune(&self, repo: &Path) -> impl Future<Output = Result<(), Error>> + Send {
        self.note(format!("prune {}", repo.display()), ())
    }
}
