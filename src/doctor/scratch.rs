use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct Scratch {
    root: Option<PathBuf>,
    pub(super) worktree: PathBuf,
    pub(super) context: PathBuf,
    pub(super) prompt: PathBuf,
}

impl Scratch {
    pub(super) async fn create() -> io::Result<Self> {
        tokio::task::spawn_blocking(build)
            .await
            .map_err(io::Error::other)?
    }

    pub(super) async fn remove(mut self) -> io::Result<()> {
        let Some(root) = self.root.take() else {
            return Ok(());
        };
        tokio::task::spawn_blocking(move || std::fs::remove_dir_all(root))
            .await
            .map_err(io::Error::other)?
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Some(root) = self.root.take() {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}

fn build() -> io::Result<Scratch> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let name = format!("diffmate-doctor-{}-{nanos}", std::process::id());
    let root = std::env::temp_dir().join(name);
    std::fs::create_dir(&root)?;
    match layout(&root) {
        Ok(scratch) => Ok(scratch),
        Err(error) => {
            let _ = std::fs::remove_dir_all(&root);
            Err(error)
        }
    }
}

fn layout(root: &Path) -> io::Result<Scratch> {
    let worktree = root.join("worktree");
    let context = root.join("context");
    let prompt = root.join("review.md");
    std::fs::create_dir(&worktree)?;
    std::fs::create_dir(&context)?;
    std::fs::write(&prompt, "# Review rules\n")?;
    Ok(Scratch {
        root: Some(root.to_path_buf()),
        worktree,
        context,
        prompt,
    })
}
