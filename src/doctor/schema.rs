use super::Check;
use crate::app::AppConfig;
use crate::error::Error;
use crate::store::Store;
use std::io;
use std::path::Path;

pub(super) async fn check(config: &AppConfig) -> Check {
    let db = config.paths.db();
    match stat_exists(&db).await {
        Ok(false) => Check::pass("schema: ok \u{2014} no database yet"),
        Ok(true) => open(config, &db).await,
        Err(error) => Check::fail(format!("schema: {}: {error}", db.display())),
    }
}

async fn open(config: &AppConfig, db: &Path) -> Check {
    match Store::open_read_only(&config.paths).await {
        Ok(Some(_store)) => Check::pass("schema: ok"),
        Ok(None) => version_zero(db).await,
        Err(error @ (Error::SchemaOutdated(_) | Error::SchemaVersion(_))) => {
            Check::fail(format!("schema: {error}"))
        }
        Err(error) => Check::fail(format!("schema: {}: {error}", db.display())),
    }
}

async fn version_zero(db: &Path) -> Check {
    match stat_exists(db).await {
        Ok(true) => Check::fail(format!("schema: {}", Error::SchemaOutdated(0))),
        Ok(false) => Check::pass("schema: ok \u{2014} no database yet"),
        Err(error) => Check::fail(format!("schema: {}: {error}", db.display())),
    }
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
