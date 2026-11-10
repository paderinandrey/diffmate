use super::Store;
use crate::error::Error;
use rusqlite::params;

fn rejected_decision() -> Error {
    Error::Db(constraint("decision"))
}

fn constraint(detail: &str) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
        Some(detail.into()),
    )
}

fn json_row(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
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

    pub async fn save_finding_text(
        &self,
        finding_id: &str,
        title: &str,
        body: &str,
    ) -> Result<String, Error> {
        let finding_id = finding_id.to_string();
        let title = title.to_string();
        let body = body.to_string();
        self.call(move |conn| {
            let sql = "SELECT title, body, decision FROM findings WHERE id = ?1";
            let row = conn.query_row(sql, [&finding_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get::<_, String>(2)?))
            })?;
            let (old_title, old_body, mut decision): (String, String, String) = row;
            if title != old_title || body != old_body {
                decision = "edited".to_string();
            }
            let sql = "UPDATE findings SET title = ?1, body = ?2, decision = ?3 WHERE id = ?4";
            conn.execute(sql, params![title, body, decision, finding_id])?;
            Ok(decision)
        })
        .await
    }

    pub async fn save_future_improvement(
        &self,
        draft_id: i64,
        index: usize,
        text: &str,
    ) -> Result<(), Error> {
        let text = text.to_string();
        self.call(move |conn| {
            let sql = "SELECT future_improvements FROM drafts WHERE id = ?1";
            let raw: String = conn.query_row(sql, [draft_id], |row| row.get(0))?;
            let mut notes: Vec<String> = serde_json::from_str(&raw).map_err(json_row)?;
            let Some(note) = notes.get_mut(index) else {
                return Err(constraint("future"));
            };
            *note = text;
            let encoded = serde_json::to_string(&notes).map_err(json_row)?;
            let sql = "UPDATE drafts SET future_improvements = ?1 WHERE id = ?2";
            conn.execute(sql, params![encoded, draft_id])?;
            Ok(())
        })
        .await
    }

    pub async fn toggle_limitation(&self, draft_id: i64) -> Result<Option<bool>, Error> {
        self.call(move |conn| {
            let sql = "UPDATE drafts SET limitation_included = 1 - limitation_included
                       WHERE id = ?1 AND limitation IS NOT NULL";
            if conn.execute(sql, [draft_id])? == 0 {
                return Ok(None);
            }
            let sql = "SELECT limitation_included FROM drafts WHERE id = ?1";
            let flag: i64 = conn.query_row(sql, [draft_id], |row| row.get(0))?;
            Ok(Some(flag == 1))
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
