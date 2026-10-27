pub(crate) mod claude;
pub(crate) mod envelope;
mod gitwrap;
mod output;
pub(crate) mod probe;
mod request;
pub(crate) mod schema;
mod spawn;
mod stop;

use crate::store::records::Metrics;
pub(crate) use output::{ReviewOutput, ReviewStatus, Severity, Side};
pub(crate) use request::AgentRequest;
use serde::Serialize;
use serde_json::Value;
use std::future::Future;

#[derive(Debug, Clone, Copy)]
pub(crate) enum FailureKind {
    CliError,
    AgentError,
    InvalidOutput,
    Interrupted,
}

impl FailureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FailureKind::CliError => "cli_error",
            FailureKind::AgentError => "agent_error",
            FailureKind::InvalidOutput => "invalid_output",
            FailureKind::Interrupted => "interrupted",
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
    Cancelled {
        metrics: Metrics,
    },
}

pub(crate) trait Agent {
    fn run(&self, request: &AgentRequest) -> impl Future<Output = AgentOutcome> + Send;
}
