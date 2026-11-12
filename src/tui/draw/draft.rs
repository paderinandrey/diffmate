use crate::render::sanitize;
use crate::store::status::DraftView;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub(super) fn screen(frame: &mut Frame<'_>, draft: &DraftView, focus: usize, notice: Option<&str>) {
    let mut area = frame.area();
    if let Some(notice) = notice.filter(|text| !text.is_empty())
        && area.height > 1
    {
        let bar = Rect {
            y: area.y + area.height - 1,
            height: 1,
            ..area
        };
        area.height -= 1;
        frame.render_widget(Paragraph::new(sanitize(notice)), bar);
    }
    let width = usize::from(area.width);
    let mut rows = Vec::new();
    let mut selected = 0..0;
    for (index, finding) in draft.findings.iter().enumerate() {
        let start = rows.len();
        push(
            &mut rows,
            &header(
                &finding.anchor,
                &finding.severity,
                &finding.title,
                &finding.decision,
                finding.hint.as_deref(),
                focus == index,
            ),
            width,
        );
        if focus == index {
            push(&mut rows, &sanitize(&finding.body), width);
            selected = start..rows.len();
        }
    }
    if let Some(limitation) = &draft.limitation {
        let suffix = if draft.included { " included" } else { "" };
        push(
            &mut rows,
            &format!("Limitation: {}{suffix}", sanitize(limitation)),
            width,
        );
    }
    let findings = draft.findings.len();
    for (index, note) in draft.future.iter().enumerate() {
        let start = rows.len();
        let chosen = focus == findings + index;
        push(
            &mut rows,
            &format!("{} {}", mark(chosen), sanitize(note)),
            width,
        );
        if chosen {
            selected = start..rows.len();
        }
    }
    let height = usize::from(area.height);
    let offset = viewport(rows.len(), height, &selected);
    let visible: Vec<_> = rows.into_iter().skip(offset).take(height).collect();
    frame.render_widget(Paragraph::new(visible), area);
}

fn header(
    anchor: &str,
    severity: &str,
    title: &str,
    decision: &str,
    hint: Option<&str>,
    chosen: bool,
) -> String {
    let hint = hint.map_or(String::new(), |hint| format!("  hint: {hint}"));
    format!(
        "{} {}  {}  {}  {}{hint}",
        mark(chosen),
        sanitize(anchor),
        sanitize(severity),
        sanitize(title),
        sanitize(decision),
    )
}

fn mark(selected: bool) -> &'static str {
    if selected { ">" } else { " " }
}

fn push(rows: &mut Vec<Line<'static>>, text: &str, width: usize) {
    let width = width.max(1);
    let mut rest = text;
    if rest.is_empty() {
        rows.push(Line::from(String::new()));
        return;
    }
    while !rest.is_empty() {
        let mut end = 0;
        for (count, ch) in rest.chars().enumerate() {
            if count == width {
                break;
            }
            end += ch.len_utf8();
        }
        rows.push(Line::from(rest[..end].to_string()));
        rest = &rest[end..];
    }
}

fn viewport(total: usize, height: usize, selected: &std::ops::Range<usize>) -> usize {
    if height == 0 || total <= height || selected.is_empty() {
        return 0;
    }
    if selected.end.saturating_sub(selected.start) >= height {
        return selected.start.min(total.saturating_sub(height));
    }
    selected.end.saturating_sub(height)
}
