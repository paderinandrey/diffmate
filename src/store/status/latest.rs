pub(super) struct Row {
    pub job_state: Option<String>,
    pub failure_reason: Option<String>,
    pub run_state: Option<String>,
    pub result: Option<String>,
    pub failure_kind: Option<String>,
    pub cause: Option<String>,
    pub findings: i64,
}

pub(super) fn latest(row: &Row) -> String {
    let none = || "none".to_string();
    match row.run_state.as_deref() {
        Some("succeeded") if row.result.as_deref() == Some("findings") => {
            format!("succeeded (findings: {})", row.findings)
        }
        Some("succeeded") => format!("succeeded ({})", row.result.clone().unwrap_or_else(none)),
        Some("failed") => match (row.failure_kind.as_deref(), row.cause.as_deref()) {
            (Some("interrupted"), Some(cause)) => format!("failed (interrupted: {cause})"),
            (Some(kind), _) => format!("failed ({kind})"),
            (None, _) => "failed".to_string(),
        },
        Some(state) => state.to_string(),
        None => match row.job_state.as_deref() {
            Some("failed") => {
                let reason = row.failure_reason.clone().unwrap_or_else(none);
                format!("failed before run: {reason}")
            }
            Some(state) => state.to_string(),
            None => "queued".to_string(),
        },
    }
}
