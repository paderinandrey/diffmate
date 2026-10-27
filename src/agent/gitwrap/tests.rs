use super::{find_git, install};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn wrapper_forces_git_isolation_variables_and_execs_the_real_git() {
    let tmp = TempDir::new().unwrap();
    let real = tmp.path().join("real git's");
    let body = "#!/bin/sh\necho \"$GIT_CONFIG_GLOBAL $GIT_CONFIG_NOSYSTEM $*\"\n";
    std::fs::write(&real, body).unwrap();
    std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o755)).unwrap();
    let prompt = tmp.path().join("prompt/review.md");

    let bin = install(&prompt, &real).await.unwrap();
    let output = Command::new(bin.join("git"))
        .args(["status", "-s"])
        .env("GIT_CONFIG_GLOBAL", "/home/owner/.gitconfig")
        .env("GIT_CONFIG_NOSYSTEM", "0")
        .output()
        .unwrap();

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "/dev/null 1 status -s\n"
    );
}

#[tokio::test]
async fn install_refuses_to_overwrite_an_existing_wrapper() {
    let tmp = TempDir::new().unwrap();
    let prompt = tmp.path().join("prompt/review.md");
    install(&prompt, &tmp.path().join("git")).await.unwrap();
    assert!(install(&prompt, &tmp.path().join("git")).await.is_err());
}

#[test]
fn find_git_returns_the_first_executable_named_git_on_the_path() {
    let tmp = TempDir::new().unwrap();
    let (first, second) = (tmp.path().join("a"), tmp.path().join("b"));
    for dir in [&first, &second] {
        std::fs::create_dir(dir).unwrap();
        std::fs::write(dir.join("git"), "").unwrap();
    }
    let path = std::env::join_paths([tmp.path().join("none"), first.clone(), second]).unwrap();
    assert_eq!(find_git(&path), Some(first.join("git")));
    assert_eq!(find_git(std::ffi::OsStr::new("")), None);
}
