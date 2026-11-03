use crate::error::Error;
use crate::pr_url::PrRef;
use crate::render;
use crate::runner::ReviewResult;
use crate::store::{JobId, Store};
use std::process::ExitCode;

pub(super) async fn stored(store: &Store, job: JobId, pr: &PrRef) -> Result<ExitCode, Error> {
    result(
        pr,
        &ReviewResult::from_stored(store.stored_result(job).await?, false),
    )
}

pub(super) fn result(pr: &PrRef, result: &ReviewResult) -> Result<ExitCode, Error> {
    let (stdout, stderr) = (std::io::stdout(), std::io::stderr());
    render::review(pr, result, &mut stdout.lock(), &mut stderr.lock())
        .map_err(Error::io("write the result"))?;
    if let ReviewResult::Succeeded {
        run_id,
        head_sha,
        reused: true,
        ..
    } = result
    {
        let head: String = head_sha.chars().take(12).collect();
        eprintln!("reused run {run_id} for head {head}; --force starts a new run");
    }
    Ok(if matches!(result, ReviewResult::Succeeded { .. }) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
