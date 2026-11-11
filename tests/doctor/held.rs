use super::support::sandbox::Sandbox;
use rusqlite::Connection;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::process::Stdio;

fn private_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

fn names(dir: &Path) -> Vec<String> {
    let mut found = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    found.sort();
    found
}

#[test]
fn held_watch_lock_and_schema_five_exit_zero() {
    let sandbox = Sandbox::new();
    sandbox.gh.route("auth status", "logged in\n");
    let state = sandbox.state_dir();
    private_dir(&state);
    private_dir(&sandbox.cache_dir());
    let lock_path = state.join("runner.lock");
    let mut lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&lock_path)
        .unwrap();
    lock.write_all(b"watch\n").unwrap();
    fs::set_permissions(&lock_path, fs::Permissions::from_mode(0o600)).unwrap();
    lock.try_lock().unwrap();
    let db = sandbox.db_path();
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch("PRAGMA user_version = 5").unwrap();
    drop(conn);
    let before = fs::read(&db).unwrap();
    let entries = names(&state);

    let output = sandbox
        .diffmate()
        .args(["doctor"])
        .stdin(Stdio::null())
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        "\
gh auth: ok
claude: ok
storage: ok
runner lock: ok \u{2014} held by watch
schema: ok
"
    );
    let probe = File::open(&lock_path).unwrap();
    assert!(matches!(probe.try_lock(), Err(TryLockError::WouldBlock)));
    drop(probe);
    assert_eq!(fs::read(&db).unwrap(), before);
    assert!(!state.join("diffmate.db.v1-backup").exists());
    assert_eq!(names(&state), entries);
    drop(lock);
}
