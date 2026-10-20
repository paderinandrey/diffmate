pub(crate) mod finish;
pub(crate) mod jobs;
pub(crate) mod readonly;
pub(crate) mod records;
pub(crate) mod runs;
pub(crate) mod status;

use crate::error::Error;
use crate::paths::{Paths, create_private_dir};
use rusqlite::{Connection, TransactionBehavior};
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) type TargetId = i64;
pub(crate) type JobId = i64;
pub(crate) type RunId = i64;

pub(crate) fn now_ms() -> i64 {
    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH);
    elapsed.map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

#[derive(Clone)]
pub(crate) struct Store {
    conn: Arc<Mutex<Connection>>,
}

fn migrate(conn: &mut Connection) -> Result<(), Error> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            tx.execute_batch(include_str!("store/schema.sql"))?;
            tx.pragma_update(None, "user_version", 1)?;
        }
        1 => {}
        newer => return Err(Error::SchemaVersion(newer)),
    }
    Ok(tx.commit()?)
}

impl Store {
    pub async fn open(paths: &Paths) -> Result<Store, Error> {
        let (dir, db) = (paths.state_dir().to_path_buf(), paths.db());
        tokio::task::spawn_blocking(move || -> Result<Store, Error> {
            create_private_dir(&dir).map_err(Error::io("create state directory"))?;
            let mut options = OpenOptions::new();
            options.append(true).create(true).mode(0o600);
            options
                .open(&db)
                .map_err(Error::io("create database file"))?;
            let mut conn = Connection::open(&db)?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "foreign_keys", "ON")?;
            conn.busy_timeout(Duration::from_millis(5000))?;
            migrate(&mut conn)?;
            Ok(Store {
                conn: Arc::new(Mutex::new(conn)),
            })
        })
        .await?
    }

    pub async fn call<R, F>(&self, f: F) -> Result<R, Error>
    where
        R: Send + 'static,
        F: FnOnce(&mut Connection) -> rusqlite::Result<R> + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let joined = tokio::task::spawn_blocking(move || {
            let mut guard = conn.lock().unwrap_or_else(PoisonError::into_inner);
            f(&mut guard)
        })
        .await?;
        Ok(joined?)
    }
}
