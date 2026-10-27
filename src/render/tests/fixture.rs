use super::super::review;
use crate::agent::{Severity, Side};
use crate::pr_url::PrRef;
use crate::runner::ReviewResult;
use crate::store::records::FindingRecord;

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

pub fn finding(
    path: Option<&str>,
    line: Option<u32>,
    severity: Severity,
    title: &str,
) -> FindingRecord {
    FindingRecord {
        severity,
        path: path.map(Into::into),
        line,
        side: line.map(|_| Side::Right),
        title: title.into(),
        body: format!("Body of {title}"),
        previous_finding_id: None,
    }
}

pub fn succeeded(
    findings: Vec<FindingRecord>,
    limitation: Option<&str>,
    items: &[&str],
) -> ReviewResult {
    ReviewResult::Succeeded {
        run_id: 1,
        head_sha: HEAD.into(),
        findings,
        limitation: limitation.map(Into::into),
        future_improvements: items.iter().map(|item| (*item).into()).collect(),
        warnings: Vec::new(),
    }
}

pub fn render(result: &ReviewResult) -> (String, String) {
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    review(&pr, result, &mut out, &mut err).unwrap();
    (
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}
