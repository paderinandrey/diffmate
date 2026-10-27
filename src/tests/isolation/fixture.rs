mod context;
mod files;
mod home;
mod layout;
mod ops;
mod scripts;
mod worktree;

use layout::Layout;
use std::fs;
use std::hash::{BuildHasher, RandomState};
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use tempfile::TempDir;

const CANARIES: [&str; 6] = [
    "claude_md",
    "import",
    "agents_md",
    "agents_commands_skills",
    "context_root_claude_md",
    "context_rules_claude_md",
];
const MARKERS: [&str; 5] = [
    "session_start_hook",
    "api_key_helper",
    "plugin_hook",
    "mcp_server",
    "git_filter",
];

pub(super) struct MaliciousPr {
    pub root: TempDir,
    pub worktree: PathBuf,
    pub context_dir: PathBuf,
    pub prompt_file: PathBuf,
    pub home: PathBuf,
    pub config_dir: PathBuf,
    pub bin: PathBuf,
    pub canaries: Vec<(&'static str, String)>,
    pub markers: Vec<(&'static str, PathBuf)>,
    pub git_log: PathBuf,
    hold: PathBuf,
    runs: AtomicUsize,
}

impl MaliciousPr {
    pub fn build(secondary_listener_url: &str) -> MaliciousPr {
        let root = tempfile::tempdir().unwrap();
        let layout = Layout::new(&root.path().canonicalize().unwrap());
        for dir in [&layout.config, &layout.bin, &layout.markers, &layout.hold] {
            fs::create_dir_all(dir).unwrap();
        }
        let hash = |name: &str| format!("dmcanary{:016x}", RandomState::new().hash_one(name));
        let canaries: Vec<_> = CANARIES.iter().map(|name| (*name, hash(name))).collect();
        scripts::write(&layout);
        worktree::write_all(&layout, secondary_listener_url, &canaries);
        home::write_all(&layout);
        context::write_all(&layout, &canaries);
        let markers = MARKERS
            .iter()
            .map(|name| (*name, layout.marker(name)))
            .collect();
        MaliciousPr {
            root,
            prompt_file: layout.prompt.join("review.md"),
            worktree: layout.worktree,
            context_dir: layout.context,
            home: layout.home,
            config_dir: layout.config,
            bin: layout.bin,
            canaries,
            markers,
            git_log: layout.git_log,
            hold: layout.hold,
            runs: AtomicUsize::new(0),
        }
    }
}
