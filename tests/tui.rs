mod support;

use support::sandbox::Sandbox;

#[test]
fn tui_without_a_terminal_exits_before_creating_a_database() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(&["tui"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: diffmate tui needs a terminal\n"
    );
    assert_eq!(output.stdout, b"");
    assert!(!sandbox.db_path().exists());
    assert!(!sandbox.state_dir().exists());
}
