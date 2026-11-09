use super::fixture::{self, Fix};
use rusqlite::params;

impl Fix {
    pub(super) async fn id(&self, number: u64) -> i64 {
        self.store
            .call(move |conn| {
                conn.query_row(
                    "SELECT id FROM targets WHERE number = ?1",
                    [i64::try_from(number).unwrap()],
                    |row| row.get(0),
                )
            })
            .await
            .unwrap()
    }

    pub(super) async fn set_heads(
        &self,
        number: u64,
        queued: Option<&str>,
        head: Option<&str>,
        checked: Option<i64>,
    ) {
        let queued = queued.map(str::to_string);
        let head = head.map(str::to_string);
        self.store
            .call(move |conn| {
                conn.execute(
                    "UPDATE targets SET queued_head = ?1, head_sha = ?2, head_checked_at = ?3
                     WHERE owner = 'o' AND repo = 'r' AND number = ?4",
                    params![queued, head, checked, i64::try_from(number).unwrap()],
                )
                .map(drop)
            })
            .await
            .unwrap();
    }

    pub(super) async fn after(&self, number: u64, now: i64, debounce: i64) {
        let id = self.id(number).await;
        self.store
            .schedule_after_run(id, now, debounce, &fixture::process())
            .await
            .unwrap();
    }
}
