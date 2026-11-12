#[path = "agent_contract/classification.rs"]
mod classification;
mod support;
#[allow(dead_code)]
mod tracer;

use serde_json::Value;
use support::fake_claude::{DEFAULT_VERSION, PROBE_OK};
use support::sandbox::Sandbox;
use tracer::{count, database, review, with_pr};

const CONTRACT: &str = include_str!("fixtures/claude/contract.json");

pub(crate) fn failed_run(sandbox: &Sandbox) -> (String, String) {
    let db = database(sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    let sql = "SELECT failure_kind, failure_detail FROM runs WHERE state = 'failed'";
    db.query_row(sql, [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
}

fn refused_before_agent(
    sandbox: &Sandbox,
    version: Option<&str>,
    help: Option<(&str, i32)>,
    expect: &str,
) {
    with_pr(sandbox);
    if let Some(version) = version {
        sandbox.claude.set_version(version);
    }
    if let Some((text, exit)) = help {
        sandbox.claude.set_help(text, exit);
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
    let help = Some(("Usage: grok\n--single\n", 0));
    refused_before_agent(&Sandbox::new(), None, help, "--tools");
}

#[test]
fn old_cli_version_fails_before_agent_starts() {
    let old = Some("grok 1.0.76 (old) [stable]");
    refused_before_agent(&Sandbox::new(), old, None, "1.0.77");
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
        Some(("something unrelated", 2)),
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
