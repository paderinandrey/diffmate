use super::caps::{CI_CAP, cap_text, reason};
use crate::gh::{Annotation, CheckRun, Gh};
use crate::pr_url::PrRef;

const FAILED: [&str; 4] = ["failure", "timed_out", "startup_failure", "action_required"];
const ANNOTATED: usize = 10;

fn failed(run: &CheckRun) -> bool {
    run.conclusion
        .as_deref()
        .is_some_and(|value| FAILED.contains(&value))
}

fn status(run: &CheckRun) -> String {
    match (run.status.as_str(), run.conclusion.as_deref()) {
        ("completed", Some(conclusion)) => format!("completed, {conclusion}"),
        (other, _) => format!("{other}, not completed"),
    }
}

fn annotation(a: &Annotation) -> String {
    let level = a.level.as_deref().unwrap_or("notice");
    let message = a.message.as_deref().unwrap_or_default().trim_end();
    format!("{}:{} [{level}] {message}", a.path, a.start_line)
}

async fn detail<H: Gh>(gh: &H, pr: &PrRef, run: &CheckRun) -> String {
    let mut lines = vec![format!("### {} ({})", run.name, status(run))];
    if let Some(output) = &run.output {
        for (label, text) in [("Title", &output.title), ("Summary", &output.summary)] {
            if let Some(text) = text.as_deref().filter(|text| !text.is_empty()) {
                lines.push(format!("{label}: {}", text.trim_end()));
            }
        }
    }
    match gh.annotations(pr, run.id).await {
        Ok(found) => lines.extend(found.iter().map(annotation)),
        Err(error) => lines.push(format!("Annotations unavailable: {}", reason(&error))),
    }
    lines.join("\n")
}

pub(super) async fn text<H: Gh>(gh: &H, pr: &PrRef, sha: &str) -> String {
    let found = match gh.check_runs(pr, sha).await {
        Ok(found) if !found.runs.is_empty() => found,
        Ok(_) => return format!("No check runs were reported for {sha}. CI status unknown.\n"),
        Err(error) => return format!("CI status unknown: {}\n", reason(&error)),
    };
    let broken: Vec<_> = found.runs.iter().filter(|run| failed(run)).collect();
    let (listed, total) = (found.runs.len(), found.total_count);
    let mut notes = Vec::new();
    if total > u64::try_from(listed).unwrap_or(u64::MAX) {
        notes.push(format!("only {listed} of {total} check runs are listed"));
    }
    if broken.len() > ANNOTATED {
        let count = broken.len();
        notes.push(format!(
            "annotations are read for only {ANNOTATED} of {count} failed checks"
        ));
    }
    let items: Vec<_> = found
        .runs
        .iter()
        .map(|run| format!("- {}: {}", run.name, status(run)))
        .collect();
    let mut parts = vec![format!("# Check runs for {sha}"), items.join("\n")];
    for run in broken.into_iter().take(ANNOTATED) {
        parts.push(detail(gh, pr, run).await);
    }
    let body = parts.join("\n\n") + "\n";
    let body = if notes.is_empty() {
        body
    } else {
        format!("TRUNCATED: ci.md: {}.\n{body}", notes.join("; "))
    };
    cap_text("ci.md", &body, CI_CAP)
}
