use std::path::{Path, PathBuf};

pub(super) struct Layout {
    pub(super) worktree: PathBuf,
    pub(super) context: PathBuf,
    pub(super) outside: PathBuf,
    pub(super) prompt: PathBuf,
    pub(super) home: PathBuf,
    pub(super) config: PathBuf,
    pub(super) bin: PathBuf,
    pub(super) scripts: PathBuf,
    pub(super) markers: PathBuf,
    pub(super) hold: PathBuf,
    pub(super) git_log: PathBuf,
}

impl Layout {
    pub(super) fn new(base: &Path) -> Layout {
        let home = base.join("home");
        Layout {
            worktree: base.join("worktree"),
            context: base.join("context"),
            outside: base.join("outside"),
            prompt: base.join("prompt"),
            config: home.join(".claude"),
            home,
            bin: base.join("bin"),
            scripts: base.join("scripts"),
            markers: base.join("markers"),
            hold: base.join("hold"),
            git_log: base.join("git.log"),
        }
    }

    pub(super) fn marker(&self, name: &str) -> PathBuf {
        self.markers.join(name)
    }
}
