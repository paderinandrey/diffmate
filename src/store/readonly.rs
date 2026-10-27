use super::Store;
use crate::error::Error;
use crate::paths::Paths;
use rusqlite::{Connection, OpenFlags};
use std::sync::{Arc, Mutex};
use std::time::Duration;

impl Store {
    pub async fn open_read_only(paths: &Paths) -> Result<Option<Store>, Error> {
        let db = paths.db();
        tokio::task::spawn_blocking(move || -> Result<Option<Store>, Error> {
            if !db.try_exists().map_err(Error::io("check database file"))? {
                return Ok(None);
            }
            let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
            let conn = Connection::open_with_flags(&db, flags)?;
            conn.busy_timeout(Duration::from_millis(5000))?;
            let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
            match version {
                0 => Ok(None),
                1 => Ok(Some(Store {
                    conn: Arc::new(Mutex::new(conn)),
                })),
                newer => Err(Error::SchemaVersion(newer)),
            }
        })
        .await?
    }
}
