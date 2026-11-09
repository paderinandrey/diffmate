mod script;
#[cfg(test)]
mod tests;

use crate::error::Error;
use crate::proc::run_tool;
use crate::procenv::InheritedEnv;
use crate::runner::{JobEnd, ReviewResult};
use crate::store::records::JobOrigin;
use crate::store::{JobId, Store};
use tokio::process::Command;

pub(crate) enum Notice {
    Ready { label: String },
    GaveUp { label: String, reason: String },
}

pub(crate) fn decide(origin: JobOrigin, result: &ReviewResult, label: &str) -> Option<Notice> {
    if origin != JobOrigin::Automatic {
        return None;
    }
    match result {
        ReviewResult::Succeeded { .. } => Some(Notice::Ready {
            label: label.to_owned(),
        }),
        ReviewResult::RunFailed {
            message,
            job_end: JobEnd::Failed,
            ..
        } => Some(gave_up(label, message)),
        ReviewResult::JobFailed { reason } => Some(gave_up(label, reason)),
        ReviewResult::Cancelled {
            job_end: JobEnd::Failed,
            ..
        } => Some(gave_up(label, "")),
        _ => None,
    }
}

fn gave_up(label: &str, reason: &str) -> Notice {
    Notice::GaveUp {
        label: label.to_owned(),
        reason: reason.to_owned(),
    }
}

pub(crate) async fn for_result(
    store: &Store,
    job: JobId,
    origin: JobOrigin,
    result: &ReviewResult,
    label: &str,
) -> Result<Option<Notice>, Error> {
    let Some(mut notice) = decide(origin, result, label) else {
        return Ok(None);
    };
    if let Notice::GaveUp { reason, .. } = &mut notice
        && matches!(
            result,
            ReviewResult::Cancelled {
                job_end: JobEnd::Failed,
                ..
            }
        )
    {
        let stored = store.job_failure_reason(job).await?;
        *reason = stored.unwrap_or_default();
    }
    Ok(Some(notice))
}

pub(crate) async fn deliver(notice: &Notice) -> Result<(), Error> {
    let script = script::display(notice);
    let mut command = Command::new("osascript");
    command.env_clear().envs(InheritedEnv::capture().tool_env());
    command.args(["-e", script.as_str()]);
    run_tool("osascript", command).await.map(drop)
}
