use super::harness::{Remote, cell, seeded};

#[tokio::test(start_paused = true)]
async fn sync_links_marker_without_post() {
    let (_tmp, store, job) = seeded().await;
    let gh = Remote::linked();
    crate::publish::execute_sync(&store, &gh, job)
        .await
        .unwrap();
    assert_eq!(gh.posts(), 0);
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT github_comment_id FROM findings WHERE id = 'f1'"
        )
        .await,
        "c1"
    );
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT github_thread_id FROM findings WHERE id = 'f1'"
        )
        .await,
        "t1"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT thread_status FROM findings WHERE id = 'f1'").await,
        "resolved"
    );
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT COALESCE(github_comment_id, '') FROM findings WHERE id = 'f2'"
        )
        .await,
        ""
    );
    assert_eq!(
        cell::<String>(&store, "SELECT thread_status FROM findings WHERE id = 'f2'").await,
        "none"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT decision FROM findings WHERE id = 'f1'").await,
        "kept"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    let state = format!("SELECT state FROM jobs WHERE id = {job}");
    assert_eq!(cell::<String>(&store, &state).await, "succeeded");
}

#[tokio::test(start_paused = true)]
async fn sync_status_read_failure_keeps_unknown() {
    let (_tmp, store, job) = seeded().await;
    let gh = Remote::status_fails();
    crate::publish::execute_sync(&store, &gh, job)
        .await
        .unwrap();
    assert_eq!(gh.posts(), 0);
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT github_comment_id FROM findings WHERE id = 'f1'"
        )
        .await,
        "c1"
    );
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT github_thread_id FROM findings WHERE id = 'f1'"
        )
        .await,
        "t1"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT thread_status FROM findings WHERE id = 'f1'").await,
        "unknown"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    let state = format!("SELECT state FROM jobs WHERE id = {job}");
    assert_eq!(cell::<String>(&store, &state).await, "queued");
    let gap =
        format!("SELECT COALESCE(next_attempt_at, 0) - updated_at FROM jobs WHERE id = {job}");
    assert_eq!(cell::<i64>(&store, &gap).await, 60_000);
}

#[tokio::test(start_paused = true)]
async fn sync_links_the_confirmed_finding_after_the_draft_changes() {
    let (_tmp, store, job) = seeded().await;
    let later = "UPDATE findings SET decision = 'dropped' WHERE id = 'f1';
        INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('f3', 1, 1, 3, 'low', 'src/b.rs', 9, 'RIGHT', 'Later', 'later', 'kept');";
    super::super::support::batch(&store, later).await;
    let gh = Remote::linked();
    crate::publish::execute_sync(&store, &gh, job)
        .await
        .unwrap();
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT github_comment_id FROM findings WHERE id = 'f1'"
        )
        .await,
        "c1"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT decision FROM findings WHERE id = 'f1'").await,
        "dropped"
    );
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT COALESCE(github_comment_id, '') FROM findings WHERE id = 'f3'"
        )
        .await,
        ""
    );
    let state = format!("SELECT state FROM jobs WHERE id = {job}");
    assert_eq!(cell::<String>(&store, &state).await, "succeeded");
}
