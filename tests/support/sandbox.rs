use super::fake_claude::FakeClaude;
use super::fake_gh::FakeGh;
use super::upstream::{Upstream, real_git};
use super::write_executable;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

pub struct Sandbox {
    pub root: PathBuf,
    pub home: PathBuf,
    pub bin: PathBuf,
    pub upstreams: PathBuf,
    pub claude: FakeClaude,
    pub gh: FakeGh,
    _tmp: TempDir,
}

impl Sandbox {
    pub fn new() -> Sandbox {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let make = |name: &str| {
            let dir = root.join(name);
            fs::create_dir(&dir).unwrap();
            dir
        };
        let (home, bin, upstreams) = (make("home"), make("bin"), make("upstreams"));
        let claude = FakeClaude::install(&bin, &make("claude-state"));
        let gh = FakeGh::install(&bin, &make("gh-state"));
        let url = format!("file://{}/", upstreams.display());
        let shim = format!(
            "#!/bin/sh\nexec '{}' -c 'url.{url}.insteadOf=https://github.com/' \"$@\"\n",
            real_git().display()
        );
        write_executable(&bin.join("git"), &shim);
        Sandbox {
            root,
            home,
            bin,
            upstreams,
            claude,
            gh,
            _tmp: tmp,
        }
    }

    pub fn state_dir(&self) -> PathBuf {
        self.home.join("Library/Application Support/diffmate")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.home.join("Library/Caches/diffmate")
    }

    pub fn db_path(&self) -> PathBuf {
        self.state_dir().join("diffmate.db")
    }

    pub fn upstream(&self, owner: &str, repo: &str) -> Upstream {
        Upstream::create(&self.upstreams, owner, repo)
    }

    pub fn diffmate(&self) -> Command {
        let path = std::env::join_paths([self.bin.as_path(), "/usr/bin".as_ref(), "/bin".as_ref()]);
        let mut command = Command::new(env!("CARGO_BIN_EXE_diffmate"));
        command
            .current_dir(&self.root)
            .env("HOME", &self.home)
            .env("PATH", path.unwrap());
        command.env_remove("XDG_CONFIG_HOME");
        for (name, _) in std::env::vars_os() {
            let lossy = name.to_string_lossy();
            let inherited = ["GIT_", "CLAUDE", "ANTHROPIC_", "GH_", "GITHUB_"];
            if inherited.iter().any(|prefix| lossy.starts_with(prefix)) {
                command.env_remove(&name);
            }
        }
        command
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.diffmate()
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
}
