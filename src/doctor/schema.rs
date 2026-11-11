use super::Check;
use crate::app::AppConfig;
use crate::error::Error;
use crate::store::SCHEMA_VERSION;
use rusqlite::{Connection, OpenFlags};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub(super) async fn check(config: &AppConfig) -> Check {
    let db = config.paths.db();
    match stat_exists(&db).await {
        Ok(false) => Check::pass("schema: ok \u{2014} no database yet"),
        Ok(true) => open(&db).await,
        Err(error) => Check::fail(format!("schema: {}: {error}", db.display())),
    }
}

async fn open(db: &Path) -> Check {
    match read_version(db).await {
        Ok(version) => classified(version),
        Err(error) => Check::fail(format!("schema: {}: {error}", db.display())),
    }
}

fn classified(version: i64) -> Check {
    match version {
        SCHEMA_VERSION => Check::pass("schema: ok"),
        found if (0..SCHEMA_VERSION).contains(&found) => {
            Check::fail(format!("schema: {}", Error::SchemaOutdated(found)))
        }
        newer => Check::fail(format!("schema: {}", Error::SchemaVersion(newer))),
    }
}

async fn read_version(db: &Path) -> Result<i64, Error> {
    let db = db.to_path_buf();
    tokio::task::spawn_blocking(move || query_version(&db)).await?
}

// A read-only open of a WAL database creates -shm unless the URI is immutable.
fn query_version(db: &Path) -> Result<i64, Error> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(immutable_uri(db), flags)?;
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

fn immutable_uri(path: &Path) -> String {
    let mut uri = String::from("file:");
    for &byte in path.as_os_str().as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'-' => {
                uri.push(char::from(byte));
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                uri.push('%');
                uri.push(char::from(HEX[usize::from(byte >> 4)]));
                uri.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
    }
    uri.push_str("?mode=ro&immutable=1");
    uri
}

async fn stat_exists(path: &Path) -> io::Result<bool> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || match std::fs::metadata(&path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    })
    .await
    .map_err(io::Error::other)?
}
