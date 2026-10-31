use crate::error::Error;
use rusqlite::{Connection, OpenFlags};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub(super) fn create(db: &Path) -> Result<(), Error> {
    let backup = db.with_file_name("diffmate.db.v1-backup");
    let exists = backup
        .try_exists()
        .map_err(Error::io("check database backup"))?;
    if exists {
        validate(&backup)?;
        sync(&backup)?;
    }
    let temporary = db.with_file_name("diffmate.db.v1-backup.tmp");
    match fs::remove_file(&temporary) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(Error::io("remove incomplete database backup")(error)),
    }
    if exists {
        return Ok(());
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(Error::io("create private database backup"))?;
    let destination = temporary.to_str().ok_or_else(|| Error::Backup {
        path: temporary.clone(),
        detail: "backup path is not UTF-8".into(),
    })?;
    let source = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    source.execute("VACUUM INTO ?1", [destination])?;
    validate(&temporary)?;
    file.sync_all().map_err(Error::io("sync database backup"))?;
    fs::hard_link(&temporary, &backup).map_err(Error::io("publish database backup"))?;
    sync(&backup)?;
    fs::remove_file(&temporary).map_err(Error::io("remove completed backup temporary file"))?;
    Ok(())
}

fn sync(path: &Path) -> Result<(), Error> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(Error::io("sync database backup"))?;
    let parent = path.parent().ok_or_else(|| Error::Backup {
        path: path.to_path_buf(),
        detail: "backup path has no parent".into(),
    })?;
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(Error::io("sync database backup directory"))?;
    Ok(())
}

fn validate(path: &Path) -> Result<(), Error> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let check: String = conn.pragma_query_value(None, "quick_check", |row| row.get(0))?;
    if version != 1 || check != "ok" {
        return Err(Error::Backup {
            path: path.to_path_buf(),
            detail: format!(
                "expected a valid version 1 database; version {version}, check {check}"
            ),
        });
    }
    Ok(())
}
