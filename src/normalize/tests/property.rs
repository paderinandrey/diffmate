use super::super::normalize;
use super::fixture::{finding, index, output};
use crate::agent::Side;
use crate::diff::DiffIndex;
use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;

fn arbitrary_finding() -> impl Strategy<Value = Value> {
    let path = prop::option::of(prop_oneof![Just("src/lib.rs"), Just("other.rs")]);
    let line = prop::option::of(0..6u32);
    let side = prop::option::of(prop_oneof![Just("LEFT"), Just("RIGHT")]);
    let previous = prop::option::of(prop_oneof![Just("1.1"), Just("9.9")]);
    (path, line, side, previous).prop_map(|(p, l, s, v)| finding(p, l, s, v))
}

proptest! {
    #[test]
    fn every_stored_anchor_is_inside_the_diff(raw in prop::collection::vec(arbitrary_finding(), 0..8)) {
        let (index, known) = (index(), HashSet::from(["1.1".to_string()]));
        let moved = raw.iter().filter(|f| anchor_moves(f, &index)).count();
        let nulled = raw.iter().filter(|f| f["previous_finding_id"] == "9.9").count();

        let normalized = normalize(&output(&raw), &index, &known);

        prop_assert_eq!(normalized.findings.len(), raw.len());
        prop_assert_eq!(normalized.warnings.len(), moved + nulled);
        for f in &normalized.findings {
            prop_assert_eq!(f.line.is_none(), f.side.is_none());
            if let (Some(path), Some(line), Some(side)) = (&f.path, f.line, f.side) {
                prop_assert!(index.contains(path, side, line));
            }
        }
    }
}

fn anchor_moves(f: &Value, index: &DiffIndex) -> bool {
    let Some(line) = f["line"].as_u64() else {
        return false;
    };
    let side = match f["side"].as_str() {
        Some("LEFT") => Side::Left,
        Some("RIGHT") => Side::Right,
        _ => return true,
    };
    let Some(path) = f["path"].as_str() else {
        return true;
    };
    !index.contains(path, side, u32::try_from(line).unwrap())
}
