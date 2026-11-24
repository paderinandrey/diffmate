use crate::render::sanitize;
use crate::store::status::{DraftFinding, DraftView};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub(super) fn screen(
    frame: &mut Frame<'_>,
    mut area: Rect,
    draft: &DraftView,
    focus: usize,
    scroll: usize,
) -> super::Window {
    if area.height > 0 {
        let bar = Rect { height: 1, ..area };
        frame.render_widget(Paragraph::new(sanitize(&draft.freshness)).bold(), bar);
        area.y = area.y.saturating_add(1);
        area.height = area.height.saturating_sub(1);
    }
    let width = usize::from(area.width);
    let mut rows = Vec::new();
    let mut selected = 0..0;
    for (index, finding) in draft.findings.iter().enumerate() {
        let start = rows.len();
        push(&mut rows, &header(finding, focus == index), width);
        if focus == index {
            for line in &mut rows[start..] {
                *line = std::mem::take(line).reversed();
            }
            push(&mut rows, &sanitize(&finding.body), width);
            selected = start..rows.len();
        }
    }
    if let Some(limitation) = &draft.limitation {
        let start = rows.len();
        let chosen = matches!(past_findings(draft, focus), Some(super::Place::Limitation));
        push(
            &mut rows,
            &limitation_line(limitation, draft.included, chosen),
            width,
        );
        if chosen {
            selected = start..rows.len();
        }
    }
    for (index, note) in draft.future.iter().enumerate() {
        let start = rows.len();
        let chosen =
            matches!(past_findings(draft, focus), Some(super::Place::Note(at)) if at == index);
        push(&mut rows, &note_line(note, chosen), width);
        if chosen {
            selected = start..rows.len();
        }
    }
    let height = usize::from(area.height);
    let offset = viewport(rows.len(), height, &selected, scroll);
    let visible: Vec<_> = rows.into_iter().skip(offset).take(height).collect();
    frame.render_widget(Paragraph::new(visible), area);
    super::Window { width, height }
}

pub(super) fn item_rows(draft: &DraftView, focus: usize, width: usize) -> usize {
    let count = |text: &str| super::wrap::lines(text, width).count();
    if let Some(finding) = draft.findings.get(focus) {
        return count(&header(finding, true)) + count(&sanitize(&finding.body));
    }
    match past_findings(draft, focus) {
        Some(super::Place::Limitation) => draft.limitation.as_ref().map_or(0, |text| {
            count(&limitation_line(text, draft.included, true))
        }),
        Some(super::Place::Note(index)) => draft
            .future
            .get(index)
            .map_or(0, |note| count(&note_line(note, true))),
        None => 0,
    }
}

pub(super) fn past_findings(draft: &DraftView, focus: usize) -> Option<super::Place> {
    let mut index = focus.checked_sub(draft.findings.len())?;
    if draft.limitation.is_some() {
        if index == 0 {
            return Some(super::Place::Limitation);
        }
        index -= 1;
    }
    Some(super::Place::Note(index))
}

fn limitation_line(text: &str, included: bool, chosen: bool) -> String {
    let suffix = if included { " included" } else { "" };
    format!("{} Limitation: {}{suffix}", mark(chosen), sanitize(text))
}

fn note_line(note: &str, chosen: bool) -> String {
    format!("{} Future improvement: {}", mark(chosen), sanitize(note))
}

fn header(finding: &DraftFinding, chosen: bool) -> String {
    let hint = finding
        .hint
        .as_ref()
        .map_or(String::new(), |hint| format!("  hint: {}", sanitize(hint)));
    format!(
        "{} [{}] {}  ·  {}  ·  {}{hint}",
        mark(chosen),
        sanitize(&finding.severity),
        sanitize(&finding.title),
        sanitize(&finding.anchor()),
        sanitize(&finding.decision),
    )
}

fn mark(selected: bool) -> &'static str {
    if selected { ">" } else { " " }
}

fn push(rows: &mut Vec<Line<'static>>, text: &str, width: usize) {
    rows.extend(super::wrap::lines(text, width).map(|s| Line::from(s.to_owned())));
}

fn viewport(
    total: usize,
    height: usize,
    selected: &std::ops::Range<usize>,
    scroll: usize,
) -> usize {
    if height == 0 || total <= height || selected.is_empty() {
        return 0;
    }
    let span = selected.end.saturating_sub(selected.start);
    if span >= height {
        let max = span.saturating_sub(height);
        let start = selected.start.saturating_add(scroll.min(max));
        return start.min(total.saturating_sub(height));
    }
    selected.end.saturating_sub(height)
}
