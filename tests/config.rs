mod support;
#[allow(dead_code)]
mod tracer;

use support::sandbox::Sandbox;

fn write_config(sandbox: &Sandbox, text: &str) {
    let dir = sandbox.home.join(".config/diffmate");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("config.toml"), text).unwrap();
}

#[test]
fn invalid_config_stops_review_before_enqueue() {
    let sandbox = Sandbox::new();
    tracer::with_pr(&sandbox);
    write_config(&sandbox, "# invalid timeout\nrun_timeout_minutes = 0\n");
    let output = tracer::review(&sandbox);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("config.toml:2: run_timeout_minutes:"),
        "{error}"
    );
    assert_eq!(error.lines().count(), 1);
    assert!(!sandbox.db_path().exists());
    assert!(sandbox.claude.calls().is_empty());
    assert_eq!(sandbox.gh.calls(), Vec::<Vec<String>>::new());
}

#[test]
fn missing_config_uses_defaults() {
    let sandbox = Sandbox::new();
    tracer::with_pr(&sandbox);
    let output = tracer::review(&sandbox);
    assert!(output.status.success(), "{output:?}");
}
