use crate::store::RunId;
use std::fs::DirBuilder;
use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct Paths {
    state_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Paths {
    pub fn new(state_dir: PathBuf, cache_dir: PathBuf) -> Paths {
        Paths {
            state_dir,
            cache_dir,
        }
    }

    pub fn from_home(home: &Path) -> Paths {
        Paths::new(
            home.join("Library/Application Support/diffmate"),
            home.join("Library/Caches/diffmate"),
        )
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn db(&self) -> PathBuf {
        self.state_dir.join("diffmate.db")
    }

    pub fn lock(&self) -> PathBuf {
        self.state_dir.join("runner.lock")
    }

    pub fn repos_dir(&self) -> PathBuf {
        self.state_dir.join("repos")
    }

    pub fn repo_dir(&self, owner: &str, repo: &str) -> PathBuf {
        let name = format!("{repo}.git");
        self.repos_dir().join(owner).join(name)
    }

    pub fn empty_hooks_dir(&self) -> PathBuf {
        self.state_dir.join("empty-hooks")
    }

    pub fn runs_dir(&self) -> PathBuf {
        self.cache_dir.join("runs")
    }

    pub fn run_dir(&self, run: RunId) -> PathBuf {
        self.runs_dir().join(run.to_string())
    }

    pub fn worktree(&self, run: RunId) -> PathBuf {
        self.run_dir(run).join("worktree")
    }

    pub fn context_dir(&self, run: RunId) -> PathBuf {
        self.run_dir(run).join("context")
    }

    pub fn prompt_file(&self, run: RunId) -> PathBuf {
        self.run_dir(run).join("prompt/review.md")
    }
}

pub(crate) fn create_private_dir(path: &Path) -> io::Result<()> {
    DirBuilder::new().recursive(true).mode(0o700).create(path)
}

pub(crate) async fn create_private_dir_async(path: PathBuf) -> io::Result<()> {
    tokio::task::spawn_blocking(move || create_private_dir(&path))
        .await
        .map_err(io::Error::other)?
}
