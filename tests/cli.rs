use std::process::Command;

#[test]
fn prints_name_and_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_diffmate"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("diffmate {}\n", env!("CARGO_PKG_VERSION"))
    );
}
