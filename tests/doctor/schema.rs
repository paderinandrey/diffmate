use super::support::sandbox::Sandbox;
use rusqlite::{Connection, OpenFlags};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::process::Output;

pub(super) fn ready() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.gh.route("auth status", "logged in\n");
    for dir in [sandbox.state_dir(), sandbox.cache_dir()] {
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let mut lock = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(sandbox.state_dir().join("runner.lock"))
        .unwrap();
    lock.write_all(b"watch\n").unwrap();
    sandbox
}

fn set_version(sandbox: &Sandbox, version: i64) -> Vec<u8> {
    let conn = Connection::open(sandbox.db_path()).unwrap();
    conn.pragma_update(None, "user_version", version).unwrap();
    drop(conn);
    fs::read(sandbox.db_path()).unwrap()
}

pub(super) fn doctor(sandbox: &Sandbox) -> (Output, String) {
    let output = sandbox.run(&["doctor"]);
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert_eq!(stdout.lines().count(), 5, "{stdout}");
    (output, stdout)
}

pub(super) fn schema_line(stdout: &str) -> &str {
    stdout.lines().nth(4).unwrap()
}

fn user_version(sandbox: &Sandbox) -> i64 {
    let conn =
        Connection::open_with_flags(sandbox.db_path(), OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn user_version_one_is_not_migrated() {
    let sandbox = ready();
    let before = set_version(&sandbox, 1);
    let (output, stdout) = doctor(&sandbox);
    assert_eq!(output.status.code(), Some(1));
    let schema = schema_line(&stdout);
    assert!(schema.contains("database schema version 1"), "{schema}");
    assert!(schema.contains("run `diffmate review` once"), "{schema}");
    assert_eq!(fs::read(sandbox.db_path()).unwrap(), before);
    assert!(!sandbox.state_dir().join("diffmate.db.v1-backup").exists());
    assert_eq!(user_version(&sandbox), 1);
}

#[test]
fn user_version_six_is_not_rewritten() {
    let sandbox = ready();
    let before = set_version(&sandbox, 6);
    let (output, stdout) = doctor(&sandbox);
    assert_eq!(output.status.code(), Some(1));
    let schema = schema_line(&stdout);
    assert!(
        schema.contains("newer than this diffmate supports"),
        "{schema}"
    );
    assert_eq!(fs::read(sandbox.db_path()).unwrap(), before);
    assert_eq!(user_version(&sandbox), 6);
}

#[test]
fn user_version_zero_is_not_a_missing_database() {
    let sandbox = ready();
    let before = set_version(&sandbox, 0);
    assert!(sandbox.db_path().exists());
    let (output, stdout) = doctor(&sandbox);
    assert_eq!(output.status.code(), Some(1));
    assert!(!stdout.contains("no database yet"), "{stdout}");
    let schema = schema_line(&stdout);
    assert!(schema.contains("database schema version 0"), "{schema}");
    assert!(schema.contains("run `diffmate review` once"), "{schema}");
    assert_eq!(fs::read(sandbox.db_path()).unwrap(), before);
}
