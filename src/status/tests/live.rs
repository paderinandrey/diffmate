use super::super::target_lines;
use crate::pr_url::PrRef;
use crate::store::now_ms;
use crate::store::status::{RetryView, TargetStatus};
use std::cell::RefCell;
use tokio::time::{Duration, Instant, sleep};

fn target(number: u8) -> TargetStatus {
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: u64::from(number),
    };
    TargetStatus {
        id: i64::from(number),
        label: pr.label(),
        pr,
        tracking: "untracked".into(),
        latest: "succeeded".into(),
        draft_head: Some(format!("head-{number}")),
        known_head: None,
        retry: RetryView {
            job_state: None,
            origin: None,
            retry_count: 0,
            max_retries: None,
            next_attempt_at: None,
            has_failed_run: false,
        },
        publication: "none".into(),
    }
}

#[tokio::test(start_paused = true)]
async fn timed_out_heads_report_the_saved_head_and_age_or_unknown() {
    let mut targets = [target(1), target(2)];
    targets[0].known_head = Some(("cached".into(), now_ms() - 5_400_000));
    let start = Instant::now();
    let rows = target_lines(&targets, async |_| {
        sleep(Duration::from_secs(30)).await;
        Ok("too late".into())
    })
    .await;

    assert_eq!(start.elapsed(), Duration::from_secs(5));
    assert_eq!(
        rows,
        [
            "o/r#1  untracked  succeeded  stale: head cached, checked 1h ago (live check failed)  none",
            "o/r#2  untracked  succeeded  freshness unknown: head never checked (live check failed)  none",
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn only_three_drafts_are_probed_at_once_and_rows_stay_in_order() {
    let mut targets = (1..=6).map(target).collect::<Vec<_>>();
    targets[3].draft_head = None;
    let start = Instant::now();
    let starts = RefCell::new(Vec::new());
    let rows = target_lines(&targets, async |pr| {
        starts
            .borrow_mut()
            .push((pr.number, start.elapsed().as_secs()));
        let seconds = match pr.number {
            1 => 4,
            3 => 3,
            _ => 1,
        };
        sleep(Duration::from_secs(seconds)).await;
        Ok(format!("head-{}", pr.number))
    })
    .await;

    let mut starts = starts.into_inner();
    starts.sort_unstable();
    assert_eq!(starts, [(1, 0), (2, 0), (3, 0), (5, 1), (6, 2)]);
    for (number, row) in (1..=6).zip(&rows) {
        let freshness = if number == 4 {
            String::new()
        } else {
            format!("  current: head head-{number}, checked just now")
        };
        assert_eq!(
            row,
            &format!("o/r#{number}  untracked  succeeded{freshness}  none")
        );
    }
    assert_eq!(rows.len(), 6);
}
