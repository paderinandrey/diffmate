mod fixture;
mod property;

use super::normalize;
use crate::agent::Side;
use fixture::{finding, index, output};
use std::collections::HashSet;

#[test]
fn in_hunk_anchors_on_both_sides_are_kept() {
    let findings = vec![
        finding(Some("src/lib.rs"), Some(2), Some("RIGHT"), None),
        finding(Some("src/lib.rs"), Some(2), Some("LEFT"), None),
    ];

    let normalized = normalize(&output(&findings), &index(), &HashSet::new());

    assert_eq!(normalized.warnings, Vec::<String>::new());
    let kept: Vec<_> = normalized
        .findings
        .iter()
        .map(|f| (f.line, f.side.map(Side::as_str)))
        .collect();
    assert_eq!(kept, [(Some(2), Some("RIGHT")), (Some(2), Some("LEFT"))]);
}

#[test]
fn an_out_of_hunk_anchor_moves_to_the_body_and_keeps_its_path() {
    let findings = vec![
        finding(Some("src/lib.rs"), Some(1), Some("RIGHT"), None),
        finding(Some("src/lib.rs"), Some(50), Some("RIGHT"), None),
        finding(Some("gone.rs"), Some(1), Some("LEFT"), None),
        finding(Some("src/lib.rs"), None, None, None),
    ];

    let normalized = normalize(&output(&findings), &index(), &HashSet::new());

    let paths: Vec<_> = normalized
        .findings
        .iter()
        .map(|f| (f.path.as_deref(), f.line, f.side.is_some()))
        .collect();
    let expected = [
        (Some("src/lib.rs"), Some(1), true),
        (Some("src/lib.rs"), None, false),
        (Some("gone.rs"), None, false),
        (Some("src/lib.rs"), None, false),
    ];
    assert_eq!(paths, expected);
    assert_eq!(
        normalized.warnings,
        [
            "finding 2: src/lib.rs:50 (RIGHT) is not inside a hunk of diff.patch; moved to the review body",
            "finding 3: gone.rs:1 (LEFT) is not inside a hunk of diff.patch; moved to the review body",
        ]
    );
}

#[test]
fn an_unknown_previous_id_is_nulled_and_a_known_one_kept() {
    let findings = vec![
        finding(None, None, None, Some("9.1")),
        finding(None, None, None, Some("2.1")),
    ];
    let known = HashSet::from(["2.1".to_string()]);

    let normalized = normalize(&output(&findings), &index(), &known);

    let ids: Vec<_> = normalized
        .findings
        .iter()
        .map(|f| f.previous_finding_id.as_deref())
        .collect();
    assert_eq!(ids, [None, Some("2.1")]);
    assert_eq!(
        normalized.warnings,
        [
            "finding 1: previous_finding_id 9.1 does not refer to a finding in previous.json; treated as new"
        ]
    );
}
