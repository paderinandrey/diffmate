use serde_json::Value;

const CONTRACT: &str = include_str!("../../../tests/fixtures/claude/contract.json");

pub(super) fn contract() -> Value {
    serde_json::from_str(CONTRACT).unwrap()
}

pub(super) fn control_fired(name: &str) -> bool {
    contract()["controls"][name].as_bool().unwrap()
}
