use super::{ClaudeCli, TestOverrides};
use crate::agent::AgentRequest;
use crate::procenv::InheritedEnv;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) fn contract() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude/contract.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

pub(super) fn inherited(extra: &[(&str, &str)]) -> InheritedEnv {
    let mut pairs = vec![("PATH".into(), std::env::var_os("PATH").unwrap())];
    pairs.extend(
        extra
            .iter()
            .map(|(name, value)| ((*name).into(), (*value).into())),
    );
    InheritedEnv::from_pairs(pairs)
}

pub(super) fn request() -> AgentRequest {
    let path = |text: &str| PathBuf::from(text);
    let prompt = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("prompts/review.md");
    AgentRequest::new(path("/wt"), path("/ctx"), prompt)
}

pub(super) fn rules(request: &AgentRequest) -> String {
    std::fs::read_to_string(&request.prompt_file).unwrap()
}

#[test]
fn overrides_change_only_the_test_command() {
    let plain = ClaudeCli::new(&inherited(&[("HOME", "/h")])).unwrap();
    let request = request();
    let rules = rules(&request);
    let base = plain.args_for(&request, &rules);
    let overrides = TestOverrides {
        extra_args: vec!["--bogus".into()],
        removed_flags: vec!["--disable-web-search", "--no-subagents"],
        extra_env: vec![
            ("HOME".into(), "/other".into()),
            ("EXTRA".into(), "1".into()),
        ],
    };
    let tuned = ClaudeCli::new(&inherited(&[("HOME", "/h")]))
        .unwrap()
        .with_overrides(overrides);

    let mut expected = base.clone();
    expected.retain(|arg| arg != "--disable-web-search" && arg != "--no-subagents");
    expected.push("--bogus".into());
    assert_eq!(tuned.args_for(&request, &rules), expected);
    assert_eq!(plain.args_for(&request, &rules), base);
    let homes: Vec<_> = tuned
        .env_vars()
        .iter()
        .filter(|(name, _)| name == "HOME")
        .collect();
    assert_eq!(homes.len(), 1);
    assert_eq!(homes[0].1, "/other");
    let last = tuned.env_vars().last().unwrap();
    assert_eq!((&last.0, &last.1), (&"EXTRA".into(), &"1".into()));
    let plain_names: Vec<_> = plain.env_vars().iter().map(|(name, _)| name).collect();
    assert!(!plain_names.contains(&&"EXTRA".into()));
}
