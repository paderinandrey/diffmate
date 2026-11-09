use super::helpers::{contents, fixture, schema, version};
use crate::store::Store;
use rusqlite::Connection;

#[tokio::test]
async fn migration_failure_rolls_back_columns_and_keeps_version_one_values() {
    let (_tmp, paths) = fixture(1);
    let conn = Connection::open(paths.db()).unwrap();
    conn.execute("CREATE INDEX jobs_state ON jobs (state)", [])
        .unwrap();
    let before = (schema(&conn), contents(&conn));
    assert!(Store::open(&paths).await.is_err());
    assert_eq!(version(&conn), 1);
    assert_eq!((schema(&conn), contents(&conn)), before);
    let saved = Connection::open(paths.state_dir().join("diffmate.db.v1-backup")).unwrap();
    assert_eq!(version(&saved), 1);
    assert_eq!(contents(&saved), contents(&conn));
}

#[tokio::test]
async fn existing_backup_with_wrong_version_prevents_upgrade() {
    let (_tmp, paths) = fixture(1);
    let backup = paths.state_dir().join("diffmate.db.v1-backup");
    let saved = Connection::open(&backup).unwrap();
    saved.pragma_update(None, "user_version", 2).unwrap();
    assert!(matches!(
        Store::open(&paths).await,
        Err(crate::error::Error::Backup { .. })
    ));
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 1);
    assert_eq!(version(&saved), 2);
}
