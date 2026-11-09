use super::helpers::{contents, fixture, version};
use crate::store::Store;
use rusqlite::Connection;

#[tokio::test]
async fn concurrent_first_opens_share_one_backup_and_migration() {
    let (_tmp, paths) = fixture(1);
    let before = contents(&Connection::open(paths.db()).unwrap());
    let (first, second, third) = tokio::join!(
        Store::open(&paths),
        Store::open(&paths),
        Store::open(&paths)
    );
    drop(first.unwrap());
    drop(second.unwrap());
    drop(third.unwrap());
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 2);
    let backup = Connection::open(paths.state_dir().join("diffmate.db.v1-backup")).unwrap();
    assert_eq!(version(&backup), 1);
    assert_eq!(contents(&backup), before);
    assert!(!paths.state_dir().join("diffmate.db.v1-backup.tmp").exists());
}
