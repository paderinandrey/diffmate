use super::Schemas;
use proptest::prelude::*;
use serde_json::{Value, json};

fn finding(anchored: bool, previous: Option<String>) -> Value {
    let mut finding = json!({"severity": "low", "path": null, "line": null, "side": null,
        "title": "t", "body": "b"});
    if anchored {
        finding["path"] = json!("a.rs");
        finding["line"] = json!(7);
        finding["side"] = json!("LEFT");
    }
    if let Some(id) = previous {
        finding["previous_finding_id"] = json!(id);
    }
    finding
}

fn valid_output() -> impl Strategy<Value = Value> {
    let one = (
        any::<bool>(),
        prop::option::of("[a-z0-9.]{1,6}"),
        "[a-z]{1,8}",
        "[a-z]{1,8}",
        1..9_999_u32,
    )
        .prop_map(|(anchored, previous, title, body, line)| {
            let mut value = finding(anchored, previous);
            value["title"] = json!(title);
            value["body"] = json!(body);
            if anchored {
                value["line"] = json!(line);
            }
            value
        });
    let findings = prop::collection::vec(one, 0..=5);
    let limitation = prop::option::of("[a-z]{1,8}");
    let future = prop::collection::vec("[a-z]{1,8}", 0..=2);
    (findings, limitation, future).prop_map(|(findings, limitation, future)| {
        let status = if findings.is_empty() {
            "no_findings"
        } else {
            "findings"
        };
        json!({"status": status, "limitation": limitation, "findings": findings,
            "future_improvements": future})
    })
}

proptest! {
    #[test]
    fn generated_outputs_are_valid_under_both_forms(value in valid_output()) {
        let schemas = Schemas::load().unwrap();
        prop_assert!(schemas.check(&value).is_ok());
        let cli: Value = serde_json::from_str(&schemas.cli_arg).unwrap();
        prop_assert!(jsonschema::draft202012::is_valid(&cli, &value));
    }
}
