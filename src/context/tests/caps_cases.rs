use crate::context::caps::{cap_diff, cap_text};

const DIFF: &str = "diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1 +1 @@\n-x\n+y\ndiff --git a/b b/b\n--- a/b\n+++ b/b\n@@ -1 +1 @@\n-p\n+q\n";

#[test]
fn text_under_the_cap_is_unchanged() {
    assert_eq!(cap_text("pr.md", "aaa\nbbb\n", 8), "aaa\nbbb\n");
}

#[test]
fn text_over_the_cap_is_cut_at_the_last_newline_with_a_first_line_marker() {
    let capped = cap_text("pr.md", "aaa\nbbb\nccc\n", 9);
    assert_eq!(capped, "TRUNCATED: pr.md kept 8 of 12 bytes.\naaa\nbbb\n");
}

#[test]
fn text_is_cut_on_a_char_boundary_when_it_has_no_newline() {
    let capped = cap_text("pr.md", "éé\néé", 3);
    assert_eq!(capped, "TRUNCATED: pr.md kept 2 of 9 bytes.\né");
}

#[test]
fn a_diff_under_the_cap_is_unchanged() {
    assert_eq!(cap_diff(DIFF.as_bytes(), DIFF.len()), DIFF.as_bytes());
}

fn capped_at(cap: usize, kept: usize) {
    let capped = String::from_utf8(cap_diff(DIFF.as_bytes(), cap)).unwrap();
    let total = DIFF.len();
    let marker =
        format!("TRUNCATED: diff.patch kept {kept} of {total} bytes, cut at a hunk boundary.\n");
    assert_eq!(capped, format!("{marker}{}", &DIFF[..kept]));
}

#[test]
fn a_diff_over_the_cap_is_cut_before_the_last_unit_start_within_the_cap() {
    let (second_file, last_hunk) = (
        DIFF.find("diff --git a/b").unwrap(),
        DIFF.rfind("@@ ").unwrap(),
    );
    capped_at(DIFF.len() - 1, last_hunk);
    capped_at(second_file + 5, second_file);
    capped_at(last_hunk, last_hunk);
}

#[test]
fn a_cap_inside_the_first_header_keeps_nothing() {
    let capped = String::from_utf8(cap_diff(DIFF.as_bytes(), 5)).unwrap();
    assert!(capped.starts_with("TRUNCATED: diff.patch kept 0 of "));
    assert!(capped.ends_with("cut at a hunk boundary.\n"));
}
