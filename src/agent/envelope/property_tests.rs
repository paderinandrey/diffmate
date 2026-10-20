use super::support::{envelope, finding, output, run};
use crate::agent::{AgentOutcome, ReviewStatus};
use proptest::prelude::*;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Shape {
    Valid,
    Absent,
    SchemaInvalid,
    Mismatched,
}

fn structured(shape: Shape) -> Option<Value> {
    match shape {
        Shape::Valid => Some(output("findings", &[finding()])),
        Shape::Absent => None,
        Shape::SchemaInvalid => Some(json!({"status": "findings", "extra": true})),
        Shape::Mismatched => Some(output("no_findings", &[finding()])),
    }
}

proptest! {
    #[test]
    fn success_requires_all_four_checks(
        exit in 0..3_i32,
        is_error in prop::option::of(any::<bool>()),
        subtype in prop::sample::select(vec![
            "success",
            "error_max_turns",
            "error_during_execution",
            "error_max_structured_output_retries",
        ]),
        shape in prop::sample::select(vec![
            Shape::Valid,
            Shape::Absent,
            Shape::SchemaInvalid,
            Shape::Mismatched,
        ]),
    ) {
        let mut value = envelope(subtype, false, structured(shape));
        match is_error {
            Some(flag) => value["is_error"] = json!(flag),
            None => { value.as_object_mut().unwrap().remove("is_error"); }
        }
        let expected = exit == 0 && is_error != Some(true) && subtype == "success" && shape == Shape::Valid;
        match run(&value, exit) {
            AgentOutcome::Succeeded { output, .. } => {
                prop_assert!(expected);
                prop_assert_eq!(output.status, ReviewStatus::Findings);
                prop_assert!(!output.findings.is_empty());
            }
            AgentOutcome::Failed { .. } | AgentOutcome::Cancelled { .. } => prop_assert!(!expected),
        }
    }
}
