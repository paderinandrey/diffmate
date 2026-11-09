use super::super::{Settings, load};
use std::error::Error;
use std::fs;

#[test]
fn missing_file_uses_defaults_and_partial_file_is_loaded() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.toml");
    assert_eq!(load(&path).unwrap(), Settings::default());
    fs::write(&path, "max_retries = 0").unwrap();
    assert_eq!(load(&path).unwrap().max_retries(), 0);
}

#[test]
fn unreadable_file_is_an_error_with_its_source() {
    let root = tempfile::tempdir().unwrap();
    let error = load(root.path()).unwrap_err();
    assert_eq!(error.path, root.path());
    assert_eq!(error.line, None);
    assert_eq!(error.key, None);
    assert!(error.source().is_some());
    assert!(
        error
            .to_string()
            .starts_with(&format!("{}: ", root.path().display()))
    );
}

#[test]
fn invalid_utf8_is_not_treated_as_a_missing_file() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.toml");
    fs::write(&path, [0xff, 0xfe]).unwrap();
    assert!(load(&path).is_err());
}
