mod backups;
mod concurrency;
mod helpers;
mod rollback;
mod v3;

use crate::error::Error;
use crate::paths::Paths;
use crate::store::Store;
use helpers::{contents, fixture, schema, version};
use rusqlite::Connection;
use std::fs;

#[tokio::test]
async fn fresh_and_upgraded_schemas_match() {
    let (_tmp, upgraded) = fixture(1);
    let fresh = Paths::new(upgraded.state_dir().join("fresh"), upgraded.runs_dir());
    let _fresh_store = Store::open(&fresh).await.unwrap();
    let _upgraded_store = Store::open(&upgraded).await.unwrap();
    assert_eq!(
        schema(&Connection::open(fresh.db()).unwrap()),
        schema(&Connection::open(upgraded.db()).unwrap())
    );
    assert!(!fresh.state_dir().join("diffmate.db.v1-backup").exists());
    assert!(!fresh.state_dir().join("diffmate.db.v2-backup").exists());
}

#[tokio::test]
async fn newer_version_is_refused_and_left_alone() {
    let (_tmp, paths) = fixture(4);
    let before = fs::read(paths.db()).unwrap();
    assert!(matches!(
        Store::open(&paths).await,
        Err(Error::SchemaVersion(4))
    ));
    assert_eq!(fs::read(paths.db()).unwrap(), before);
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 4);
    assert!(!paths.state_dir().join("diffmate.db.v1-backup").exists());
    assert!(!paths.state_dir().join("diffmate.db.v2-backup").exists());
}

#[tokio::test]
async fn reopening_migrated_database_changes_nothing() {
    let (_tmp, paths) = fixture(1);
    drop(Store::open(&paths).await.unwrap());
    let conn = Connection::open(paths.db()).unwrap();
    let before = (schema(&conn), contents(&conn));
    drop(Store::open(&paths).await.unwrap());
    assert_eq!((schema(&conn), contents(&conn)), before);
    assert_eq!(version(&conn), 3);
}

#[tokio::test]
async fn check_constraints_hold() {
    let (_tmp, paths) = fixture(1);
    let _store = Store::open(&paths).await.unwrap();
    let conn = Connection::open(paths.db()).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    for assignment in [
        "origin = 'robot'",
        "force = 2",
        "stop_reason = 'later'",
        "retry_count = -1",
        "max_retries = -1",
        "result_run_id = 999",
    ] {
        let error = conn
            .execute(&format!("UPDATE jobs SET {assignment}"), [])
            .unwrap_err();
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }
    let seen = conn
        .execute("UPDATE targets SET request_seen = 2", [])
        .unwrap_err();
    assert_eq!(
        seen.sqlite_error_code(),
        Some(rusqlite::ErrorCode::ConstraintViolation)
    );
}
