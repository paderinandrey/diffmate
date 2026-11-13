use super::helpers::version;
use crate::error::Error;
use crate::paths::Paths;
use crate::store::Store;
use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

const DRAFT: &str = "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES ('review', 1, 'succeeded', 1, 1, 1, 1);
INSERT INTO runs (target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 'succeeded', 'findings', 'h', 'b', 'b', 'main', 1);
INSERT INTO drafts (run_id, original_output, created_at) VALUES (1, '{}', 1);";

const FINDINGS: &str = "INSERT INTO findings (id, run_id, draft_id, ordinal, severity, title, body) VALUES ('loose', 1, 1, 1, 'low', 'Loose', 'body');
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body) VALUES ('anchored', 1, 1, 2, 'high', 'src/a.rs', 3, 'RIGHT', 'Tight', 'body');";

fn version_four(extra: &str) -> (TempDir, Paths) {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    fs::create_dir_all(paths.state_dir()).unwrap();
    let conn = Connection::open(paths.db()).unwrap();
    for sql in [
        include_str!("../../schema.sql"),
        include_str!("../../migrate_v2.sql"),
        include_str!("../../migrate_v3.sql"),
        include_str!("../../migrate_v4.sql"),
        "INSERT INTO targets (owner, repo, number, tracking, created_at) VALUES ('o', 'r', 1, 'tracked', 1)",
    ] {
        conn.execute_batch(sql).unwrap();
    }
    if !extra.is_empty() {
        conn.execute_batch(extra).unwrap();
    }
    conn.pragma_update(None, "user_version", 4).unwrap();
    (tmp, paths)
}

fn backed_up(paths: &Paths, version: i64) -> bool {
    paths
        .state_dir()
        .join(format!("diffmate.db.v{version}-backup"))
        .exists()
}

#[tokio::test]
async fn version_four_is_snapshotted_then_upgraded() {
    let (_tmp, paths) = version_four(DRAFT);
    drop(Store::open(&paths).await.unwrap());
    let conn = Connection::open(paths.db()).unwrap();
    assert_eq!(version(&conn), 5);
    let included: i64 = conn
        .query_row("SELECT limitation_included FROM drafts", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(included, 0);
    let saved = paths.state_dir().join("diffmate.db.v4-backup");
    assert_eq!(version(&Connection::open(saved).unwrap()), 4);
    assert!(!backed_up(&paths, 3));
}

#[tokio::test]
async fn fresh_database_writes_no_backup() {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    drop(Store::open(&paths).await.unwrap());
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 5);
    assert!((1..=5).all(|version| !backed_up(&paths, version)));
}

#[tokio::test]
async fn read_only_version_four_is_outdated_without_a_backup() {
    let (_tmp, paths) = version_four("");
    assert!(matches!(
        Store::open_read_only(&paths).await,
        Err(Error::SchemaOutdated(4))
    ));
    assert_eq!(version(&Connection::open(paths.db()).unwrap()), 4);
    assert!((1..=5).all(|version| !backed_up(&paths, version)));
}

#[tokio::test]
async fn null_line_findings_become_thread_status_none() {
    let (_tmp, paths) = version_four(&format!("{DRAFT}{FINDINGS}"));
    drop(Store::open(&paths).await.unwrap());
    let conn = Connection::open(paths.db()).unwrap();
    let status = |id: &str| -> String {
        conn.query_row(
            "SELECT thread_status FROM findings WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap()
    };
    assert_eq!(status("loose"), "none");
    assert_eq!(status("anchored"), "unknown");
}
