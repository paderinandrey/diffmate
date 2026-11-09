use super::*;
use crate::agent::{FailureDetail, FailureKind};
use serde_json::{Value, json};

pub(super) const NOW: i64 = 1_700_000_000_000;

pub(super) fn detail(status: Option<Value>, message: &str) -> FailureDetail {
    FailureDetail {
        message: message.into(),
        api_error_status: status,
        ..FailureDetail::default()
    }
}

pub(super) fn agent(detail: &FailureDetail) -> Failure {
    if is_transient(detail) {
        Failure::TransientAgent
    } else {
        Failure::Terminal(FailureKind::AgentError)
    }
}

pub(super) fn open(retry_count: u32, max_retries: Option<u32>) -> RetryState {
    RetryState {
        retry_count,
        max_retries,
    }
}

pub(super) fn status(code: u64) -> Value {
    json!(code)
}
