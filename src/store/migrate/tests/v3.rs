use super::helpers::version;
use crate::error::Error;
use crate::paths::Paths;
use crate::store::Store;
use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

fn version_two() -> (TempDir, Paths) {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    fs::create_dir_all(paths.state_dir()).unwrap();
    let conn = Connection::open(paths.db()).unwrap();
    conn.execute_batch(include_str!("../../schema.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../../migrate_v2.sql"))
        .unwrap();
    conn.execute(
        "INSERT INTO targets (owner, repo, number, tracking, created_at)
         VALUES ('o', 'r', 1, 'tracked', 1)",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 2).unwrap();
    (tmp, paths)
}

fn any_backup(paths: &Paths) -> bool {
    fs::read_dir(paths.state_dir())
        .unwrap()
        .flatten()
        .any(|entry| entry.file_name().to_string_lossy().contains("backup"))
}

#[tokio::test]
async fn version_two_keeps_tracking_and_snapshots_that_version() {
    let (_tmp, paths) = version_two();
    drop(Store::open(&paths).await.unwrap());
    let conn = Connection::open(paths.db()).unwrap();
    assert_eq!(version(&conn), 5);
    assert!(!paths.state_dir().join("diffmate.db.v4-backup").exists());
    let row: (i64, String, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT request_seen, tracking, queued_head, head_sha FROM targets",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(row, (0, "tracked".to_string(), None, None));
    let saved = Connection::open(paths.state_dir().join("diffmate.db.v2-backup")).unwrap();
    assert_eq!(version(&saved), 2);
    assert!(!paths.state_dir().join("diffmate.db.v1-backup").exists());
}

#[tokio::test]
async fn fresh_database_has_no_backup() {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    drop(Store::open(&paths).await.unwrap());
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 5);
    assert!(!any_backup(&paths));
}

#[tokio::test]
async fn read_only_version_two_is_outdated_without_a_backup() {
    let (_tmp, paths) = version_two();
    assert!(matches!(
        Store::open_read_only(&paths).await,
        Err(Error::SchemaOutdated(2))
    ));
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 2);
    assert!(!any_backup(&paths));
}

#[tokio::test]
async fn version_three_snapshots_that_version_and_adds_attempt_columns() {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    fs::create_dir_all(paths.state_dir()).unwrap();
    let conn = Connection::open(paths.db()).unwrap();
    conn.execute_batch(include_str!("../../schema.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../../migrate_v2.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../../migrate_v3.sql"))
        .unwrap();
    conn.pragma_update(None, "user_version", 3).unwrap();
    drop(conn);
    assert!(matches!(
        Store::open_read_only(&paths).await,
        Err(Error::SchemaOutdated(3))
    ));
    drop(Store::open(&paths).await.unwrap());
    let conn = Connection::open(paths.db()).unwrap();
    assert_eq!(version(&conn), 5);
    assert!(!paths.state_dir().join("diffmate.db.v4-backup").exists());
    let columns: i64 = conn
        .query_row(
            "SELECT count(*) FROM pragma_table_info('jobs') WHERE name IN ('head_sha', 'held')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(columns, 2);
    let saved = Connection::open(paths.state_dir().join("diffmate.db.v3-backup")).unwrap();
    assert_eq!(version(&saved), 3);
    assert!(!paths.state_dir().join("diffmate.db.v2-backup").exists());
}
