use super::run::capture;
use super::{check_flags, check_version, interpret_probe};
use crate::agent::AgentRequest;
use crate::agent::claude::{ClaudeCli, TestOverrides};
use crate::procenv::InheritedEnv;
use std::ffi::OsStr;
use tempfile::TempDir;

const FULL_SCHEMA: &str = include_str!("../../../schemas/review.schema.json");

struct Live {
    _tmp: TempDir,
    cli: ClaudeCli,
    request: AgentRequest,
}

fn live(overrides: TestOverrides) -> Live {
    let tmp = TempDir::new().unwrap();
    let make = |name: &str| {
        let dir = tmp.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        dir
    };
    let (worktree, context) = (make("worktree"), make("context"));
    let prompt = tmp.path().join("review.md");
    std::fs::write(&prompt, "# Review rules\n").unwrap();
    let cli = ClaudeCli::new(&InheritedEnv::capture()).unwrap();
    Live {
        cli: cli.with_overrides(overrides),
        request: AgentRequest::new(worktree, context, prompt),
        _tmp: tmp,
    }
}

async fn probe(live: &Live) -> Result<(), String> {
    let program = OsStr::new("claude");
    let env = live.cli.env_vars();
    check_version(program, env, None)
        .await
        .map_err(|d| d.message)?;
    let args = live.cli.args_for(&live.request);
    let probed = check_flags(program, env, &args, &live.request.worktree, None).await;
    probed.map(drop).map_err(|detail| detail.message)
}

#[tokio::test]
#[ignore = "needs the real claude CLI"]
async fn live_probe_accepts_production_flags() {
    let live = live(TestOverrides::default());
    probe(&live).await.unwrap();
}

#[tokio::test]
#[ignore = "needs the real claude CLI"]
async fn live_probe_names_unknown_flag() {
    let live = live(TestOverrides {
        extra_args: vec!["--bogus-flag-xyz".into()],
        ..TestOverrides::default()
    });
    let message = probe(&live).await.unwrap_err();
    assert!(message.contains("--bogus-flag-xyz"), "{message}");
}

#[tokio::test]
#[ignore = "needs the real claude CLI"]
async fn live_probe_rejects_dollar_schema_and_accepts_cli_schema() {
    let live = live(TestOverrides::default());
    let mut args = live.cli.args_for(&live.request);
    let at = args.iter().position(|arg| arg == "--json-schema").unwrap();
    args[at + 1] = FULL_SCHEMA.into();
    let cwd = Some(live.request.worktree.as_path());
    let done = capture(OsStr::new("claude"), &args, live.cli.env_vars(), cwd, None).await;
    let done = done.unwrap().unwrap();
    let verdict = interpret_probe(&done.stdout, &done.stderr, done.status.code());
    assert!(verdict.unwrap_err().message.contains("--json-schema"));
    probe(&live).await.unwrap();
}
