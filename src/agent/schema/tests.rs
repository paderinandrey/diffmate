use super::{FULL, Schemas};
use serde_json::{Value, json};
use std::path::Path;

fn full() -> Value {
    serde_json::from_str(FULL).unwrap()
}

fn contract() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude/contract.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn full_schema_is_valid_draft_2020_12() {
    assert!(jsonschema::draft202012::meta::is_valid(&full()));
}

#[test]
fn cli_schema_argument_has_no_dollar_schema() {
    let cli: Value = serde_json::from_str(&Schemas::load().unwrap().cli_arg).unwrap();
    assert!(cli.is_object() && cli.get("$schema").is_none());
}

#[test]
fn cli_schema_argument_is_the_form_recorded_by_the_spike() {
    let contract = contract();
    assert_eq!(contract["schema_form"], "full");
    assert!(contract["schema_file"].is_null());
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/review.cli.schema.json");
    assert!(!file.exists());
    let mut expected = full();
    expected.as_object_mut().unwrap().remove("$schema");
    let cli: Value = serde_json::from_str(&Schemas::load().unwrap().cli_arg).unwrap();
    assert_eq!(cli, expected);
}

const MUTATIONS: [&str; 7] = [
    "missing status",
    "extra property",
    "line 0",
    "side UP",
    "three improvements",
    "empty title",
    "line without side",
];

fn mutate(name: &str, v: &mut Value) {
    match name {
        "missing status" => {
            v.as_object_mut().unwrap().remove("status");
        }
        "extra property" => v["extra"] = json!(1),
        "line 0" => v["findings"][0]["line"] = json!(0),
        "side UP" => v["findings"][0]["side"] = json!("UP"),
        "three improvements" => v["future_improvements"] = json!(["a", "b", "c"]),
        "empty title" => v["findings"][0]["title"] = json!(""),
        _ => v["findings"][0]["side"] = Value::Null,
    }
}

#[test]
fn the_negative_corpus_is_rejected() {
    let base = json!({"status": "findings", "limitation": null,
        "findings": [{"severity": "low", "path": "a.rs", "line": 7, "side": "LEFT",
            "title": "t", "body": "b"}], "future_improvements": []});
    let schemas = Schemas::load().unwrap();
    assert!(schemas.check(&base).is_ok());
    for name in MUTATIONS {
        let mut value = base.clone();
        mutate(name, &mut value);
        assert!(schemas.check(&value).is_err(), "{name}");
    }
}
