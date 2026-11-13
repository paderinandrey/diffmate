use super::support::sandbox::Sandbox;
use std::fs;
use std::process::{Output, Stdio};

const SCHEMA: &str = "schema: ok \u{2014} no database yet";

fn fresh() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.gh.route("auth status", "logged in\n");
    sandbox
}

fn doctor(sandbox: &Sandbox) -> Output {
    let tmp = sandbox.root.join("tmp");
    fs::create_dir(&tmp).unwrap();
    sandbox
        .diffmate()
        .env("TMPDIR", &tmp)
        .args(["doctor"])
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn stdout_lines(output: &Output) -> Vec<String> {
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(stdout.ends_with('\n'), "{stdout:?}");
    assert!(!stdout.ends_with("\n\n"), "{stdout:?}");
    stdout.lines().map(ToOwned::to_owned).collect()
}

fn assert_order(lines: &[String]) {
    let prefixes = ["gh auth:", "grok:", "storage:", "runner lock:", "schema:"];
    assert_eq!(lines.len(), prefixes.len(), "{lines:?}");
    for (line, prefix) in lines.iter().zip(prefixes) {
        assert!(line.starts_with(prefix), "{lines:?}");
    }
}

fn assert_tmp_empty(sandbox: &Sandbox) {
    let entries = fs::read_dir(sandbox.root.join("tmp")).unwrap();
    assert_eq!(entries.count(), 0);
}

#[test]
fn fresh_home_prints_five_lines_and_creates_nothing() {
    let sandbox = fresh();
    let output = doctor(&sandbox);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty(), "{output:?}");
    let lines = stdout_lines(&output);
    assert_order(&lines);
    assert_eq!(lines[0], "gh auth: ok");
    assert_eq!(lines[1], "grok: ok");
    assert_eq!(lines[2], "storage: ok");
    assert!(!lines[3].starts_with("runner lock: ok"), "{lines:?}");
    assert_eq!(lines[4], SCHEMA);
    assert!(!sandbox.state_dir().exists());
    assert!(sandbox.claude.run_calls().is_empty());
    assert_eq!(
        sandbox.gh.calls(),
        vec![vec!["auth".to_owned(), "status".to_owned()]]
    );
    assert_tmp_empty(&sandbox);
}

#[test]
fn broken_config_still_prints_five_lines() {
    let sandbox = fresh();
    let dir = sandbox.home.join(".config/diffmate");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("config.toml"), "run_timeout_minutes = 0\n").unwrap();
    let output = doctor(&sandbox);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty(), "{output:?}");
    let lines = stdout_lines(&output);
    assert_order(&lines);
    assert!(lines[2].contains("config.toml"), "{lines:?}");
    assert!(lines[2].contains("run_timeout_minutes"), "{lines:?}");
    assert!(!lines[3].starts_with("runner lock: ok"), "{lines:?}");
    assert!(!sandbox.db_path().exists());
    assert_tmp_empty(&sandbox);
}
