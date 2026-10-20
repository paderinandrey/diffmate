pub(crate) mod claude;
pub(crate) mod envelope;
mod gitwrap;
mod output;
pub(crate) mod probe;
pub(crate) mod schema;
mod spawn;

use crate::store::records::Metrics;
pub(crate) use output::{ReviewOutput, ReviewStatus, Severity, Side};
use serde::Serialize;
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub(crate) enum FailureKind {
    CliError,
    AgentError,
    InvalidOutput,
}

impl FailureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FailureKind::CliError => "cli_error",
            FailureKind::AgentError => "agent_error",
            FailureKind::InvalidOutput => "invalid_output",
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct FailureDetail {
    pub message: String,
    pub exit_code: Option<i32>,
    pub subtype: Option<String>,
    pub is_error: Option<bool>,
    pub errors: Option<Value>,
    pub api_error_status: Option<Value>,
    pub terminal_reason: Option<Value>,
    pub stderr_excerpt: Option<String>,
}

pub(crate) enum AgentOutcome {
    Succeeded {
        output: ReviewOutput,
        raw: Value,
        metrics: Metrics,
    },
    Failed {
        kind: FailureKind,
        detail: FailureDetail,
        metrics: Metrics,
    },
}

pub(crate) struct AgentRequest {
    pub worktree: PathBuf,
    pub context_dir: PathBuf,
    pub prompt_file: PathBuf,
}

impl AgentRequest {
    pub fn new(worktree: PathBuf, context_dir: PathBuf, prompt_file: PathBuf) -> AgentRequest {
        AgentRequest {
            worktree,
            context_dir,
            prompt_file,
        }
    }
}

pub(crate) trait Agent {
    fn run(&self, request: &AgentRequest) -> impl Future<Output = AgentOutcome> + Send;
}
