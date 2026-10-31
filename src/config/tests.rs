mod errors;
mod files;

use super::{Settings, parse};
use proptest::prelude::*;
use std::path::Path;

const PATH: &str = "/home/u/.config/diffmate/config.toml";

#[test]
fn empty_and_comment_only_inputs_use_spec_defaults() {
    let defaults = Settings::default();
    assert_eq!(defaults.run_timeout().as_secs(), 900);
    assert_eq!(defaults.max_retries(), 2);
    assert_eq!(defaults.debounce_seconds, 60);
    assert_eq!(defaults.poll_interval_seconds, 60);
    for text in ["", "# owner settings\n\n", "\r\n# comment\r\n"] {
        assert_eq!(parse(text, Path::new(PATH)).unwrap(), defaults);
    }
}

#[test]
fn each_key_overrides_only_its_default() {
    for (key, expected) in [
        (
            "run_timeout_minutes",
            Settings {
                run_timeout_minutes: 7,
                ..Settings::default()
            },
        ),
        (
            "debounce_seconds",
            Settings {
                debounce_seconds: 7,
                ..Settings::default()
            },
        ),
        (
            "poll_interval_seconds",
            Settings {
                poll_interval_seconds: 7,
                ..Settings::default()
            },
        ),
        (
            "max_retries",
            Settings {
                max_retries: 7,
                ..Settings::default()
            },
        ),
    ] {
        assert_eq!(
            parse(&format!("{key} = 7"), Path::new(PATH)).unwrap(),
            expected
        );
    }
}

#[test]
fn all_four_keys_apply_and_zero_retries_are_allowed() {
    let text =
        "run_timeout_minutes = 1\ndebounce_seconds = 2\npoll_interval_seconds = 3\nmax_retries = 0";
    let settings = parse(text, Path::new(PATH)).unwrap();
    assert_eq!(settings.run_timeout().as_secs(), 60);
    assert_eq!(settings.max_retries(), 0);
    assert_eq!(settings.debounce_seconds, 2);
    assert_eq!(settings.poll_interval_seconds, 3);
}

proptest! {
    #[test]
    fn duration_ranges_match_their_units(value in any::<i64>()) {
        for (key, maximum) in [
            ("run_timeout_minutes", u64::MAX / 60),
            ("debounce_seconds", u64::MAX),
            ("poll_interval_seconds", u64::MAX),
        ] {
            let result = parse(&format!("{key} = {value}"), Path::new(PATH));
            let valid = u64::try_from(value).is_ok_and(|v| (1..=maximum).contains(&v));
            prop_assert_eq!(result.is_ok(), valid);
        }
    }

    #[test]
    fn retry_range_matches_u32(value in any::<i64>()) {
        let result = parse(&format!("max_retries = {value}"), Path::new(PATH));
        prop_assert_eq!(result.is_ok(), u32::try_from(value).is_ok());
    }

    #[test]
    fn arbitrary_text_never_panics(text in any::<String>()) {
        if let Err(error) = parse(&text, Path::new(PATH))
            && let Some(line) = error.line {
            prop_assert!((1..=text.split('\n').count()).contains(&line));
        }
    }
}
