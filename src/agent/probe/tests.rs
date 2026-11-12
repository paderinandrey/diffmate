use super::{HELP_FLAGS, MIN_VERSION, interpret_help, parse_version};
use proptest::prelude::*;

#[test]
fn parses_the_version_line() {
    assert_eq!(parse_version("2.1.304 (Claude Code)"), Some((2, 1, 304)));
    assert_eq!(
        parse_version("grok 1.0.77 (50ca579fa996) [stable]\n"),
        Some((1, 0, 77))
    );
}

#[test]
fn compares_numerically_not_as_text() {
    assert!(parse_version("1.0.76").unwrap() < MIN_VERSION);
    assert!(parse_version("1.0.1000").unwrap() > MIN_VERSION);
    assert!(parse_version("1.1.0").unwrap() > MIN_VERSION);
    assert!(parse_version("2.0.0").unwrap() > MIN_VERSION);
}

#[test]
fn garbage_is_not_a_version() {
    for text in [
        "", "garbage", "2.1", "2.1.x", "2.1.3.4", "+2.1.3", "-1.2.3", "v2.1.3",
    ] {
        assert_eq!(parse_version(text), None, "{text:?}");
    }
}

proptest! {
    #[test]
    fn version_triples_round_trip(a in any::<u32>(), b in any::<u32>(), c in any::<u32>()) {
        prop_assert_eq!(parse_version(&format!("{a}.{b}.{c} (Claude Code)")), Some((a, b, c)));
    }

    #[test]
    fn arbitrary_text_never_panics(text in ".*") {
        let _ = parse_version(&text);
    }
}

fn help(stdout: &str, exit: Option<i32>) -> Result<(), String> {
    interpret_help(stdout.as_bytes(), exit).map_err(|detail| detail.message)
}

#[test]
fn help_that_lists_every_flag_passes() {
    assert!(help(&HELP_FLAGS.join("\n"), Some(0)).is_ok());
}

#[test]
fn help_missing_a_flag_names_it() {
    let text = HELP_FLAGS.join("\n").replace("--json-schema", "");
    let message = help(&text, Some(0)).unwrap_err();
    assert!(message.contains("--json-schema"), "{message}");
}

#[test]
fn help_that_does_not_exit_zero_is_inconclusive() {
    for exit in [Some(1), None] {
        let message = help("nope", exit).unwrap_err();
        assert!(message.starts_with("flag probe inconclusive"), "{message}");
    }
}
