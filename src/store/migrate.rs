mod backup;
#[cfg(test)]
mod tests;

use crate::error::Error;
use rusqlite::{Connection, TransactionBehavior};
use std::path::Path;

pub(crate) const SCHEMA_VERSION: i64 = 3;

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
        2 => backup::create(db, 2)?,
        SCHEMA_VERSION => return Ok(()),
        newer => return Err(Error::SchemaVersion(newer)),
    }
    tx.execute_batch(include_str!("migrate_v3.sql"))?;
    tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(tx.commit()?)
}
