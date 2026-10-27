use super::{MaliciousPr, files};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

impl MaliciousPr {
    pub fn canary(&self, name: &str) -> &str {
        &self
            .canaries
            .iter()
            .find(|(have, _)| *have == name)
            .unwrap()
            .1
    }

    pub fn fired(&self) -> Vec<&'static str> {
        let present = |(_, path): &&(&str, PathBuf)| path.exists();
        self.markers
            .iter()
            .filter(present)
            .map(|(name, _)| *name)
            .collect()
    }

    pub fn prompt_for(&self, label: &str) -> PathBuf {
        let n = self.runs.fetch_add(1, Ordering::SeqCst);
        let dir = self.root.path().join(format!("prompt-{n}-{label}"));
        let copy = dir.join("review.md");
        files::write(&copy, &fs::read_to_string(&self.prompt_file).unwrap());
        copy
    }

    pub fn reset_traces(&self) {
        for (_, path) in &self.markers {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_file(&self.git_log);
        files::write(&self.worktree.join("app.py"), "print('hi')\n");
    }

    pub fn set_claude_md(&self, present: bool) {
        let (here, held) = (self.worktree.join("CLAUDE.md"), self.hold.join("CLAUDE.md"));
        let (from, to) = if present { (held, here) } else { (here, held) };
        fs::rename(from, to).unwrap();
    }
}
