use crate::store::records::Metrics;
use serde_json::{Map, Value};

const NOT_LOGGED_IN: &str = "Not logged in";

pub(super) struct Fields(Map<String, Value>);

impl Fields {
    pub fn parse(stdout: &[u8]) -> Option<Fields> {
        let trimmed = stdout.trim_ascii();
        if let Ok(Value::Object(map)) = serde_json::from_slice(trimmed) {
            return Some(Fields(map));
        }
        let mut result = None;
        for line in trimmed.split(|byte| *byte == b'\n') {
            let line = line.trim_ascii();
            if line.is_empty() {
                continue;
            }
            let Ok(Value::Object(map)) = serde_json::from_slice::<Value>(line) else {
                return None;
            };
            if map.get("type").and_then(Value::as_str) == Some("result") {
                result = Some(Fields(map));
            }
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn subtype(&self) -> Option<&str> {
        self.get("subtype").and_then(Value::as_str)
    }

    pub fn is_error(&self) -> bool {
        self.get("is_error").and_then(Value::as_bool) == Some(true)
    }

    pub fn metrics(&self) -> Metrics {
        Metrics {
            total_cost_usd: self.get("total_cost_usd").and_then(Value::as_f64),
            duration_ms: self.get("duration_ms").and_then(Value::as_i64),
        }
    }

    pub fn logged_out(&self) -> bool {
        let result = self.get("result").and_then(Value::as_str);
        let api_error = self.get("terminal_reason").and_then(Value::as_str) == Some("api_error");
        let no_status = self.get("api_error_status").is_none_or(Value::is_null);
        self.is_error()
            && api_error
            && no_status
            && result.is_some_and(|text| text.starts_with(NOT_LOGGED_IN))
    }

    pub fn reason(&self) -> String {
        let first_error = self.get("errors").and_then(|errors| errors.get(0));
        let text = first_error
            .or_else(|| self.get("result"))
            .or_else(|| self.get("message"));
        let text = text.and_then(Value::as_str).unwrap_or("no error text");
        text.chars().take(200).collect()
    }
}
