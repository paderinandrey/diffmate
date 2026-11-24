mod draft;
mod queue;
mod wrap;

pub(super) enum Place {
    Limitation,
    Note(usize),
}

pub(super) fn past_findings(
    draft: &crate::store::status::DraftView,
    focus: usize,
) -> Option<Place> {
    draft::past_findings(draft, focus)
}

pub(super) fn item_rows(
    draft: &crate::store::status::DraftView,
    focus: usize,
    width: usize,
) -> usize {
    draft::item_rows(draft, focus, width)
}

use super::model::Session;
use crate::render::sanitize;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;

pub(super) struct Window {
    pub width: usize,
    pub height: usize,
}

impl Window {
    pub(super) const fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

pub(super) fn screen(frame: &mut Frame<'_>, session: &Session, running: bool) -> Window {
    let mut area = frame.area();
    if running && area.height > 1 {
        area.height -= 1;
        let bar = ratatui::layout::Rect {
            y: area.bottom(),
            height: 1,
            ..area
        };
        frame.render_widget(Paragraph::new("runner: tui"), bar);
    }
    if let Some(confirm) = session.confirm_modal() {
        super::confirm::draw(frame, confirm, area);
        return Window::empty();
    }
    if area.height >= 9 {
        let (title, keys) = if let Some((draft, _)) = session.draft() {
            let label = session
                .rows
                .get(session.selected)
                .map_or("", |row| &row.label);
            let title = format!(
                "Draft  /  {}  /  Findings: {}",
                sanitize(label),
                draft.findings.len()
            );
            (
                title,
                "↑↓ move   k keep   d drop   K keep rest   e edit   i include limit   Esc back\na approve   r request changes   c comment",
            )
        } else {
            (
                format!("Review queue  /  Targets: {}", session.rows.len()),
                "↑↓ select   Enter draft   o open PR   n re-run   x discard   q quit",
            )
        };
        let heading = Rect { height: 1, ..area };
        frame.render_widget(Paragraph::new(title).bold(), heading);
        area.y += 1;
        area.height -= 1;
        bottom(frame, &mut area, keys, Style::default().italic());
    }
    if let Some(report) = session.publication_report() {
        bottom(
            frame,
            &mut area,
            &sanitize(&report.text),
            Style::default().bold(),
        );
    }
    if let Some(notice) = session.notice().filter(|s| !s.is_empty()) {
        bottom(frame, &mut area, &sanitize(notice), Style::default().bold());
    }
    if let Some((draft, focus)) = session.draft() {
        return draft::screen(frame, area, draft, focus, session.scroll());
    }
    queue::screen(frame, session, area);
    Window::empty()
}

fn bottom(frame: &mut Frame<'_>, area: &mut Rect, text: &str, style: Style) {
    if area.height <= 1 || area.width == 0 {
        return;
    }
    let lines = wrapped(text, area.width, area.height.div_ceil(2));
    let height = u16::try_from(lines.height()).unwrap_or(area.height - 1);
    area.height -= height;
    let bar = Rect {
        y: area.bottom(),
        height,
        ..*area
    };
    frame.render_widget(Paragraph::new(lines).style(style), bar);
}

pub(super) fn wrapped(text: &str, width: u16, limit: u16) -> Text<'_> {
    wrap::lines(text, usize::from(width))
        .take(usize::from(limit))
        .collect()
}
