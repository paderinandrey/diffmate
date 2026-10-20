use super::{ClaudeCli, TestOverrides};
use crate::agent::AgentRequest;
use crate::procenv::InheritedEnv;
use std::path::PathBuf;

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
    AgentRequest::new(path("/wt"), path("/ctx"), path("/prompt/review.md"))
}

#[test]
fn overrides_change_only_the_test_command() {
    let plain = ClaudeCli::new(&inherited(&[("HOME", "/h")])).unwrap();
    let base = plain.args_for(&request());
    let overrides = TestOverrides {
        extra_args: vec!["--bogus".into()],
        removed_flags: vec!["--disable-slash-commands", "--safe-mode"],
        extra_env: vec![
            ("HOME".into(), "/other".into()),
            ("EXTRA".into(), "1".into()),
        ],
    };
    let tuned = ClaudeCli::new(&inherited(&[("HOME", "/h")]))
        .unwrap()
        .with_overrides(overrides);

    let mut expected = base.clone();
    expected.retain(|arg| arg != "--disable-slash-commands" && arg != "--safe-mode");
    expected.push("--bogus".into());
    assert_eq!(tuned.args_for(&request()), expected);
    assert_eq!(plain.args_for(&request()), base);
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
