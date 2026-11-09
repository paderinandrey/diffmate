use super::helpers::{contents, fixture, version};
use crate::store::Store;
use rusqlite::Connection;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn backup_preserves_version_one_rows_and_is_not_rewritten() {
    let (_tmp, paths) = fixture(1);
    let conn = Connection::open(paths.db()).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();
    conn.execute("UPDATE findings SET title = 'Last WAL write'", [])
        .unwrap();
    let before = contents(&conn);
    let backup = paths.state_dir().join("diffmate.db.v1-backup");
    drop(Store::open(&paths).await.unwrap());
    let saved = Connection::open(&backup).unwrap();
    assert_eq!(version(&saved), 1);
    assert_eq!(contents(&saved), before);
    assert_eq!(
        fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let bytes = fs::read(&backup).unwrap();
    let modified = fs::metadata(&backup).unwrap().modified().unwrap();
    drop(Store::open(&paths).await.unwrap());
    assert_eq!(fs::read(&backup).unwrap(), bytes);
    assert_eq!(fs::metadata(&backup).unwrap().modified().unwrap(), modified);
}

#[tokio::test]
async fn interrupted_temporary_backup_is_replaced() {
    let (_tmp, paths) = fixture(1);
    let temporary = paths.state_dir().join("diffmate.db.v1-backup.tmp");
    fs::write(&temporary, b"partial SQLite output").unwrap();
    drop(Store::open(&paths).await.unwrap());
    assert!(!temporary.exists());
    let saved = Connection::open(paths.state_dir().join("diffmate.db.v1-backup")).unwrap();
    assert_eq!(version(&saved), 1);
    assert_eq!(contents(&saved)[0].len(), 1);
}

#[tokio::test]
async fn published_backup_before_interruption_is_kept_and_temporary_removed() {
    let (_tmp, paths) = fixture(1);
    let backup = paths.state_dir().join("diffmate.db.v1-backup");
    let temporary = paths.state_dir().join("diffmate.db.v1-backup.tmp");
    let conn = Connection::open(paths.db()).unwrap();
    conn.execute("VACUUM INTO ?1", [backup.to_str().unwrap()])
        .unwrap();
    fs::hard_link(&backup, &temporary).unwrap();
    let before = fs::read(&backup).unwrap();
    drop(Store::open(&paths).await.unwrap());
    assert_eq!(version(&conn), 3);
    assert!(!paths.state_dir().join("diffmate.db.v2-backup").exists());
    assert_eq!(fs::read(&backup).unwrap(), before);
    assert!(!temporary.exists());
}

#[tokio::test]
async fn corrupt_backup_prevents_upgrade_and_is_kept() {
    let (_tmp, paths) = fixture(1);
    let backup = paths.state_dir().join("diffmate.db.v1-backup");
    fs::write(&backup, b"partial SQLite output").unwrap();
    assert!(Store::open(&paths).await.is_err());
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 1);
    assert_eq!(fs::read(&backup).unwrap(), b"partial SQLite output");
}
