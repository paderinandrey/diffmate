use super::{PATH, Path, parse};
use std::error::Error;

#[test]
fn unknown_keys_and_tables_name_the_field_and_line() {
    for (text, key) in [
        ("\n\npoll_seconds = 5", "poll_seconds"),
        ("\n\n[watch]", "watch"),
    ] {
        let error = parse(text, Path::new(PATH)).unwrap_err();
        assert_eq!(error.line, Some(3));
        assert_eq!(error.key.as_deref(), Some(key));
        assert_eq!(
            error.to_string(),
            format!("{PATH}:3: {key}: {}", error.message)
        );
        assert!(error.source().is_some());
    }
}

#[test]
fn invalid_values_name_the_field_and_line() {
    for (key, values) in [
        (
            "run_timeout_minutes",
            vec!["\"15\"", "1.5", "0", "-1", "9223372036854775807"],
        ),
        ("debounce_seconds", vec!["0", "-1", "true"]),
        ("poll_interval_seconds", vec!["0", "-1", "[]"]),
        ("max_retries", vec!["-1", "4294967296", "{}"]),
    ] {
        for value in values {
            let text = format!("# first\n\n{key} = {value}\n");
            let error = parse(&text, Path::new(PATH)).unwrap_err();
            assert_eq!(error.line, Some(3), "{text}: {error:?}");
            assert_eq!(error.key.as_deref(), Some(key), "{text}: {error:?}");
        }
    }
}

#[test]
fn syntax_errors_keep_the_key_and_line_when_an_assignment_exists() {
    for (text, line) in [
        ("run_timeout_minutes = 1\nrun_timeout_minutes = 2", 2),
        ("# config\nrun_timeout_minutes =", 2),
        ("# config\nrun_timeout_minutes =\n", 2),
        ("# config\nrun_timeout_minutes = 9223372036854775808", 2),
    ] {
        let error = parse(text, Path::new(PATH)).unwrap_err();
        assert_eq!(error.line, Some(line), "{text}: {error:?}");
        assert_eq!(
            error.key.as_deref(),
            Some("run_timeout_minutes"),
            "{text}: {error:?}"
        );
    }
}

#[test]
fn integer_boundaries_are_checked_without_overflow() {
    let maximum = u64::MAX / 60;
    let settings = parse(&format!("run_timeout_minutes = {maximum}"), Path::new(PATH)).unwrap();
    assert_eq!(settings.run_timeout().as_secs(), maximum * 60);
    assert!(
        parse(
            &format!("run_timeout_minutes = {}", maximum + 1),
            Path::new(PATH)
        )
        .is_err()
    );
    let settings = parse("max_retries = 4294967295", Path::new(PATH)).unwrap();
    assert_eq!(settings.max_retries(), u32::MAX);
}

#[test]
fn quoted_keys_keep_their_decoded_names() {
    for (text, key) in [
        ("\"run_timeout_minutes\" =", "run_timeout_minutes"),
        ("'run_timeout_minutes' =", "run_timeout_minutes"),
        ("\"odd=key\" =", "odd=key"),
        ("[\"odd`table\"]", "odd`table"),
    ] {
        let error = parse(text, Path::new(PATH)).unwrap_err();
        assert_eq!(error.line, Some(1));
        assert_eq!(error.key.as_deref(), Some(key), "{text}: {error:?}");
    }
}
