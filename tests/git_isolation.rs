mod support;
#[allow(dead_code)]
mod tracer;

use std::fs;
use support::fake_gh::pr_view_json;
use support::poison::Poison;
use support::sandbox::Sandbox;
use tracer::{BASE_LIB, HEAD_LIB, URL, count, database};

fn review_with(sandbox: &Sandbox, poison: &Poison) -> std::process::Output {
    let (home, xdg) = (sandbox.home.clone(), sandbox.root.join("xdg"));
    fs::create_dir_all(xdg.join("git")).unwrap();
    fs::copy(&poison.config, home.join(".gitconfig")).unwrap();
    fs::copy(&poison.config, xdg.join("git/config")).unwrap();
    let monitor = format!("'core.fsmonitor'='{}'", poison.fsmonitor.display());
    let mut command = sandbox.diffmate();
    command.args(["review", URL]).env("XDG_CONFIG_HOME", &xdg);
    command.env("GIT_CONFIG_GLOBAL", &poison.config);
    command.env("GIT_CONFIG_PARAMETERS", monitor);
    command.env("GIT_CONFIG_COUNT", "1");
    command.env("GIT_CONFIG_KEY_0", "core.hooksPath");
    command.env("GIT_CONFIG_VALUE_0", &poison.hooks);
    command.stdin(std::process::Stdio::null()).output().unwrap()
}

#[test]
fn poisoned_environment_runs_no_git_config() {
    let sandbox = Sandbox::new();
    let poison = Poison::write(&sandbox.root);
    let upstream = sandbox.upstream("o", "r");
    let base = upstream.commit(&[("src/lib.rs", BASE_LIB)], "base");
    upstream.push_branch("main");
    let files = [
        ("src/lib.rs", HEAD_LIB),
        (".gitattributes", "* filter=canary diff=canary\n"),
    ];
    let head = upstream.pr_head(1, "main", &files);
    let json = pr_view_json("o", "r", 1, &head, "main", false);
    sandbox.gh.route(&format!("pr view {URL}"), &json);
    poison.control(&sandbox.root, &upstream.bare, (&base, &head));

    let output = review_with(&sandbox, &poison);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let db = database(&sandbox);
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE state = 'succeeded'"),
        1
    );
    assert_eq!(fs::read_dir(&poison.markers).unwrap().count(), 0);
}

#[test]
fn pr_symlink_reaches_the_agent_as_a_plain_file() {
    let sandbox = Sandbox::new();
    sandbox.claude.snapshot_tree();
    let upstream = sandbox.upstream("o", "r");
    upstream.commit(&[("src/lib.rs", BASE_LIB)], "base");
    upstream.push_branch("main");
    upstream.git(&["checkout", "-q", "--detach", "refs/heads/main"]);
    let head = upstream.commit_symlink("link", "../../outside", "pr");
    upstream.git(&[
        "push",
        "-q",
        upstream.bare.to_str().unwrap(),
        "HEAD:refs/pull/1/head",
    ]);
    let json = pr_view_json("o", "r", 1, &head, "main", false);
    sandbox.gh.route(&format!("pr view {URL}"), &json);

    let output = tracer::review(&sandbox);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let calls = sandbox.claude.run_calls();
    let link = sandbox
        .claude
        .dir
        .join(format!("calls/{}/tree/link", calls[0].n));
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_file());
    assert_eq!(fs::read_to_string(&link).unwrap(), "../../outside");
}
