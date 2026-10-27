use super::{MIN_VERSION, PROBE_OK_TEXT, interpret_probe, parse_version};
use proptest::prelude::*;

#[test]
fn parses_the_version_line() {
    assert_eq!(parse_version("2.1.304 (Claude Code)"), Some((2, 1, 304)));
    assert_eq!(parse_version("2.1.304 (Claude Code)\n"), Some((2, 1, 304)));
}

#[test]
fn compares_numerically_not_as_text() {
    assert!(parse_version("2.1.40 (Claude Code)").unwrap() < MIN_VERSION);
    assert!(parse_version("2.1.1000 (Claude Code)").unwrap() > MIN_VERSION);
    assert!(parse_version("2.2.0 (Claude Code)").unwrap() > MIN_VERSION);
    assert!(parse_version("10.0.0 (Claude Code)").unwrap() > MIN_VERSION);
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

fn probe(stderr: &str, exit: Option<i32>) -> Result<(), String> {
    interpret_probe(b"", stderr.as_bytes(), exit).map_err(|detail| detail.message)
}

#[test]
fn expected_input_error_is_the_only_pass() {
    assert!(probe(PROBE_OK_TEXT, Some(1)).is_ok());
    assert!(probe(&format!("{PROBE_OK_TEXT}\n"), Some(2)).is_ok());
}

#[test]
fn unknown_option_names_the_flag() {
    let text = "error: unknown option '--disable-slash-commands'\n";
    let message = probe(text, Some(1)).unwrap_err();
    assert!(message.contains("--disable-slash-commands"), "{message}");
}

#[test]
fn rejected_schema_and_missing_prompt_file_are_named() {
    let schema = "Error: --json-schema is not a valid JSON Schema: no schema with key or ref";
    assert!(
        probe(schema, Some(1))
            .unwrap_err()
            .contains("--json-schema")
    );
    let prompt = "Error: Append system prompt file not found: /nonexistent/prompt.md";
    assert!(probe(prompt, Some(1)).unwrap_err().contains("prompt file"));
}

#[test]
fn anything_else_is_inconclusive() {
    for (text, exit) in [
        ("", Some(1)),
        ("boom", Some(1)),
        (PROBE_OK_TEXT, Some(0)),
        (PROBE_OK_TEXT, None),
    ] {
        let message = probe(text, exit).unwrap_err();
        assert!(message.starts_with("flag probe inconclusive"), "{message}");
    }
}
