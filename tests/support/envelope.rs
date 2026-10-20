fn quote(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn nullable(text: Option<&str>) -> String {
    text.map_or_else(|| "null".to_string(), quote)
}

pub fn success_envelope(structured_output_json: &str) -> String {
    format!(
        "{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"duration_ms\":1234,\"duration_api_ms\":1000,\"num_turns\":2,\"result\":\"\",\"session_id\":\"00000000-0000-0000-0000-000000000000\",\"total_cost_usd\":0.0123,\"structured_output\":{structured_output_json}}}"
    )
}

pub fn finding_json(
    path: Option<&str>,
    line: Option<u32>,
    side: Option<&str>,
    severity: &str,
    title: &str,
    body: &str,
) -> String {
    let line = line.map_or_else(|| "null".to_string(), |n| n.to_string());
    format!(
        "{{\"severity\":{},\"path\":{},\"line\":{line},\"side\":{},\"title\":{},\"body\":{},\"previous_finding_id\":null}}",
        quote(severity),
        nullable(path),
        nullable(side),
        quote(title),
        quote(body),
    )
}

pub fn output_json(
    findings: &[String],
    limitation: Option<&str>,
    future_improvements: &[&str],
) -> String {
    let status = if findings.is_empty() {
        "no_findings"
    } else {
        "findings"
    };
    let improvements: Vec<String> = future_improvements.iter().map(|item| quote(item)).collect();
    format!(
        "{{\"status\":\"{status}\",\"limitation\":{},\"findings\":[{}],\"future_improvements\":[{}]}}",
        nullable(limitation),
        findings.join(","),
        improvements.join(","),
    )
}
