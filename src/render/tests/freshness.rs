use super::fixture::{finding, render, succeeded};
use crate::agent::Severity;
use crate::runner::ReviewResult;

#[test]
fn stale_heads_change_only_the_first_line() {
    let mut result = succeeded(
        vec![finding(Some("src/a.rs"), Some(3), Severity::High, "Issue")],
        Some("Limited"),
        &["Add tests"],
    );
    let (original, original_err) = render(&result);
    let (first, rest) = original.split_once('\n').unwrap();
    for known in [
        "0123456789abcdef0123456789abcdef01234567",
        "999999999999000000",
    ] {
        if let ReviewResult::Succeeded { known_head, .. } = &mut result {
            *known_head = Some(known.into());
        }
        let (out, err) = render(&result);
        let expected = if known.starts_with('9') {
            format!("{first}, stale: PR head is now 999999999999\n{rest}")
        } else {
            original.clone()
        };
        assert_eq!(out, expected);
        assert_eq!(err, original_err);
    }
}
