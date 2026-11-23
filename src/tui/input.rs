use super::model::Session;
use crate::store::status::DraftView;
use ratatui::crossterm::event::KeyEvent;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug)]
pub(super) struct Captured {
    pub key: KeyEvent,
    at: Instant,
}

impl From<KeyEvent> for Captured {
    fn from(key: KeyEvent) -> Self {
        Self {
            key,
            at: Instant::now(),
        }
    }
}

pub(super) struct Presented {
    since: Instant,
    target: Option<i64>,
    draft: Option<Arc<DraftView>>,
    publication: Option<Arc<super::follow::Report>>,
    focus: Option<usize>,
    modal: Option<(u64, bool)>,
}

impl Presented {
    pub fn draw(session: &Session, previous: Option<&Self>) -> Self {
        Self {
            since: previous
                .filter(|frame| frame.matches(session))
                .map_or_else(Instant::now, |frame| frame.since),
            target: session.selected_target(),
            draft: session.displayed_draft().cloned(),
            publication: session.publication_report().cloned(),
            focus: session.draft().map(|(_, focus)| focus),
            modal: Self::modal(session),
        }
    }

    fn matches(&self, session: &Session) -> bool {
        self.target == session.selected_target()
            && self.modal == Self::modal(session)
            && self.publication.as_ref() == session.publication_report()
            && self.focus == session.draft().map(|(_, focus)| focus)
            && match (&self.draft, session.displayed_draft()) {
                (None, None) => true,
                (Some(old), Some(new)) => Arc::ptr_eq(old, new) || old.same_content(new),
                _ => false,
            }
    }

    pub fn accepts(&self, session: &Session, key: &Captured) -> bool {
        key.at >= self.since && self.matches(session)
    }

    fn modal(session: &Session) -> Option<(u64, bool)> {
        session
            .confirm_modal()
            .map(|confirm| (session.modal_generation, confirm.highlighted_safe))
    }
}
