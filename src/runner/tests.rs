mod before;
use before::cancelled_before_run;

use super::{Deps, ReviewResult, execute_review_job};
use crate::agent::claude::ClaudeCli;
use crate::app::ProcessInfo;
use crate::cancel::StopReason;
use crate::gh::GhCli;
use crate::git::GitCli;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::procenv::InheritedEnv;
use crate::store::Store;
use tempfile::TempDir;
use tokio::sync::watch;

#[tokio::test]
async fn a_cancel_before_the_run_exists_fails_the_job_and_creates_no_run() {
    cancelled_before_run(None, false).await;
}

#[tokio::test]
async fn shutdown_before_the_run_exists_keeps_its_cause_and_creates_no_run() {
    cancelled_before_run(Some(StopReason::Shutdown), false).await;
}

#[tokio::test]
async fn user_cancel_before_the_run_exists_keeps_its_cause_and_creates_no_run() {
    cancelled_before_run(Some(StopReason::UserCancel), false).await;
}

#[tokio::test]
async fn an_automatic_shutdown_before_the_run_requeues_without_creating_a_run() {
    cancelled_before_run(Some(StopReason::Shutdown), true).await;
}
