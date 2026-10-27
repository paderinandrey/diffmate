use crate::agent::Side;
use crate::context::caps::cap_diff;
use crate::diff::parse;
use proptest::prelude::*;
use std::fmt::Write;

type Hunks = Vec<(u32, u32)>;

fn render(files: &[Hunks]) -> String {
    let mut text = String::new();
    for (index, hunks) in files.iter().enumerate() {
        let header = format!("diff --git a/f{index} b/f{index}\n--- a/f{index}\n+++ b/f{index}\n");
        text.push_str(&header);
        let mut next = 1;
        for (gap, count) in hunks {
            let start = next + gap;
            writeln!(text, "@@ -{start},{count} +{start},{count} @@").unwrap();
            text += &" line\n".repeat(*count as usize);
            next = start + count;
        }
    }
    text
}

fn diffs() -> impl Strategy<Value = Vec<Hunks>> {
    let hunk = (0..5u32, 1..6u32);
    prop::collection::vec(prop::collection::vec(hunk, 1..4), 1..5)
}

proptest! {
    #[test]
    fn the_kept_part_is_a_bounded_prefix_that_adds_no_anchor(files in diffs(), extra in 0..80usize) {
        let diff = render(&files);
        let cap = diff.len().saturating_sub(extra);
        let capped = cap_diff(diff.as_bytes(), cap);
        if diff.len() <= cap {
            prop_assert_eq!(&capped, diff.as_bytes());
        } else {
            let newline = capped.iter().position(|b| *b == b'\n').unwrap();
            let (marker, kept) = capped.split_at(newline + 1);
            prop_assert!(marker.starts_with(b"TRUNCATED: diff.patch kept "));
            prop_assert!(kept.len() <= cap);
            prop_assert!(diff.as_bytes().starts_with(kept));
            let rest = &diff.as_bytes()[kept.len()..];
            prop_assert!(rest.starts_with(b"diff --git ") || rest.starts_with(b"@@ "));
            let (full, cut) = (parse(diff.as_bytes()), parse(&capped));
            for index in 0..files.len() {
                for line in 1..80 {
                    for side in [Side::Left, Side::Right] {
                        let path = format!("f{index}");
                        prop_assert!(!cut.contains(&path, side, line) || full.contains(&path, side, line));
                    }
                }
            }
        }
    }
}
