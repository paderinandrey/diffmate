mod backup;
#[cfg(test)]
mod tests;

use crate::error::Error;
use rusqlite::{Connection, TransactionBehavior};
use std::path::Path;

pub(crate) const SCHEMA_VERSION: i64 = 6;

pub(super) fn run(conn: &mut Connection, db: &Path) -> Result<(), Error> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            tx.execute_batch(include_str!("schema.sql"))?;
            tx.execute_batch(include_str!("migrate_v2.sql"))?;
        }
        1 => {
            backup::create(db, 1)?;
            tx.execute_batch(include_str!("migrate_v2.sql"))?;
        }
        2..=5 => backup::create(db, version)?,
        SCHEMA_VERSION => return Ok(()),
        newer => return Err(Error::SchemaVersion(newer)),
    }
    if version < 3 {
        tx.execute_batch(include_str!("migrate_v3.sql"))?;
    }
    if version < 4 {
        tx.execute_batch(include_str!("migrate_v4.sql"))?;
    }
    if version < 5 {
        tx.execute_batch(include_str!("migrate_v5.sql"))?;
    }
    tx.execute_batch(include_str!("migrate_v6.sql"))?;
    tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(tx.commit()?)
}
