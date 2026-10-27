use super::{Upstream, git, git_with, script};
use std::fs;
use std::path::{Path, PathBuf};

const MARKERS: [&str; 5] = ["smudge", "fsmonitor", "extdiff", "textconv", "hook"];

pub(in crate::git) struct Poison {
    pub config: PathBuf,
    pub markers: PathBuf,
    pub fsmonitor: PathBuf,
    pub hooks: PathBuf,
}

fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}

pub(in crate::git) fn poison(root: &Path) -> Poison {
    let (markers, hooks) = (root.join("markers"), root.join("poison-hooks"));
    let scripts = root.join("scripts");
    for dir in [&markers, &hooks, &scripts] {
        fs::create_dir_all(dir).unwrap();
    }
    let config = root.join("poison.gitconfig");
    let set = |key: &str, value: &Path| {
        git(root, &["config", "--file", text(&config), key, text(value)]);
    };
    let marked = |name: &str, body: &str| {
        let path = scripts.join(name);
        script(
            &path,
            &format!("touch '{}/{name}'\n{body}", markers.display()),
        );
        path
    };
    set("filter.canary.smudge", &marked("smudge", "cat"));
    set("diff.canary.command", &marked("extdiff", "exit 0"));
    set("diff.canary.textconv", &marked("textconv", "cat \"$1\""));
    let fsmonitor = marked("fsmonitor", "exit 1");
    set("core.fsmonitor", &fsmonitor);
    script(
        &hooks.join("post-checkout"),
        &format!("touch '{}/hook'", markers.display()),
    );
    set("core.hooksPath", &hooks);
    Poison {
        config,
        markers,
        fsmonitor,
        hooks,
    }
}

pub(in crate::git) fn control(poison: &Poison, root: &Path, up: &Upstream) {
    let (repo, tree) = (root.join("control.git"), root.join("control-tree"));
    let plain = |dir: &Path, args: &[&str]| git_with(dir, &poison.config, args);
    plain(root, &["init", "-q", "--bare", text(&repo)]);
    plain(&repo, &["config", "remote.origin.url", text(&up.path)]);
    let specs = ["+refs/pull/1/head:refs/p/1", "+refs/heads/main:refs/b/main"];
    plain(
        &repo,
        &["fetch", "-q", "--no-tags", "origin", specs[0], specs[1]],
    );
    plain(
        &repo,
        &["worktree", "add", "-q", "--detach", text(&tree), &up.head],
    );
    plain(&tree, &["status"]);
    plain(&tree, &["diff", &up.base, &up.head]);
    plain(&tree, &["diff", "--no-ext-diff", &up.base, &up.head]);
    let present = |marker: &&str| poison.markers.join(marker).exists();
    let missing: Vec<_> = MARKERS.iter().filter(|m| !present(m)).collect();
    assert!(
        missing.is_empty(),
        "control vacuous, no marker for {missing:?}"
    );
    for marker in MARKERS {
        fs::remove_file(poison.markers.join(marker)).unwrap();
    }
}
