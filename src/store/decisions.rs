use super::Store;
use crate::error::Error;
use rusqlite::params;

fn rejected_decision() -> Error {
    Error::Db(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
        Some("decision".into()),
    ))
}

impl Store {
    pub async fn set_decision(&self, finding_id: &str, decision: &str) -> Result<(), Error> {
        if !matches!(decision, "kept" | "dropped" | "undecided") {
            return Err(rejected_decision());
        }
        let finding_id = finding_id.to_string();
        let decision = decision.to_string();
        self.call(move |conn| {
            conn.execute(
                "UPDATE findings SET decision = ?1 WHERE id = ?2",
                params![decision, finding_id],
            )
            .map(drop)
        })
        .await
    }

    pub async fn keep_undecided(&self, draft_id: i64) -> Result<u32, Error> {
        self.call(move |conn| {
            let sql = "UPDATE findings SET decision = 'kept'
                       WHERE draft_id = ?1 AND decision = 'undecided'";
            conn.execute(sql, [draft_id])?;
            Ok(u32::try_from(conn.changes()).unwrap_or(u32::MAX))
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::paths::Paths;

    #[tokio::test]
    async fn other_names_are_database_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
        let store = super::Store::open(&paths).await.unwrap();
        let error = store.set_decision("missing", "keptish").await.unwrap_err();
        assert!(error.to_string().starts_with("database:"), "{error}");
    }
}
