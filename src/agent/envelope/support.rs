use super::classify;
use crate::agent::schema::Schemas;
use crate::agent::{AgentOutcome, FailureDetail};
use crate::proc::Captured;
use crate::store::records::Metrics;
use serde_json::{Value, json};
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

pub(super) fn schemas() -> Schemas {
    Schemas::load().unwrap()
}

pub(super) fn captured(stdout: &str, exit: i32) -> Captured {
    Captured {
        status: ExitStatus::from_raw(exit << 8),
        stdout: stdout.as_bytes().to_vec(),
        stderr: b"stderr text".to_vec(),
        stdout_overflow: false,
    }
}

pub(super) fn finding() -> Value {
    json!({"severity": "high", "path": "a.rs", "line": 3, "side": "RIGHT", "title": "T", "body": "B"})
}

pub(super) fn output(status: &str, findings: &[Value]) -> Value {
    json!({"status": status, "limitation": null, "findings": findings, "future_improvements": []})
}

pub(super) fn envelope(subtype: &str, is_error: bool, structured: Option<Value>) -> Value {
    let mut value = json!({"type": "result", "subtype": subtype, "is_error": is_error});
    if let Some(structured) = structured {
        value["structured_output"] = structured;
    }
    value
}

pub(super) fn run(value: &Value, exit: i32) -> AgentOutcome {
    classify(&captured(&value.to_string(), exit), &schemas())
}

pub(super) fn failure(outcome: AgentOutcome) -> (&'static str, FailureDetail, Metrics) {
    match outcome {
        AgentOutcome::Failed {
            kind,
            detail,
            metrics,
        } => (kind.as_str(), detail, metrics),
        AgentOutcome::Succeeded { .. } => panic!("expected a failure"),
    }
}
