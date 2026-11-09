use super::fixture::Fixture;

#[tokio::test]
async fn a_manual_review_does_not_attach_to_a_held_automatic_job() {
    let fixture = Fixture::new(true).await;
    let held_id = hold(&fixture).await;
    let manual = fixture.enqueue(false, false).await;
    assert!(!manual.attached);
    assert_ne!(manual.job.job_id, held_id);
    assert_eq!(fixture.rows().await.len(), 2);
    assert_eq!(held_flag(&fixture, held_id).await, 1);

    let forced_only = Fixture::new(true).await;
    let held_id = hold(&forced_only).await;
    let forced = forced_only.enqueue(true, false).await;
    assert!(!forced.attached);
    assert_ne!(forced.job.job_id, held_id);
    let rows = forced_only.rows().await;
    assert!(rows.last().unwrap().force);
    assert_eq!(held_flag(&forced_only, held_id).await, 1);
}

async fn hold(fixture: &Fixture) -> i64 {
    let held = fixture.enqueue(false, true).await;
    let id = held.job.job_id;
    fixture
        .store
        .call(move |conn| {
            conn.execute("UPDATE jobs SET held = 1 WHERE id = ?1", [id])
                .map(drop)
        })
        .await
        .unwrap();
    id
}

async fn held_flag(fixture: &Fixture, job: i64) -> i64 {
    fixture
        .store
        .call(move |conn| {
            conn.query_row("SELECT held FROM jobs WHERE id = ?1", [job], |row| {
                row.get(0)
            })
        })
        .await
        .unwrap()
}
