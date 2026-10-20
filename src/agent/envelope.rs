mod fields;
#[cfg(test)]
mod golden_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod support;
#[cfg(test)]
mod synthetic_tests;
#[cfg(test)]
mod tests;

use super::schema::Schemas;
use super::{AgentOutcome, FailureDetail, FailureKind, ReviewOutput, ReviewStatus};
use crate::proc::{Captured, excerpt};
use crate::store::records::Metrics;
use fields::Fields;
use serde_json::Value;

const RETRIES_EXHAUSTED: &str = "error_max_structured_output_retries";

fn failed(kind: FailureKind, detail: FailureDetail, metrics: Metrics) -> AgentOutcome {
    AgentOutcome::Failed {
        kind,
        detail,
        metrics,
    }
}

fn structured(fields: &Fields, schemas: &Schemas) -> Result<(ReviewOutput, Value), String> {
    let value = match fields.get("structured_output") {
        None | Some(Value::Null) => return Err("success envelope has no structured_output".into()),
        Some(Value::String(text)) => serde_json::from_str(text)
            .map_err(|error| format!("structured_output is a string that is not JSON: {error}"))?,
        Some(other) => other.clone(),
    };
    if let Err(why) = schemas.check(&value) {
        return Err(format!("structured_output violates the schema: {why}"));
    }
    let output = serde_json::from_value::<ReviewOutput>(value.clone());
    let output = output.map_err(|error| error.to_string())?;
    if (output.status == ReviewStatus::Findings) == output.findings.is_empty() {
        return Err("status disagrees with findings".into());
    }
    Ok((output, value))
}

pub(crate) fn classify(captured: &Captured, schemas: &Schemas) -> AgentOutcome {
    let exit_code = captured.status.code();
    let stderr = excerpt(&captured.stderr, 500);
    let mut detail = FailureDetail {
        exit_code,
        stderr_excerpt: Some(stderr).filter(|text| !text.is_empty()),
        ..FailureDetail::default()
    };
    let fields = (!captured.stdout_overflow).then_some(&captured.stdout);
    let fields = fields.and_then(|stdout| Fields::parse(stdout));
    let Some(fields) = fields else {
        let overflow = captured.stdout_overflow;
        detail.message = if overflow {
            "stdout exceeds the capture limit".into()
        } else {
            "stdout is not a result envelope".into()
        };
        return failed(FailureKind::InvalidOutput, detail, Metrics::default());
    };
    let metrics = fields.metrics();
    detail.subtype = fields.subtype().map(str::to_string);
    detail.is_error = fields.get("is_error").and_then(Value::as_bool);
    detail.errors = fields.get("errors").cloned();
    detail.api_error_status = fields.get("api_error_status").cloned();
    detail.terminal_reason = fields.get("terminal_reason").cloned();
    let subtype = fields.subtype().unwrap_or("none").to_string();
    if fields.logged_out() {
        detail.message = "Claude CLI is not authenticated".into();
        return failed(FailureKind::CliError, detail, metrics);
    }
    if fields.subtype() == Some(RETRIES_EXHAUSTED) {
        detail.message = "the agent could not produce schema-valid output".into();
        return failed(FailureKind::InvalidOutput, detail, metrics);
    }
    if exit_code != Some(0) || fields.is_error() || fields.subtype() != Some("success") {
        let reason = fields.reason();
        detail.message = format!("agent run failed ({subtype}, exit {exit_code:?}): {reason}");
        return failed(FailureKind::AgentError, detail, metrics);
    }
    match structured(&fields, schemas) {
        Ok((output, raw)) => AgentOutcome::Succeeded {
            output,
            raw,
            metrics,
        },
        Err(message) => {
            detail.message = message;
            failed(FailureKind::InvalidOutput, detail, metrics)
        }
    }
}
