use crate::support::sandbox::Sandbox;
use rusqlite::{Connection, types::Value};

const TABLES: [&str; 5] = ["targets", "jobs", "runs", "drafts", "findings"];
pub type Snapshot = Vec<(String, Vec<Vec<Value>>)>;

pub fn create(sandbox: &Sandbox) -> Snapshot {
    std::fs::create_dir_all(sandbox.state_dir()).unwrap();
    let conn = Connection::open(sandbox.db_path()).unwrap();
    conn.execute_batch(include_str!("../../src/store/schema.sql"))
        .unwrap();
    conn.execute_batch("PRAGMA user_version = 1").unwrap();
    conn.execute_batch(include_str!("fixture.sql")).unwrap();
    TABLES
        .iter()
        .map(|table| {
            let mut stmt = conn.prepare(&format!("SELECT * FROM {table}")).unwrap();
            let columns = stmt.column_names().collect::<Vec<_>>().join(", ");
            let values = read(&mut stmt);
            let sql = format!("SELECT {columns} FROM {table} WHERE id = ?1");
            (sql, values)
        })
        .collect()
}

fn read(stmt: &mut rusqlite::Statement<'_>) -> Vec<Vec<Value>> {
    let columns = stmt.column_count();
    stmt.query_map([], |row| (0..columns).map(|i| row.get(i)).collect())
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

pub fn assert_preserved(conn: &Connection, before: &Snapshot) {
    for (sql, expected) in before {
        let mut stmt = conn.prepare(sql).unwrap();
        let columns = stmt.column_count();
        let id = if sql.contains("FROM findings ") {
            "1.1"
        } else {
            "1"
        };
        let after: Vec<Vec<Value>> = stmt
            .query_map([id], |row| (0..columns).map(|i| row.get(i)).collect())
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(&after, expected, "{sql}");
    }
}
