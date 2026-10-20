use super::contract::contract;
use super::fixture::MaliciousPr;
use std::path::Path;
use std::process::Command;

fn git_status(pr: &MaliciousPr, global: Option<&Path>) {
    let mut command = Command::new("git");
    command
        .args(["status", "--short"])
        .current_dir(&pr.worktree)
        .env_clear();
    command
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", &pr.home);
    command.env("GIT_CONFIG_NOSYSTEM", "1");
    if let Some(global) = global {
        command.env("GIT_CONFIG_GLOBAL", global);
    }
    assert!(command.output().unwrap().status.success());
}

#[test]
fn canaries_are_distinct_ascii_tokens_and_markers_are_inside_the_temp_dir() {
    let pr = MaliciousPr::build("http://127.0.0.1:9");
    let values: Vec<&str> = pr
        .canaries
        .iter()
        .map(|(_, value)| value.as_str())
        .collect();
    for value in &values {
        assert!(
            value.starts_with("dmcanary") && value.len() == 24,
            "{value}"
        );
        assert!(value.bytes().all(|byte| byte.is_ascii_alphanumeric()));
    }
    let unique: std::collections::BTreeSet<_> = values.iter().collect();
    assert_eq!(unique.len(), values.len());
    let base = pr.root.path().canonicalize().unwrap();
    assert!(pr.markers.iter().all(|(_, path)| path.starts_with(&base)));
    assert_eq!(pr.fired(), Vec::<&str>::new());
}

#[test]
fn the_worktree_is_a_one_commit_repository_with_the_planted_files() {
    let pr = MaliciousPr::build("http://127.0.0.1:9");
    let output = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(&pr.worktree)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "1");
    let claude_md = std::fs::read_to_string(pr.worktree.join("CLAUDE.md")).unwrap();
    assert!(claude_md.contains(pr.canary("claude_md")));
    let settings = std::fs::read_to_string(pr.worktree.join(".claude/settings.json")).unwrap();
    assert!(settings.contains("http://127.0.0.1:9"));
    let state = std::fs::read_to_string(pr.config_dir.join(".claude.json")).unwrap();
    for key in contract()["controls"]["trust_keys"].as_array().unwrap() {
        let name = key
            .as_str()
            .unwrap()
            .strip_prefix("projects[path].")
            .unwrap();
        assert!(state.contains(name), "{name}");
    }
}

#[test]
fn the_global_filter_fires_with_plain_git_and_not_with_a_null_global_config() {
    let pr = MaliciousPr::build("http://127.0.0.1:9");
    pr.reset_traces();
    git_status(&pr, Some(Path::new("/dev/null")));
    assert_eq!(pr.fired(), Vec::<&str>::new());
    pr.reset_traces();
    git_status(&pr, None);
    assert_eq!(pr.fired(), ["git_filter"], "control vacuous");
}

#[test]
fn claude_md_can_be_hidden_and_restored() {
    let pr = MaliciousPr::build("http://127.0.0.1:9");
    pr.set_claude_md(false);
    assert!(!pr.worktree.join("CLAUDE.md").exists());
    pr.set_claude_md(true);
    assert!(pr.worktree.join("CLAUDE.md").exists());
}
