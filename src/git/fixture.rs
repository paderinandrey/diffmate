pub(super) mod poison;

use super::GitCli;
use crate::procenv::InheritedEnv;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) struct Upstream {
    pub path: PathBuf,
    pub base: String,
    pub head: String,
}

pub(super) fn path_var() -> std::ffi::OsString {
    std::env::var_os("PATH").unwrap()
}

pub(super) fn cli(home: &Path) -> GitCli {
    let pairs = vec![("PATH".into(), path_var()), ("HOME".into(), home.into())];
    GitCli::new(&InheritedEnv::from_pairs(pairs))
}

pub(super) fn git_with(dir: &Path, global: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    command.args(args).current_dir(dir).env_clear();
    command.env("PATH", path_var()).env("HOME", dir);
    command.env("GIT_CONFIG_GLOBAL", global);
    command.env("GIT_CONFIG_NOSYSTEM", "1");
    for who in ["AUTHOR", "COMMITTER"] {
        command.env(format!("GIT_{who}_NAME"), "Fixture");
        command.env(format!("GIT_{who}_EMAIL"), "fixture@example.com");
    }
    let output = command.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "git {args:?}: {stderr}");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

pub(super) fn git(dir: &Path, args: &[&str]) -> String {
    git_with(dir, Path::new("/dev/null"), args)
}

pub(super) fn script(path: &Path, body: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(format!("#!/bin/sh\n{body}\n").as_bytes())
        .unwrap();
    file.sync_all().unwrap();
    drop(file);
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

pub(super) fn upstream(root: &Path) -> Upstream {
    let (origin, work) = (root.join("up.git"), root.join("up-work"));
    for dir in [&origin, &work] {
        fs::create_dir_all(dir).unwrap();
    }
    git(&origin, &["init", "-q", "--bare", "-b", "main"]);
    git(&work, &["init", "-q", "-b", "main"]);
    let commit = |message: &str, refname: &str| {
        git(&work, &["add", "-A"]);
        git(&work, &["commit", "-q", "-m", message]);
        let target = format!("HEAD:{refname}");
        git(&work, &["push", "-q", origin.to_str().unwrap(), &target]);
        git(&work, &["rev-parse", "HEAD"])
    };
    fs::write(work.join("a.txt"), "one\n").unwrap();
    let base = commit("base", "refs/heads/main");
    fs::write(work.join("a.txt"), "two\n").unwrap();
    fs::write(work.join(".gitattributes"), "* filter=canary diff=canary\n").unwrap();
    symlink("../../outside", work.join("link")).unwrap();
    let head = commit("pr", "refs/pull/1/head");
    Upstream {
        path: origin,
        base,
        head,
    }
}
