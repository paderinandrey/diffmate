use super::Notice;
use crate::render::sanitize;

const READY_TITLE: &str = "diffmate: draft ready";
const FAILED_TITLE: &str = "diffmate: review failed";

pub(super) fn display(notice: &Notice) -> String {
    let (title, body) = match notice {
        Notice::Ready { label } => (READY_TITLE, label.clone()),
        Notice::GaveUp { label, reason } => (FAILED_TITLE, format!("{label}: {reason}")),
    };
    format!(
        "display notification \"{}\" with title \"{}\"",
        escape(&body),
        escape(title)
    )
}

fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in sanitize(text).chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' | '\r' | '\u{2028}' | '\u{2029}' => out.push(' '),
            other => out.push(other),
        }
    }
    out
}
