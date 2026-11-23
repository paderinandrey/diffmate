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
        if self.state == "unknown" {
            let result = match self.reconcile.as_deref() {
                Some("failed") => format!(
                    "reconciliation failed: {}",
                    self.failure.as_deref().unwrap_or("unspecified error")
                ),
                Some("succeeded") => "marker not found; b: abandon only after checking".into(),
                Some(state) => format!("reconciliation {state}"),
                None => "outcome not confirmed".into(),
            };
            return format!("publication: unknown; {result}; y: reconcile; o: open PR");
        }
        if let Some(reason) = &self.reason {
            return format!("publication: {}: {reason}", self.state);
        }
        format!("publication: {}", self.state)
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
