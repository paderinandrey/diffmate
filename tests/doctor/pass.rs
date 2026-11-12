use super::support::sandbox::Sandbox;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::process::Stdio;

fn expected() -> String {
    [
        "gh auth: ok",
        "grok: ok",
        "storage: ok",
        "runner lock: ok \u{2014} not held",
        "schema: ok \u{2014} no database yet",
    ]
    .join("\n")
        + "\n"
}

#[test]
fn free_lock_and_no_database_exits_zero() {
    let sandbox = Sandbox::new();
    sandbox.gh.route("auth status", "logged in\n");
    let state = sandbox.state_dir();
    fs::create_dir_all(&state).unwrap();
    let mode = fs::Permissions::from_mode(0o700);
    fs::set_permissions(&state, mode).unwrap();
    let lock_path = state.join("runner.lock");
    let mut lock = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&lock_path)
        .unwrap();
    lock.write_all(b"watch\n").unwrap();
    drop(lock);
    fs::set_permissions(&lock_path, fs::Permissions::from_mode(0o600)).unwrap();

    let output = sandbox
        .diffmate()
        .args(["doctor"])
        .stdin(Stdio::null())
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, expected());
    let held = File::open(&lock_path).unwrap();
    held.try_lock().unwrap();
    drop(held);
    let released = File::open(&lock_path).unwrap();
    released.try_lock().unwrap();
    drop(released);
    assert_eq!(fs::read(&lock_path).unwrap(), b"watch\n");
    assert!(!sandbox.db_path().exists());
    assert!(!sandbox.cache_dir().exists());
}
