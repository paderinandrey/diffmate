use crate::render::sanitize;
use ratatui::Frame;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub(crate) struct Confirm {
    pub prompt: String,
    pub safe: String,
    pub danger: String,
    pub highlighted_safe: bool,
}

impl Confirm {
    pub(crate) fn discard(label: &str) -> Self {
        Self {
            prompt: format!("{label} will be discarded."),
            safe: "Keep".to_string(),
            danger: "Discard".to_string(),
            highlighted_safe: true,
        }
    }
}

pub(super) fn draw(frame: &mut Frame<'_>, confirm: &Confirm) {
    let safe = mark(&confirm.safe, confirm.highlighted_safe);
    let danger = mark(&confirm.danger, !confirm.highlighted_safe);
    let lines = vec![
        Line::from(sanitize(&confirm.prompt)),
        Line::from(sanitize(&format!("{safe}  {danger}"))),
    ];
    frame.render_widget(Paragraph::new(lines), frame.area());
}

fn mark(label: &str, highlighted: bool) -> String {
    if highlighted {
        format!("[{label}]")
    } else {
        label.to_string()
    }
}
