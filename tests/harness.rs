mod support;

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use support::fake_claude::{DEFAULT_VERSION, FakeClaude, PROBE_OK};
use support::fake_gh::FakeGh;
use support::sandbox::Sandbox;
use support::upstream::real_git;
use support::{exec, signal};
use tempfile::TempDir;

fn bin_dir(tmp: &TempDir) -> std::path::PathBuf {
    let bin = tmp.path().join("bin");
    fs::create_dir(&bin).unwrap();
    bin
}

#[test]
fn fake_claude_answers_version_probe_and_run() {
    let tmp = TempDir::new().unwrap();
    let bin = bin_dir(&tmp);
    let claude = FakeClaude::install(&bin, &tmp.path().join("state"));
    let context = tmp.path().join("context");
    fs::create_dir(&context).unwrap();
    fs::write(context.join("pr.md"), "pr").unwrap();
    let program = bin.join("claude");

    let version = exec(&program, &["--version"], "");
    assert_eq!(
        String::from_utf8_lossy(&version.stdout),
        format!("{DEFAULT_VERSION}\n")
    );
    let probe = exec(&program, &["-p"], "");
    assert_eq!(probe.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&probe.stderr).contains(PROBE_OK));

    claude.set_run("{\"x\":1}", 3);
    let args = ["-p", "--add-dir", context.to_str().unwrap()];
    let run = exec(&program, &args, "prompt");
    assert_eq!(run.status.code(), Some(3));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "{\"x\":1}");

    assert_eq!(claude.calls().len(), 3);
    let runs = claude.run_calls();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].stdin, b"prompt");
    assert_eq!(runs[0].argv, args);
    assert_eq!(
        fs::read_to_string(runs[0].context.as_ref().unwrap().join("pr.md")).unwrap(),
        "pr"
    );
    assert!(runs[0].pid.is_some() && runs[0].child_pid.is_none());
}

#[test]
fn fake_claude_holds_run_and_spawns_child() {
    let tmp = TempDir::new().unwrap();
    let bin = bin_dir(&tmp);
    let claude = FakeClaude::install(&bin, &tmp.path().join("state"));
    claude.spawn_child();
    let hold = claude.hold_run(1);
    let mut child = Command::new(bin.join("claude"))
        .arg("-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"prompt").unwrap();

    hold.wait_started();
    let background = claude.run_calls()[0].child_pid.unwrap();
    assert!(signal("-0", background));
    hold.release();
    assert!(child.wait().unwrap().success());
    assert!(signal("-9", background));
}

#[test]
fn fake_gh_routes_by_prefix() {
    let tmp = TempDir::new().unwrap();
    let bin = bin_dir(&tmp);
    let gh = FakeGh::install(&bin, &tmp.path().join("gh-state"));
    gh.route("pr view", "{\"number\":1}\n");
    gh.route_fail("pr diff", "boom", 4);
    let program = bin.join("gh");

    let view = exec(&program, &["pr", "view", "u", "--json", "x"], "");
    assert!(view.status.success());
    assert_eq!(String::from_utf8_lossy(&view.stdout), "{\"number\":1}\n");
    let diff = exec(&program, &["pr", "diff", "u"], "");
    assert_eq!(diff.status.code(), Some(4));
    assert_eq!(String::from_utf8_lossy(&diff.stderr), "boom");
    let unknown = exec(&program, &["repo", "view"], "");
    assert_eq!(unknown.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("fake gh: no route for: repo view"));
    assert_eq!(gh.calls()[0], ["pr", "view", "u", "--json", "x"]);
    assert_eq!(gh.calls().len(), 3);
}

#[test]
fn upstream_pr_head_exists_only_under_pull_ref() {
    let sandbox = Sandbox::new();
    let upstream = sandbox.upstream("o", "r");
    upstream.commit(&[("a.txt", "base\n")], "base");
    upstream.push_branch("main");
    let head = upstream.pr_head(1, "main", &[("b.txt", "head\n")]);

    let refs = upstream.git(&["ls-remote", upstream.bare.to_str().unwrap()]);
    assert!(refs.contains("refs/heads/main") && refs.contains("refs/pull/1/head"));
    assert!(!refs.contains("refs/heads/pr"));
    let ancestor = [
        "--git-dir",
        upstream.bare.to_str().unwrap(),
        "merge-base",
        "--is-ancestor",
        &head,
        "main",
    ];
    let status = Command::new(real_git()).args(ancestor).status().unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn git_shim_resolves_github_remote() {
    let sandbox = Sandbox::new();
    let upstream = sandbox.upstream("o", "r");
    upstream.commit(&[("a.txt", "base\n")], "base");
    upstream.push_branch("main");
    upstream.pr_head(1, "main", &[("b.txt", "head\n")]);

    let output = Command::new(sandbox.bin.join("git"))
        .args(["ls-remote", "https://github.com/o/r.git"])
        .env_clear()
        .env("HOME", &sandbox.home)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("refs/pull/1/head"));
}

#[test]
fn sandbox_uses_spec_paths() {
    let sandbox = Sandbox::new();
    assert!(
        sandbox
            .state_dir()
            .ends_with("Library/Application Support/diffmate")
    );
    assert!(sandbox.cache_dir().ends_with("Library/Caches/diffmate"));
    assert!(
        sandbox
            .db_path()
            .ends_with("Application Support/diffmate/diffmate.db")
    );
    for dir in [sandbox.state_dir(), sandbox.cache_dir()] {
        assert!(dir.starts_with(&sandbox.home) && sandbox.home.starts_with(&sandbox.root));
    }
    assert!(sandbox.run(&[]).status.success());
}
