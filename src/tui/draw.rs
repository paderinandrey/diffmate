mod draft;

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
use crate::store::status::QueueRow;
use ratatui::Frame;
use ratatui::text::Line;
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
    if running && area.height > 0 {
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
    if let Some(report) = session.publication_report()
        && area.height > 1
    {
        area.height -= 1;
        let bar = ratatui::layout::Rect {
            y: area.bottom(),
            height: 1,
            ..area
        };
        frame.render_widget(Paragraph::new(sanitize(&report.text)), bar);
    }
    if let Some((draft, focus)) = session.draft() {
        return draft::screen(
            frame,
            area,
            draft,
            focus,
            session.notice(),
            session.scroll(),
        );
    }
    let height = usize::from(area.height);
    let extra = usize::from(session.notice().is_some());
    let mut lines = queue_lines(session, height.saturating_sub(extra));
    if let Some(notice) = session.notice() {
        lines.push(Line::from(sanitize(notice)));
    }
    frame.render_widget(Paragraph::new(lines), area);
    Window::empty()
}

fn queue_lines(session: &Session, room: usize) -> Vec<Line<'static>> {
    if session.rows.is_empty() {
        return vec![Line::from("No targets.")];
    }
    let start = queue_start(session.rows.len(), room, session.selected);
    session
        .rows
        .iter()
        .enumerate()
        .skip(start)
        .take(room)
        .map(|(index, row)| {
            let mark = if index == session.selected { ">" } else { " " };
            Line::from(format!("{mark} {}", row_text(row)))
        })
        .collect()
}

fn queue_start(total: usize, room: usize, selected: usize) -> usize {
    if room == 0 || total <= room || selected < room {
        0
    } else {
        selected + 1 - room
    }
}

fn row_text(row: &QueueRow) -> String {
    let mut parts = vec![
        sanitize(&row.label),
        sanitize(&row.tracking),
        sanitize(&row.latest),
    ];
    if !row.freshness.is_empty() {
        parts.push(sanitize(&row.freshness));
    }
    parts.push(sanitize(&row.publication));
    parts.join("  ")
}
