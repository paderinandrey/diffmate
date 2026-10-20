mod model;
mod never_panics;
mod place;
mod roundtrip;

use super::{DiffIndex, parse};
use crate::agent::Side;

const MODIFIED: &str = "diff --git a/src/lib.rs b/src/lib.rs\nindex 1..2 100644\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-old\n+new\n";

fn lines(index: &DiffIndex, path: &str, side: Side) -> Vec<u32> {
    (0..=30)
        .filter(|n| index.contains(path, side, *n))
        .collect()
}

fn both(text: &str, path: &str) -> (Vec<u32>, Vec<u32>) {
    let index = parse(text.as_bytes());
    (
        lines(&index, path, Side::Left),
        lines(&index, path, Side::Right),
    )
}

#[test]
fn omitted_counts_mean_one_line_on_each_side() {
    assert_eq!(both(MODIFIED, "src/lib.rs"), (vec![1], vec![1]));
}

#[test]
fn a_new_file_has_right_lines_only() {
    let text = "diff --git a/n.rs b/n.rs\nnew file mode 100644\n--- /dev/null\n+++ b/n.rs\n@@ -0,0 +1,3 @@\n+a\n+b\n+c\n";
    assert_eq!(both(text, "n.rs"), (vec![], vec![1, 2, 3]));
}

#[test]
fn a_deleted_file_is_keyed_by_its_old_path_with_left_lines_only() {
    let text = "diff --git a/gone.rs b/gone.rs\ndeleted file mode 100644\n--- a/gone.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-a\n-b\n";
    assert_eq!(both(text, "gone.rs"), (vec![1, 2], vec![]));
    assert_eq!(both(text, "/dev/null"), (vec![], vec![]));
}

#[test]
fn a_rename_with_hunks_is_keyed_by_the_new_path() {
    let text = "diff --git a/old.rs b/new.rs\nrename from old.rs\nrename to new.rs\n--- a/old.rs\n+++ b/new.rs\n@@ -2,2 +2,2 @@\n a\n-b\n+c\n";
    assert_eq!(both(text, "new.rs"), (vec![2, 3], vec![2, 3]));
    assert_eq!(both(text, "old.rs"), (vec![], vec![]));
}

#[test]
fn sections_without_hunks_add_nothing() {
    let text = "diff --git a/o b/n\nsimilarity index 100%\nrename from o\nrename to n\ndiff --git a/m b/m\nold mode 100644\nnew mode 100755\ndiff --git a/i.png b/i.png\nBinary files a/i.png and b/i.png differ\n";
    assert!(parse(text.as_bytes()).files.is_empty());
}

#[test]
fn no_newline_markers_do_not_shift_line_numbers() {
    let marker = "\\ No newline at end of file\n";
    let text = format!(
        "diff --git a/f b/f\n--- a/f\n+++ b/f\n@@ -1,2 +1,2 @@\n a\n-b\n{marker}+c\n{marker}@@ -10 +10 @@\n-x\n+y\n"
    );
    assert_eq!(both(&text, "f"), (vec![1, 2, 10], vec![1, 2, 10]));
}

#[test]
fn quoted_and_tab_terminated_paths_are_decoded() {
    let quoted = "diff --git \"a/caf\\303\\251.txt\" \"b/caf\\303\\251.txt\"\n--- \"a/caf\\303\\251.txt\"\n+++ \"b/caf\\303\\251.txt\"\n@@ -1 +1 @@\n-a\n+b\n";
    assert_eq!(both(quoted, "café.txt"), (vec![1], vec![1]));
    let spaced = "diff --git a/my file.txt b/my file.txt\n--- a/my file.txt\t\n+++ b/my file.txt\t\n@@ -1 +1 @@\n-a\n+b\n";
    assert_eq!(both(spaced, "my file.txt"), (vec![1], vec![1]));
}

#[test]
fn lines_before_the_first_diff_header_are_ignored() {
    let text = format!("TRUNCATED: cut after 10 bytes\n{MODIFIED}");
    assert_eq!(both(&text, "src/lib.rs"), (vec![1], vec![1]));
}

#[test]
fn content_that_looks_like_headers_stays_content() {
    let text = format!("{MODIFIED}@@ -5 +5 @@\n--- a\n+++ b\n");
    assert_eq!(both(&text, "src/lib.rs"), (vec![1, 5], vec![1, 5]));
    assert_eq!(parse(text.as_bytes()).files.len(), 1);
}
