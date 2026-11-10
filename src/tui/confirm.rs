use crate::render::sanitize;
use ratatui::Frame;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

enum Purpose {
    Discard,
    Cancel,
    Stale,
    Moved,
}

pub(super) struct Offer {
    pub event: String,
    pub sha: String,
    pub publication: Option<String>,
}

pub(crate) struct Confirm {
    pub prompt: String,
    pub safe: String,
    pub danger: String,
    pub highlighted_safe: bool,
    purpose: Purpose,
}

impl Confirm {
    pub(crate) fn discard(label: &str) -> Self {
        Self {
            prompt: format!("{label} will be discarded."),
            safe: "Keep".to_string(),
            danger: "Discard".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Discard,
        }
    }

    pub(crate) fn cancel_queued() -> Self {
        Self {
            prompt: "Cancel the queued publication?".to_string(),
            safe: "Keep queued".to_string(),
            danger: "Cancel publication".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Cancel,
        }
    }

    pub(crate) fn stale(prompt: String) -> Self {
        Self {
            prompt,
            safe: "Don't send".to_string(),
            danger: "Send anyway".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Stale,
        }
    }

    pub(super) fn cancels(&self) -> bool {
        matches!(self.purpose, Purpose::Cancel)
    }

    pub(crate) fn moved(prompt: String) -> Self {
        Self {
            prompt,
            safe: "Don't send".to_string(),
            danger: "Send anyway".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Moved,
        }
    }

    pub(super) fn sends(&self) -> bool {
        matches!(self.purpose, Purpose::Stale | Purpose::Moved)
    }

    pub(super) fn is_moved(&self) -> bool {
        matches!(self.purpose, Purpose::Moved)
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
