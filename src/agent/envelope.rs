use super::schema::Schemas;
use super::{AgentOutcome, FailureDetail, FailureKind, ReviewOutput, ReviewStatus};
use crate::proc::{Captured, excerpt};
use crate::store::records::Metrics;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Envelope {
    is_error: bool,
    subtype: Option<String>,
    structured_output: Option<Value>,
    errors: Option<Value>,
    api_error_status: Option<Value>,
    terminal_reason: Option<Value>,
    total_cost_usd: Option<f64>,
    duration_ms: Option<i64>,
}

fn failed(kind: FailureKind, detail: FailureDetail, metrics: Metrics) -> AgentOutcome {
    AgentOutcome::Failed {
        kind,
        detail,
        metrics,
    }
}

pub(crate) fn classify(captured: &Captured, schemas: &Schemas) -> AgentOutcome {
    let exit_code = captured.status.code();
    let stderr = excerpt(&captured.stderr, 500);
    let stderr_excerpt = Some(stderr).filter(|text| !text.is_empty());
    let invalid = |message: String, metrics: Metrics| {
        let detail = FailureDetail {
            message,
            exit_code,
            stderr_excerpt: stderr_excerpt.clone(),
            ..FailureDetail::default()
        };
        failed(FailureKind::InvalidOutput, detail, metrics)
    };
    let envelope = if captured.stdout_overflow {
        None
    } else {
        serde_json::from_slice::<Envelope>(&captured.stdout).ok()
    };
    let Some(envelope) = envelope else {
        let message = "stdout is not a result envelope".to_string();
        return invalid(message, Metrics::default());
    };
    let metrics = Metrics {
        total_cost_usd: envelope.total_cost_usd,
        duration_ms: envelope.duration_ms,
    };
    if exit_code != Some(0) || envelope.is_error || envelope.subtype.as_deref() != Some("success") {
        let detail = FailureDetail {
            message: format!("agent run failed (exit code {exit_code:?})"),
            exit_code,
            subtype: envelope.subtype,
            is_error: Some(envelope.is_error),
            errors: envelope.errors,
            api_error_status: envelope.api_error_status,
            terminal_reason: envelope.terminal_reason,
            stderr_excerpt,
        };
        return failed(FailureKind::AgentError, detail, metrics);
    }
    let Some(raw) = envelope.structured_output else {
        return invalid("success envelope has no structured_output".into(), metrics);
    };
    if let Err(why) = schemas.check(&raw) {
        return invalid(
            format!("structured_output violates the schema: {why}"),
            metrics,
        );
    }
    let output = match serde_json::from_value::<ReviewOutput>(raw.clone()) {
        Ok(output) => output,
        Err(error) => return invalid(error.to_string(), metrics),
    };
    if (output.status == ReviewStatus::Findings) == output.findings.is_empty() {
        return invalid("status disagrees with findings".into(), metrics);
    }
    AgentOutcome::Succeeded {
        output,
        raw,
        metrics,
    }
}
