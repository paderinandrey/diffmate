use super::{Session, bottom};
use crate::render::sanitize;
use Constraint::{Fill, Length};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Paragraph, Row, Table};

pub(super) fn screen(frame: &mut Frame<'_>, session: &Session, mut area: Rect) {
    if session.rows.is_empty() {
        frame.render_widget(Paragraph::new("No targets."), area);
        return;
    }
    let wide = area.width >= 120;
    if !wide
        && area.height >= 4
        && let Some(pr) = session.rows.get(session.selected)
    {
        let detail = format!(
            "{}  {}  {}",
            sanitize(&pr.label),
            sanitize(&pr.tracking),
            sanitize(&pr.freshness)
        );
        bottom(frame, &mut area, &detail, Style::default());
    }
    let header = area.height >= 7;
    let room = usize::from(area.height).saturating_sub(usize::from(header));
    let start = (session.selected + 1).saturating_sub(room);
    let rows = session
        .rows
        .iter()
        .enumerate()
        .skip(start)
        .take(room)
        .map(|(index, row)| {
            let selected = index == session.selected;
            let mark = if selected { "> " } else { "  " };
            let state = row
                .publication
                .split_once(": ")
                .map_or(row.publication.as_str(), |(state, _)| state);
            let mut cells = vec![
                format!("{mark}{}", sanitize(&row.label)),
                sanitize(&row.latest),
                sanitize(state),
            ];
            if wide {
                cells.extend([sanitize(&row.tracking), sanitize(&row.freshness)]);
            }
            let row = Row::new(cells);
            if selected { row.reversed() } else { row }
        });
    let widths: &[Constraint] = if wide {
        &[Fill(3), Fill(4), Length(11), Length(10), Fill(5)]
    } else {
        &[Fill(3), Fill(4), Length(11)]
    };
    let mut table = Table::new(rows, widths.iter().copied()).column_spacing(1);
    if header {
        let labels = [
            "  Pull request",
            "Latest review",
            "Publication",
            "Tracking",
            "Draft freshness",
        ];
        table = table.header(Row::new(labels.into_iter().take(widths.len())).bold());
    }
    frame.render_widget(table, area);
}
