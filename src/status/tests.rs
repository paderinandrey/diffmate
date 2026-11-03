use super::{freshness, when};
use freshness::LiveHead;
use proptest::prelude::*;

mod fixture;
mod stored;

#[test]
fn elapsed_times_use_whole_units_and_clamp_clock_skew() {
    for (elapsed, expected) in [
        (-1, "just now"),
        (0, "just now"),
        (59_999, "just now"),
        (60_000, "1m ago"),
        (3_599_999, "59m ago"),
        (3_600_000, "1h ago"),
        (86_400_000, "1d ago"),
    ] {
        assert_eq!(when::ago(elapsed), expected);
    }
}

#[test]
fn future_times_use_whole_units_and_due_times_are_now() {
    for (remaining, expected) in [
        (-1, "now"),
        (0, "now"),
        (1_000, "in 1s"),
        (59_999, "in 59s"),
        (60_000, "in 1m"),
        (3_600_000, "in 1h"),
    ] {
        assert_eq!(when::until(remaining), expected);
    }
}

fn represented_ms(text: &str) -> i64 {
    if text == "just now" {
        return 0;
    }
    let quantity = text.strip_suffix(" ago").unwrap();
    let (number, unit) = quantity.split_at(quantity.len() - 1);
    assert!(number.bytes().all(|byte| byte.is_ascii_digit()));
    let factor = match unit {
        "m" => 60_000,
        "h" => 3_600_000,
        "d" => 86_400_000,
        _ => panic!("invalid unit"),
    };
    number.parse::<i64>().unwrap() * factor
}

proptest! {
    #[test]
    fn elapsed_time_labels_are_valid_and_never_go_backwards(a in 0i64..315_360_000_000, b in 0i64..315_360_000_000) {
        let earlier = represented_ms(&when::ago(a.min(b)));
        let later = represented_ms(&when::ago(a.max(b)));
        prop_assert!(earlier <= later);
        prop_assert!(later <= a.max(b));
    }
}

#[test]
fn failed_checks_compare_the_stored_head_and_name_its_age() {
    let head = "9f3c2a1b4e5d0000000000000000000000000000";
    for (draft, state) in [(head, "current"), ("older", "stale")] {
        assert_eq!(
            freshness::describe(draft, &LiveHead::Failed, Some((head, 20_000)), 200_000),
            format!("{state}: head 9f3c2a1b4e5d, checked 3m ago (live check failed)")
        );
    }
    assert_eq!(
        freshness::describe(head, &LiveHead::Failed, None, 200_000),
        "freshness unknown: head never checked (live check failed)"
    );
}

#[test]
fn successful_checks_override_the_stored_head() {
    for (live, state) in [("draft", "current"), ("newer", "stale")] {
        assert_eq!(
            freshness::describe(
                "draft",
                &LiveHead::Fetched(live.into()),
                Some(("old", 0)),
                200_000
            ),
            format!("{state}: head {live}, checked just now")
        );
    }
}

#[test]
fn row_output_removes_terminal_controls_and_line_breaks() {
    assert_eq!(
        super::line(["o/\u{202e}r#1", "a\n\tb\u{1b}"]),
        "o/r#1  a  b"
    );
}
