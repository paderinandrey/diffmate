use crate::agent::{Severity, Side};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JobOrigin {
    Manual,
    Automatic,
}

impl JobOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Automatic => "automatic",
        }
    }

    pub fn parse(value: &str) -> rusqlite::Result<Self> {
        match value {
            "manual" => Ok(Self::Manual),
            "automatic" => Ok(Self::Automatic),
            _ => Err(rusqlite::Error::InvalidQuery),
        }
    }
}

pub(crate) struct Revision {
    pub head_sha: String,
    pub merge_base_sha: String,
    pub base_tip_sha: String,
    pub base_ref: String,
}

#[derive(Clone)]
pub(crate) struct FindingRecord {
    pub severity: Severity,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub side: Option<Side>,
    pub title: String,
    pub body: String,
    pub previous_finding_id: Option<String>,
}

pub(crate) struct DraftRecord {
    pub original_output: serde_json::Value,
    pub limitation: Option<String>,
    pub future_improvements: Vec<String>,
    pub findings: Vec<FindingRecord>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Metrics {
    pub total_cost_usd: Option<f64>,
    pub duration_ms: Option<i64>,
}
