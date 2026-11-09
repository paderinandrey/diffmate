use super::*;
use std::process::Output;

pub(super) fn restarted(sandbox: &Sandbox, point: &str, head: &str, resumed: &Output) {
    let db = database(sandbox);
    let before_job = matches!(point, "enqueued" | "locked" | "recovery-db" | "recovered");
    let before_run = matches!(point, "job-started" | "fetched" | "revision");
    assert_eq!(
        count(&db, "SELECT count(*) FROM jobs"),
        if before_job { 1 } else { 2 }
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs"),
        if before_job || before_run || point == "stored" {
            1
        } else {
            2
        }
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE state = 'succeeded'"),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT head_sha FROM runs WHERE state = 'succeeded'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        head
    );
    if before_job {
        assert!(String::from_utf8_lossy(&resumed.stderr).contains("attached to job 1"));
        assert_eq!(
            count(
                &db,
                "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'succeeded'"
            ),
            1
        );
    } else if point == "stored" {
        assert_eq!(
            count(
                &db,
                "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND result_run_id = 1"
            ),
            2
        );
    } else {
        assert_eq!(
            count(
                &db,
                "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'failed' AND failure_reason = 'interrupted: crash'"
            ),
            1
        );
        assert_eq!(
            count(
                &db,
                "SELECT count(*) FROM runs WHERE job_id = 2 AND state = 'succeeded'"
            ),
            1
        );
        if !before_run {
            assert_eq!(
                count(
                    &db,
                    "SELECT count(*) FROM runs WHERE id = 1 AND state = 'failed' AND failure_kind = 'interrupted' AND interrupted_cause = 'crash'"
                ),
                1
            );
        }
    }
}
