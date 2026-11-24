use crate::render::sanitize;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

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

pub(super) fn draw(frame: &mut Frame<'_>, confirm: &Confirm, mut area: Rect) {
    if area.height >= 6 {
        frame.render_widget(
            Paragraph::new("Confirm action").bold(),
            Rect { height: 1, ..area },
        );
        area.y += 2;
        area.height -= 2;
    }
    let safe = mark(&confirm.safe, confirm.highlighted_safe);
    let danger = mark(&confirm.danger, !confirm.highlighted_safe);
    let options = if confirm.highlighted_safe {
        vec![Span::from(safe).reversed(), "  ".into(), danger.into()]
    } else {
        vec![safe.into(), "  ".into(), Span::from(danger).reversed()]
    };
    let options = Line::from(options);
    let choice_rows = if options.width() > usize::from(area.width) {
        2
    } else {
        1
    };
    let hint = u16::from(area.height >= 5 && area.width >= 40);
    let available = area.height.saturating_sub(choice_rows + hint);
    let text = sanitize(&confirm.prompt);
    let prompt = super::draw::wrapped(&text, area.width, available);
    let prompt_rows = u16::try_from(prompt.height()).unwrap_or(available);
    frame.render_widget(
        Paragraph::new(prompt),
        Rect {
            height: prompt_rows,
            ..area
        },
    );
    let choices = Rect {
        y: area.y + prompt_rows + u16::from(prompt_rows < available),
        height: choice_rows.min(area.height),
        ..area
    };
    frame.render_widget(Paragraph::new(options).wrap(Wrap { trim: true }), choices);
    if hint != 0 {
        let keys = Paragraph::new("← → choose   Enter confirm   Esc cancel").italic();
        frame.render_widget(
            keys,
            Rect {
                y: choices.bottom(),
                height: 1,
                ..area
            },
        );
    }
}

fn mark(label: &str, highlighted: bool) -> String {
    if highlighted {
        format!("[{label}]")
    } else {
        label.to_string()
    }
}
