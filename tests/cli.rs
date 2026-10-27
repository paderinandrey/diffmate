mod support;

use support::sandbox::Sandbox;

#[test]
fn no_arguments_is_a_usage_error() {
    let output = Sandbox::new().run(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage"));
}

#[test]
fn version_prints_name_and_version() {
    let output = Sandbox::new().run(&["--version"]);

    assert!(output.status.success());
    let expected = format!("diffmate {}\n", env!("CARGO_PKG_VERSION"));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn review_rejects_a_non_github_url_before_touching_the_database() {
    let sandbox = Sandbox::new();

    let output = sandbox.run(&["review", "https://example.com/o/r/pull/1"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a github.com pull request URL"));
    assert!(!sandbox.db_path().exists());
}
