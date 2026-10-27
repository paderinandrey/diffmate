use super::fake_gh::{FakeGh, pr};
use crate::context::ci::text;
use crate::gh::{CheckRun, CheckRuns};

pub(super) fn run(id: u64, name: &str, status: &str, conclusion: Option<&str>) -> CheckRun {
    CheckRun {
        id,
        name: name.into(),
        status: status.into(),
        conclusion: conclusion.map(Into::into),
        output: None,
    }
}

pub(super) fn fake(runs: Vec<CheckRun>, total: usize) -> FakeGh {
    let total_count = total as u64;
    FakeGh {
        runs: Some(CheckRuns { total_count, runs }).into(),
        ..FakeGh::default()
    }
}

#[tokio::test]
async fn every_check_is_listed_and_only_failed_ones_are_annotated() {
    let runs = vec![
        run(1, "build", "completed", Some("success")),
        run(2, "lint", "in_progress", None),
        run(3, "test", "completed", Some("failure")),
        run(4, "deploy", "completed", Some("timed_out")),
    ];
    let gh = fake(runs, 4);

    let ci = text(&gh, &pr(), "abc123").await;

    assert!(ci.starts_with("# Check runs for abc123\n\n- build: completed, success\n- lint: in_progress, not completed\n"), "{ci}");
    assert!(
        ci.contains("### test (completed, failure)\nsrc/a.rs:7 [failure] boom"),
        "{ci}"
    );
    assert!(ci.contains("### deploy (completed, timed_out)"), "{ci}");
    assert_eq!(*gh.annotated.lock().unwrap(), [3, 4]);
    assert!(!ci.to_lowercase().contains("passing") && !ci.contains("TRUNCATED"));
}

#[tokio::test]
async fn at_most_ten_failed_checks_are_annotated_and_the_cut_is_stated() {
    let runs = (1..=12)
        .map(|id| run(id, "job", "completed", Some("failure")))
        .collect();
    let gh = fake(runs, 12);

    let ci = text(&gh, &pr(), "abc").await;

    assert_eq!(gh.annotated.lock().unwrap().len(), 10);
    assert!(
        ci.starts_with("TRUNCATED: ci.md: annotations are read for only 10 of 12 failed checks.\n"),
        "{ci}"
    );
}

#[tokio::test]
async fn a_short_list_is_marked_truncated() {
    let ci = text(
        &fake(vec![run(1, "a", "completed", Some("success"))], 5),
        &pr(),
        "abc",
    )
    .await;

    assert!(
        ci.starts_with("TRUNCATED: ci.md: only 1 of 5 check runs are listed.\n"),
        "{ci}"
    );
}
