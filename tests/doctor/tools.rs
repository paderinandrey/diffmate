use super::support::sandbox::Sandbox;

fn failed_lines(sandbox: &Sandbox) -> Vec<String> {
    let output = sandbox.run(&["doctor"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    assert_eq!(stdout.lines().count(), 5, "{stdout}");
    stdout.lines().map(str::to_owned).collect()
}

#[test]
fn missing_gh_and_claude_name_path_without_a_sixth_line() {
    let sandbox = Sandbox::new();
    let output = sandbox
        .diffmate()
        .env("PATH", "/usr/bin:/bin")
        .args(["doctor"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    assert_eq!(lines.len(), 5, "{stdout}");
    assert!(lines[0].contains("/usr/bin:/bin"), "{stdout}");
    assert!(lines[1].contains("/usr/bin:/bin"), "{stdout}");
}

#[test]
fn old_cli_does_not_run_the_flag_probe() {
    let sandbox = Sandbox::new();
    sandbox.gh.route("auth status", "logged in\n");
    sandbox.claude.set_version("grok 1.0.76 (old) [stable]");
    let lines = failed_lines(&sandbox);
    assert!(lines[1].contains("older than the minimum"), "{lines:?}");
    assert!(
        sandbox
            .claude
            .calls()
            .iter()
            .all(|call| { call.argv.iter().all(|arg| !arg.contains("--json-schema")) })
    );
}

#[test]
fn gh_auth_failure_names_login_and_skips_the_agent_run() {
    let sandbox = Sandbox::new();
    sandbox.gh.route_fail("auth status", "not logged in", 1);
    let lines = failed_lines(&sandbox);
    assert!(lines[0].contains("not logged in"), "{lines:?}");
    assert!(lines[0].contains("gh auth login"), "{lines:?}");
    assert_ne!(lines[0], "gh auth: ok");
    assert!(sandbox.claude.run_calls().is_empty());
}
