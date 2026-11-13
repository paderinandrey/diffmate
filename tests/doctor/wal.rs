use super::schema::{doctor, ready, schema_line};
use rusqlite::Connection;
use std::fs;
use std::path::Path;

#[test]
fn wal_user_version_five_adds_no_directory_entry() {
    let sandbox = ready();
    let conn = Connection::open(sandbox.db_path()).unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA user_version = 5; PRAGMA wal_checkpoint(TRUNCATE);",
    )
    .unwrap();
    drop(conn);
    for name in ["diffmate.db-wal", "diffmate.db-shm"] {
        let path = sandbox.state_dir().join(name);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }
    let before = names(&sandbox.state_dir());
    let (output, stdout) = doctor(&sandbox);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(schema_line(&stdout), "schema: ok");
    assert_eq!(names(&sandbox.state_dir()), before);
}

#[test]
fn open_wal_shows_the_uncheckpointed_user_version() {
    let sandbox = ready();
    let conn = Connection::open(sandbox.db_path()).unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA user_version = 4; PRAGMA wal_checkpoint(TRUNCATE);",
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 5).unwrap();
    let header = fs::read(sandbox.db_path()).unwrap();
    assert_eq!(&header[60..64], &[0, 0, 0, 4]);
    let before = names(&sandbox.state_dir());
    assert!(before.iter().any(|name| name.ends_with("-wal")));
    let (output, stdout) = doctor(&sandbox);
    assert!(output.status.success(), "{output:?} {stdout}");
    assert_eq!(schema_line(&stdout), "schema: ok");
    assert_eq!(names(&sandbox.state_dir()), before);
    drop(conn);
}

fn names(dir: &Path) -> Vec<String> {
    let mut found = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    found.sort();
    found
}
