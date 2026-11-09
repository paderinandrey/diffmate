#[path = "migration/fixture.rs"]
mod fixture;
#[allow(dead_code)]
#[path = "concurrency/helpers.rs"]
mod helpers;
mod support;
#[allow(dead_code)]
mod tracer;

use fixture::{assert_preserved, create};
use helpers::{url, with_two_prs};
use rusqlite::types::Value;
use support::sandbox::Sandbox;
use tracer::{count, database};

#[test]
fn phase_one_database_is_upgraded_and_kept() {
    let sandbox = Sandbox::new();
    with_two_prs(&sandbox);
    let before = create(&sandbox);

    let output = sandbox.run(&["review", &url(2)]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let conn = database(&sandbox);
    assert_eq!(count(&conn, "PRAGMA user_version"), 2);
    assert_preserved(&conn, &before);
    let defaults: Vec<Value> = conn
        .query_row(
            "SELECT origin, force, stop_reason, retry_count, max_retries,
         next_attempt_at, result_run_id FROM jobs WHERE id = 1",
            [],
            |row| (0..7).map(|i| row.get(i)).collect(),
        )
        .unwrap();
    assert_eq!(
        defaults,
        vec![
            Value::Text("manual".into()),
            Value::Integer(0),
            Value::Null,
            Value::Integer(0),
            Value::Null,
            Value::Null,
            Value::Null
        ]
    );
    assert_eq!(
        count(
            &conn,
            "SELECT count(*) FROM targets WHERE id = 1 AND head_sha IS NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &conn,
            "SELECT count(*) FROM targets WHERE id = 2 AND head_sha IS NOT NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &conn,
            "SELECT count(*) FROM runs WHERE target_id = 2 AND state = 'succeeded'"
        ),
        1
    );
    let status = sandbox.run(&["status"]);
    assert!(status.status.success());
    assert!(
        String::from_utf8_lossy(&status.stdout)
            .starts_with("o/r#1  untracked  succeeded (findings: 1)")
    );
    let backup =
        rusqlite::Connection::open(sandbox.state_dir().join("diffmate.db.v1-backup")).unwrap();
    assert_eq!(count(&backup, "PRAGMA user_version"), 1);
    assert_preserved(&backup, &before);
}

#[test]
fn status_on_phase_one_database_asks_for_an_upgrade() {
    let sandbox = Sandbox::new();
    let before = create(&sandbox);
    let bytes = std::fs::read(sandbox.db_path()).unwrap();
    let output = sandbox.run(&["status"]);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("schema version 1") && error.contains("diffmate review"));
    let conn = database(&sandbox);
    assert_eq!(count(&conn, "PRAGMA user_version"), 1);
    assert_preserved(&conn, &before);
    assert_eq!(std::fs::read(sandbox.db_path()).unwrap(), bytes);
    assert_eq!(std::fs::read_dir(sandbox.state_dir()).unwrap().count(), 1);
}
