use super::endpoint::{FakeAnthropic, Recorded};
use super::fixture::MaliciousPr;
use crate::agent::claude::{ClaudeCli, TestOverrides};
use crate::agent::{Agent, AgentOutcome, AgentRequest};
use crate::procenv::InheritedEnv;
use std::ffi::OsString;

pub(super) const EXPECTED_TOOLS: [&str; 5] = [
    "Read",
    "Grep",
    "Glob",
    "StructuredOutput",
    "DeferredToolPlaceholder",
];
const WITHOUT_ISOLATION: [&str; 3] = ["--safe-mode", "--restricted", "--strict-mcp-config"];
const WITHOUT_TOOL_LIMITS: [&str; 3] = ["--tools", "Read,Grep,Glob", "--disable-slash-commands"];

fn inherited(pr: &MaliciousPr) -> InheritedEnv {
    let process_path = std::env::var_os("PATH").unwrap();
    let dirs = std::iter::once(pr.bin.clone()).chain(std::env::split_paths(&process_path));
    let mut pairs = vec![
        ("PATH".into(), std::env::join_paths(dirs).unwrap()),
        ("HOME".into(), pr.home.clone().into()),
    ];
    for name in ["USER", "LOGNAME"] {
        pairs.extend(std::env::var_os(name).map(|value| (name.into(), value)));
    }
    InheritedEnv::from_pairs(pairs)
}

fn endpoint_env(pr: &MaliciousPr, endpoint: &FakeAnthropic) -> Vec<(OsString, OsString)> {
    vec![
        ("HOME".into(), pr.home.clone().into()),
        ("CLAUDE_CONFIG_DIR".into(), pr.config_dir.clone().into()),
        ("ANTHROPIC_BASE_URL".into(), endpoint.url.clone().into()),
        ("ANTHROPIC_API_KEY".into(), "dummy".into()),
        (
            "CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD".into(),
            "1".into(),
        ),
    ]
}

async fn run(
    pr: &MaliciousPr,
    endpoint: &FakeAnthropic,
    label: &str,
    removed_flags: Vec<&'static str>,
) -> AgentOutcome {
    pr.reset_traces();
    let overrides = TestOverrides {
        extra_args: Vec::new(),
        removed_flags,
        extra_env: endpoint_env(pr, endpoint),
    };
    let cli = ClaudeCli::new(&inherited(pr))
        .unwrap()
        .with_overrides(overrides);
    let prompt = pr.prompt_for(label);
    let request = AgentRequest::new(pr.worktree.clone(), pr.context_dir.clone(), prompt);
    cli.run(&request).await
}

pub(super) async fn run_isolated(pr: &MaliciousPr, endpoint: &FakeAnthropic) -> AgentOutcome {
    run(pr, endpoint, "isolated", Vec::new()).await
}

pub(super) async fn run_control(pr: &MaliciousPr, endpoint: &FakeAnthropic) -> AgentOutcome {
    run(pr, endpoint, "control", WITHOUT_ISOLATION.to_vec()).await
}

pub(super) async fn run_open_control(pr: &MaliciousPr, endpoint: &FakeAnthropic) -> AgentOutcome {
    let removed = [WITHOUT_ISOLATION, WITHOUT_TOOL_LIMITS].concat();
    run(pr, endpoint, "open-control", removed).await
}

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Isolated,
    Control,
    OpenControl,
}

pub(super) async fn observe(
    pr: &MaliciousPr,
    secondary: &FakeAnthropic,
    mode: Mode,
) -> Vec<Recorded> {
    let endpoint = FakeAnthropic::start().await;
    match mode {
        Mode::Isolated => run_isolated(pr, &endpoint).await,
        Mode::Control => run_control(pr, &endpoint).await,
        Mode::OpenControl => run_open_control(pr, &endpoint).await,
    };
    [endpoint.take(), secondary.take()].concat()
}
