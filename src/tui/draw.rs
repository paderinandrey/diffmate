mod draft;

use super::model::Session;
use crate::render::sanitize;
use crate::store::status::QueueRow;
use ratatui::Frame;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub(super) fn screen(frame: &mut Frame<'_>, session: &Session) {
    if let Some(confirm) = session.confirm_modal() {
        super::confirm::draw(frame, confirm);
        return;
    }
    if let Some((draft, focus)) = session.draft() {
        draft::screen(frame, draft, focus, session.notice());
        return;
    }
    let mut lines = if session.rows.is_empty() {
        vec![Line::from("No targets.")]
    } else {
        session
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let mark = if index == session.selected { ">" } else { " " };
                Line::from(format!("{mark} {}", row_text(row)))
            })
            .collect()
    };
    if let Some(notice) = session.notice() {
        lines.push(Line::from(sanitize(notice)));
    }
    frame.render_widget(Paragraph::new(lines), frame.area());
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
