use super::model::Session;
use crate::error::Error;
use crate::store::Store;
mod row;
use row::Publication;

#[derive(PartialEq, Eq)]
pub(super) struct Report {
    pub target: i64,
    pub screen_draft: Option<i64>,
    pub id: String,
    pub text: String,
}

impl Publication {
    fn status(&self) -> String {
        let mut text = format!("publication: {}", self.state);
        if let Some(reason) = &self.reason {
            text.push_str(": ");
            text.push_str(reason);
        }
        if self.state == "unknown" {
            if self.reason.is_none() {
                text.push_str(": no confirmed response");
            }
            match self.reconcile.as_deref() {
                Some("failed") => text.push_str("; reconciliation failed"),
                Some("succeeded") => {
                    text.push_str("; marker not found; b: abandon only after checking");
                }
                Some(state) => {
                    text.push_str("; reconciliation ");
                    text.push_str(state);
                }
                None => {}
            }
            text.push_str("; y: reconcile; o: open PR");
        }
        text
    }
}

pub(super) async fn refresh(session: &mut Session, store: &Store) -> Result<(), Error> {
    if session.confirming() {
        return Ok(());
    }
    let Some(target) = session.selected_target() else {
        return Ok(());
    };
    let draft = session.draft_id();
    let publication = row::load(store, target, draft).await?;
    let Some(p) = publication else {
        session.publication = None;
        return Ok(());
    };
    if p.job == "needs_confirmation"
        && p.state == "pending"
        && let Some(seen) = &p.seen
    {
        let now = crate::store::now_ms();
        let line = crate::status::freshness::stored(&p.run, Some((seen.as_str(), now)), now);
        session.ask_moved(p.draft, &p.id, seen, line);
    }
    let text = p.status();
    session.publication = Some(std::sync::Arc::new(Report {
        target,
        screen_draft: draft,
        id: p.id,
        text,
    }));
    Ok(())
}
