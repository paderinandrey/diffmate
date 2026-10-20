use super::validate_branch_name;
use proptest::prelude::*;

#[test]
fn ordinary_branch_names_are_accepted() {
    for name in ["main", "release/1.2", "feature-x", "user/fix_it", "ünï"] {
        assert!(validate_branch_name(name).is_ok(), "{name}");
    }
}

#[test]
fn names_that_could_alter_a_refspec_are_rejected() {
    let rejected = [
        "",
        "-x",
        "a:b",
        "a b",
        "a..b",
        "a\u{7}",
        "a*",
        "a?",
        "a[",
        "a\\b",
        "/a",
        "a/",
        "a.lock",
        "a//b",
        "a@{1}",
        "@",
        "a.",
        ".a",
        "a/.b",
        "a~1",
        "a^",
        "a\tb",
        "a\u{7f}",
        "a\u{a0}b",
        "main:refs/heads/evil",
    ];
    for name in rejected {
        assert!(validate_branch_name(name).is_err(), "{name:?}");
    }
}

proptest! {
    #[test]
    fn accepted_names_are_safe_in_a_refspec(name in "\\PC{0,24}") {
        if validate_branch_name(&name).is_ok() {
            let refspec = format!("+refs/heads/{name}:refs/diffmate/base/{name}");
            prop_assert_eq!(refspec.matches(':').count(), 1);
            prop_assert!(!name.contains("..") && !name.starts_with('-'));
            prop_assert!(!name.chars().any(|c| c.is_whitespace() || c.is_control() || "*?[\\~^".contains(c)));
        }
    }
}
