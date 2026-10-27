use super::endpoint::Recorded;
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn bodies_contain(requests: &[Recorded], needle: &str) -> bool {
    assert_ne!(needle, "");
    let needle = needle.as_bytes();
    let has = |request: &Recorded| {
        request
            .body
            .windows(needle.len())
            .any(|part| part == needle)
    };
    requests.iter().any(has)
}

pub(super) fn tool_names(requests: &[Recorded]) -> BTreeSet<String> {
    let parsed = |request: &Recorded| serde_json::from_slice::<Value>(&request.body).ok();
    let tools = requests.iter().filter_map(parsed);
    let tools = tools.filter_map(|body| body["tools"].as_array().cloned());
    let tools: Vec<Value> = tools.flatten().collect();
    let names = tools.iter().filter_map(|tool| tool["name"].as_str());
    names.map(str::to_string).collect()
}

pub(super) fn assert_reached(requests: &[Recorded], what: &str) {
    let reached = |r: &Recorded| {
        r.method == "POST" && r.path.starts_with("/v1/messages") && !r.body.is_empty()
    };
    assert!(
        requests.iter().any(reached),
        "{what}: the CLI never sent a model request"
    );
}
