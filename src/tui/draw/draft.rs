use crate::render::sanitize;
use crate::store::status::DraftView;
use ratatui::Frame;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub(super) fn screen(frame: &mut Frame<'_>, draft: &DraftView, focus: usize, notice: Option<&str>) {
    let mut lines = Vec::new();
    for (index, finding) in draft.findings.iter().enumerate() {
        let hint = finding
            .hint
            .as_deref()
            .map_or(String::new(), |hint| format!("  hint: {hint}"));
        lines.push(Line::from(format!(
            "{} {}  {}  {}  {}{hint}",
            mark(focus == index),
            sanitize(&finding.anchor),
            sanitize(&finding.severity),
            sanitize(&finding.title),
            sanitize(&finding.decision),
        )));
    }
    match draft.findings.get(focus) {
        Some(finding) => {
            lines.push(Line::from(sanitize(&finding.title)));
            lines.push(Line::from(sanitize(&finding.body)));
        }
        None => {
            if let Some(note) = draft.future.get(focus.saturating_sub(draft.findings.len())) {
                lines.push(Line::from(sanitize(note)));
            }
        }
    }
    if let Some(limitation) = &draft.limitation {
        let suffix = if draft.included { " included" } else { "" };
        let text = format!("Limitation: {}{suffix}", sanitize(limitation));
        lines.push(Line::from(text));
    }
    let findings = draft.findings.len();
    for (index, note) in draft.future.iter().enumerate() {
        lines.push(Line::from(format!(
            "{} {}",
            mark(focus == findings + index),
            sanitize(note)
        )));
    }
    if let Some(notice) = notice {
        lines.push(Line::from(sanitize(notice)));
    }
    frame.render_widget(Paragraph::new(lines), frame.area());
}

fn mark(selected: bool) -> &'static str {
    if selected { ">" } else { " " }
}
