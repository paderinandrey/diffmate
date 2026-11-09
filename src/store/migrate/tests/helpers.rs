use crate::paths::Paths;
use rusqlite::{Connection, types::Value};
use tempfile::TempDir;

pub fn fixture(version: i64) -> (TempDir, Paths) {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    std::fs::create_dir_all(paths.state_dir()).unwrap();
    let conn = Connection::open(paths.db()).unwrap();
    conn.execute_batch(include_str!("../../schema.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../../../../tests/migration/fixture.sql"))
        .unwrap();
    conn.pragma_update(None, "user_version", version).unwrap();
    (tmp, paths)
}

pub fn rows(conn: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut stmt = conn.prepare(sql).unwrap();
    let columns = stmt.column_count();
    stmt.query_map([], |row| (0..columns).map(|i| row.get(i)).collect())
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

pub fn schema(conn: &Connection) -> Vec<Vec<Value>> {
    rows(
        conn,
        "SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name",
    )
}

pub fn contents(conn: &Connection) -> Vec<Vec<Vec<Value>>> {
    ["targets", "jobs", "runs", "drafts", "findings"]
        .map(|table| rows(conn, &format!("SELECT * FROM {table} ORDER BY id")))
        .to_vec()
}

pub fn version(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap()
}
