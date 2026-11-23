use crate::render::sanitize;
use ratatui::Frame;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub(super) enum Purpose {
    Discard(crate::pr_url::PrRef),
    Cancel(String),
    Send(Offer),
    Abandon(String),
}

pub(super) enum Offer {
    New {
        displayed: std::sync::Arc<crate::store::status::DraftView>,
        event: String,
        sha: String,
    },
    Moved {
        draft: i64,
        publication: String,
        sha: String,
    },
}

pub(crate) struct Confirm {
    pub prompt: String,
    pub safe: String,
    pub danger: String,
    pub highlighted_safe: bool,
    pub(super) purpose: Purpose,
}

impl Confirm {
    pub(super) fn discard(pr: crate::pr_url::PrRef) -> Self {
        Self {
            prompt: format!("{} will be discarded.", pr.label()),
            safe: "Keep".to_string(),
            danger: "Discard".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Discard(pr),
        }
    }

    pub(super) fn cancel_queued(id: String) -> Self {
        Self {
            prompt: format!("Cancel the queued publication? {id}"),
            safe: "Keep queued".to_string(),
            danger: "Cancel publication".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Cancel(id),
        }
    }

    pub(super) fn send(prompt: String, offer: Offer) -> Self {
        Self {
            prompt,
            safe: "Don't send".to_string(),
            danger: "Send anyway".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Send(offer),
        }
    }

    pub(super) fn abandon_unknown(id: String) -> Self {
        Self {
            prompt: format!("Abandon this unknown publication? {id}"),
            safe: "Leave unknown".to_string(),
            danger: "Abandon".to_string(),
            highlighted_safe: true,
            purpose: Purpose::Abandon(id),
        }
    }
}

pub(super) fn draw(frame: &mut Frame<'_>, confirm: &Confirm, area: ratatui::layout::Rect) {
    let safe = mark(&confirm.safe, confirm.highlighted_safe);
    let danger = mark(&confirm.danger, !confirm.highlighted_safe);
    let lines = vec![
        Line::from(sanitize(&confirm.prompt)),
        Line::from(sanitize(&format!("{safe}  {danger}"))),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn mark(label: &str, highlighted: bool) -> String {
    if highlighted {
        format!("[{label}]")
    } else {
        label.to_string()
    }
}
