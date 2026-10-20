mod support;
#[allow(dead_code)]
mod tracer;

use serde_json::Value;
use support::fake_claude::{DEFAULT_VERSION, PROBE_OK};
use support::sandbox::Sandbox;
use tracer::{count, database, review, with_pr};

const CONTRACT: &str = include_str!("fixtures/claude/contract.json");

fn failed_run(sandbox: &Sandbox) -> (String, String) {
    let db = database(sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    let sql = "SELECT failure_kind, failure_detail FROM runs WHERE state = 'failed'";
    db.query_row(sql, [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
}

fn refused_before_agent(
    sandbox: &Sandbox,
    version: Option<&str>,
    probe: Option<&str>,
    expect: &str,
) {
    with_pr(sandbox);
    if let Some(version) = version {
        sandbox.claude.set_version(version);
    }
    if let Some(probe) = probe {
        sandbox.claude.set_probe(probe, 1);
    }

    let output = review(sandbox);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("failed (cli_error)") && stderr.contains(expect),
        "{stderr}"
    );
    assert!(sandbox.claude.run_calls().is_empty());
    let calls = sandbox.claude.calls().len();
    assert_eq!(calls, if version.is_some() { 1 } else { 2 });
    let (kind, detail) = failed_run(sandbox);
    assert_eq!(kind, "cli_error");
    assert!(detail.contains(expect), "{detail}");
}

#[test]
fn missing_flag_fails_before_agent_starts() {
    let probe = "error: unknown option '--disable-slash-commands'";
    refused_before_agent(
        &Sandbox::new(),
        None,
        Some(probe),
        "--disable-slash-commands",
    );
}

#[test]
fn old_cli_version_fails_before_agent_starts() {
    let old = Some("2.1.200 (Claude Code)");
    refused_before_agent(&Sandbox::new(), old, None, "2.1.304");
}

#[test]
fn unreadable_cli_version_fails_before_agent_starts() {
    refused_before_agent(&Sandbox::new(), Some("not a version"), None, "cannot read");
}

#[test]
fn inconclusive_probe_fails_closed() {
    refused_before_agent(
        &Sandbox::new(),
        None,
        Some("something unrelated"),
        "inconclusive",
    );
}

#[test]
fn fake_probe_text_matches_spike() {
    let contract: Value = serde_json::from_str(CONTRACT).unwrap();
    assert_eq!(PROBE_OK, contract["probe"]["ok_stderr"].as_str().unwrap());
    assert_eq!(
        DEFAULT_VERSION,
        contract["version_stdout"].as_str().unwrap()
    );
}
