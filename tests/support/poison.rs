use super::git_in;
use super::upstream::real_git;
use super::write_executable;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MARKERS: [&str; 5] = ["smudge", "fsmonitor", "extdiff", "textconv", "hook"];

pub struct Poison {
    pub config: PathBuf,
    pub markers: PathBuf,
    pub fsmonitor: PathBuf,
    pub hooks: PathBuf,
}

fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}

impl Poison {
    pub fn write(root: &Path) -> Poison {
        let (markers, hooks) = (root.join("markers"), root.join("poison-hooks"));
        let scripts = root.join("scripts");
        for dir in [&markers, &hooks, &scripts] {
            fs::create_dir_all(dir).unwrap();
        }
        let config = root.join("poison.gitconfig");
        let set = |key: &str, value: &Path| {
            let args = ["config", "--file", text(&config), key, text(value)];
            git_in(root, root, &args);
        };
        let marked = |name: &str, body: &str| {
            let path = scripts.join(name);
            let touch = format!("#!/bin/sh\ntouch '{}/{name}'\n{body}\n", markers.display());
            write_executable(&path, &touch);
            path
        };
        set("filter.canary.smudge", &marked("smudge", "cat"));
        set("diff.canary.command", &marked("extdiff", "exit 0"));
        set("diff.canary.textconv", &marked("textconv", "cat \"$1\""));
        let fsmonitor = marked("fsmonitor", "exit 1");
        set("core.fsmonitor", &fsmonitor);
        let hook = format!("#!/bin/sh\ntouch '{}/hook'\n", markers.display());
        write_executable(&hooks.join("post-checkout"), &hook);
        set("core.hooksPath", &hooks);
        Poison {
            config,
            markers,
            fsmonitor,
            hooks,
        }
    }

    pub fn clear(&self) {
        for marker in MARKERS {
            let _ = fs::remove_file(self.markers.join(marker));
        }
    }

    pub fn control(&self, root: &Path, origin: &Path, (base, head): (&str, &str)) {
        let (repo, tree) = (root.join("control.git"), root.join("control-tree"));
        let plain = |dir: &Path, args: &[&str]| {
            let mut command = Command::new(real_git());
            command.args(args).current_dir(dir).env_clear();
            command.env("PATH", std::env::var_os("PATH").unwrap());
            command.env("HOME", root).env("LC_ALL", "C");
            command.env("GIT_CONFIG_GLOBAL", &self.config);
            command.env("GIT_CONFIG_NOSYSTEM", "1");
            let output = command.output().unwrap();
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(output.status.success(), "git {args:?}: {stderr}");
        };
        plain(root, &["init", "-q", "--bare", text(&repo)]);
        plain(&repo, &["config", "remote.origin.url", text(origin)]);
        let specs = ["+refs/pull/1/head:refs/p/1", "+refs/heads/main:refs/b/main"];
        plain(
            &repo,
            &["fetch", "-q", "--no-tags", "origin", specs[0], specs[1]],
        );
        plain(
            &repo,
            &["worktree", "add", "-q", "--detach", text(&tree), head],
        );
        plain(&tree, &["status"]);
        plain(&tree, &["diff", base, head]);
        plain(&tree, &["diff", "--no-ext-diff", base, head]);
        let missing: Vec<_> = MARKERS
            .iter()
            .filter(|marker| !self.markers.join(marker).exists())
            .collect();
        assert!(
            missing.is_empty(),
            "control vacuous, no marker for {missing:?}"
        );
        self.clear();
    }
}
