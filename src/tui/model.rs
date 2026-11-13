use super::spawn::{Browser, BrowserCommand, Editor, EditorCommand, ReapGate};
use crate::app::ProcessInfo;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::store::Store;
use crate::store::jobs::EnqueueRequest;
use crate::store::records::JobOrigin;
use crate::store::status::{DraftView, QueueRow};
use std::path::PathBuf;
use std::sync::Arc;

enum Screen {
    Queue,
    Draft(DraftView),
}

pub(super) struct Session {
    pub rows: Vec<QueueRow>,
    pub selected: usize,
    screen: Screen,
    focus: usize,
    scroll: usize,
    view_width: usize,
    view_height: usize,
    notice: Option<String>,
    editor: Option<String>,
    file_seq: u64,
    editor_launch: Box<dyn Editor>,
    pub(super) reap: Arc<ReapGate>,
    browser_launch: Box<dyn Browser>,
    cache: PathBuf,
    live: bool,
    repaint: bool,
    confirm: Option<super::confirm::Confirm>,
    offer: Option<super::confirm::Offer>,
    lock_path: PathBuf,
    opener: Option<String>,
    process: ProcessInfo,
    runner: Option<crate::lock::RunnerLock>,
}

pub(super) struct Intent {
    pub draft: i64,
    pub target: i64,
    pub pr: PrRef,
    pub undecided: Vec<String>,
    pub pid: u32,
    pub started: i64,
}

impl Session {
    pub async fn load(store: &Store) -> Result<Self, Error> {
        let reap = Arc::new(ReapGate::new());
        Ok(Self {
            rows: store.queue_rows().await?,
            selected: 0,
            screen: Screen::Queue,
            focus: 0,
            scroll: 0,
            view_width: 0,
            view_height: 0,
            notice: None,
            editor: None,
            file_seq: 0,
            editor_launch: Box::new(EditorCommand::new(Arc::clone(&reap))),
            reap,
            browser_launch: Box::new(BrowserCommand),
            cache: PathBuf::new(),
            live: false,
            repaint: false,
            confirm: None,
            offer: None,
            lock_path: PathBuf::new(),
            opener: None,
            process: ProcessInfo {
                pid: std::process::id(),
                started_at_ms: 0,
            },
            runner: None,
        })
    }

    pub(super) fn intent(&self) -> Option<Intent> {
        let (draft, _) = self.draft()?;
        let row = self.rows.get(self.selected)?;
        let undecided = draft
            .findings
            .iter()
            .filter(|finding| finding.decision == "undecided")
            .map(|finding| finding.id.clone())
            .collect();
        Some(Intent {
            draft: draft.id,
            target: row.target_id,
            pr: row.pr.clone(),
            undecided,
            pid: self.process.pid,
            started: self.process.started_at_ms,
        })
    }

    pub(super) fn set_notice(&mut self, text: String) {
        self.notice = Some(text);
    }

    pub(super) fn running(&self) -> bool {
        self.runner.is_some()
    }

    pub(super) fn lock_path(&self) -> &std::path::Path {
        &self.lock_path
    }

    pub(super) fn process_info(&self) -> ProcessInfo {
        ProcessInfo {
            pid: self.process.pid,
            started_at_ms: self.process.started_at_ms,
        }
    }

    pub(super) fn hold(&mut self, lock: crate::lock::RunnerLock) {
        self.runner = Some(lock);
    }

    pub(super) async fn reload_rows(&mut self, store: &Store) -> Result<(), Error> {
        let kept = self.rows.get(self.selected).map(|row| row.target_id);
        self.rows = store.queue_rows().await?;
        self.selected = kept
            .and_then(|id| self.rows.iter().position(|row| row.target_id == id))
            .unwrap_or_else(|| self.rows.len().saturating_sub(1).min(self.selected));
        Ok(())
    }

    pub(super) fn take_repaint(&mut self) -> bool {
        std::mem::take(&mut self.repaint)
    }

    pub(super) fn bind_runner(&mut self, process: ProcessInfo, lock: PathBuf) {
        self.process = process;
        self.lock_path = lock;
    }

    #[cfg(test)]
    pub(super) fn arm_actions(&mut self, lock: Option<PathBuf>, opener: Option<String>) {
        if let Some(lock) = lock {
            self.lock_path = lock;
        }
        self.opener = opener;
    }

    pub(super) async fn open_pull(&mut self) -> Result<(), Error> {
        let Some(pr) = self.selected_pr() else {
            return Ok(());
        };
        let url = pr.web_url();
        let program = self.browser().to_owned();
        if let Err(error) = self.browser_launch.launch(&program, &url).await {
            self.notice = Some(format!("could not open the pull request: {error}"));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn use_editor(&mut self, editor: Box<dyn Editor>) {
        self.editor_launch = editor;
    }

    #[cfg(test)]
    pub(super) fn use_browser(&mut self, browser: Box<dyn Browser>) {
        self.browser_launch = browser;
    }

    pub(super) async fn rerun(&mut self, store: &Store) -> Result<(), Error> {
        let Some(pr) = self.selected_pr() else {
            return Ok(());
        };
        let watching = crate::lock::holder(&self.lock_path).await == Some(crate::lock::Role::Watch);
        let request = EnqueueRequest {
            origin: JobOrigin::Manual,
            force: false,
        };
        store.enqueue_review(&pr, &self.process, request).await?;
        if !watching {
            self.notice = Some("diffmate review or diffmate watch must run it".to_string());
        }
        Ok(())
    }

    fn browser(&self) -> &str {
        self.opener.as_deref().unwrap_or(super::open::PROGRAM)
    }

    pub(super) fn confirming(&self) -> bool {
        self.confirm.is_some()
    }

    pub(super) fn confirm_modal(&self) -> Option<&super::confirm::Confirm> {
        self.confirm.as_ref()
    }

    pub(super) fn ask_discard(&mut self) {
        let Some(label) = self.rows.get(self.selected).map(|row| row.label.clone()) else {
            return;
        };
        self.offer = None;
        self.confirm = Some(super::confirm::Confirm::discard(&label));
    }

    pub(super) fn ask_stale(&mut self, draft: i64, event: &str, sha: &str, prompt: String) {
        self.offer = Some(super::confirm::Offer {
            draft,
            event: event.to_string(),
            sha: sha.to_string(),
            publication: None,
        });
        self.confirm = Some(super::confirm::Confirm::stale(prompt));
    }

    pub(super) fn ask_moved(&mut self, draft: i64, publication: &str, sha: &str, prompt: String) {
        self.offer = Some(super::confirm::Offer {
            draft,
            event: String::new(),
            sha: sha.to_string(),
            publication: Some(publication.to_string()),
        });
        self.confirm = Some(super::confirm::Confirm::moved(prompt));
    }

    pub(super) fn highlight_confirm(&mut self, safe: bool) {
        if let Some(confirm) = &mut self.confirm {
            confirm.highlighted_safe = safe;
        }
    }

    pub(super) fn cancel_confirm(&mut self) {
        self.confirm = None;
        self.offer = None;
    }

    pub(super) async fn accept_confirm(
        &mut self,
        store: &Store,
    ) -> Result<Option<super::confirm::Offer>, Error> {
        let Some(confirm) = self.confirm.take() else {
            return Ok(None);
        };
        let offer = self.offer.take();
        if confirm.highlighted_safe {
            if confirm.is_moved()
                && let Some(id) = offer.and_then(|item| item.publication)
            {
                store.decline(&id).await?;
                self.reload_rows(store).await?;
            }
            return Ok(None);
        }
        if confirm.sends() {
            return Ok(offer);
        }
        if confirm.cancels() {
            self.cancel_publication(store).await?;
            return Ok(None);
        }
        if confirm.abandons() {
            self.abandon_selected(store).await?;
            return Ok(None);
        }
        let Some(pr) = self.selected_pr() else {
            return Ok(None);
        };
        store.discard(&pr).await?;
        self.rows = store.queue_rows().await?;
        Ok(None)
    }

    pub(super) async fn ask_abandon(&mut self, store: &Store) -> Result<(), Error> {
        let Some(target) = self.selected_target() else {
            return Ok(());
        };
        if store.unknown_publication(target).await?.is_none() {
            return Ok(());
        }
        self.offer = None;
        self.confirm = Some(super::confirm::Confirm::abandon_unknown());
        Ok(())
    }

    async fn abandon_selected(&mut self, store: &Store) -> Result<(), Error> {
        let Some(target) = self.selected_target() else {
            return Ok(());
        };
        if let Some(id) = store.unknown_publication(target).await? {
            store.abandon_unknown(&id).await?;
            self.reload_rows(store).await?;
        }
        Ok(())
    }

    pub(super) async fn ask_cancel(&mut self, store: &Store) -> Result<(), Error> {
        let Some(target) = self.selected_target() else {
            return Ok(());
        };
        match store.publish_gate(target).await? {
            crate::store::publish::Gate::Queued => {
                self.offer = None;
                self.confirm = Some(super::confirm::Confirm::cancel_queued());
            }
            crate::store::publish::Gate::Started => {
                self.set_notice("it already started".to_string());
            }
            crate::store::publish::Gate::Other => {}
        }
        Ok(())
    }

    async fn cancel_publication(&mut self, store: &Store) -> Result<(), Error> {
        let Some(target) = self.selected_target() else {
            return Ok(());
        };
        if matches!(
            store.cancel_queued(target).await?,
            crate::store::publish::CancelQueued::Started
        ) {
            self.set_notice("it already started".to_string());
        }
        self.reload_rows(store).await
    }

    pub(super) fn selected_target(&self) -> Option<i64> {
        self.rows.get(self.selected).map(|row| row.target_id)
    }

    fn selected_pr(&self) -> Option<PrRef> {
        self.rows.get(self.selected).map(|row| row.pr.clone())
    }

    pub(super) fn clear_notice(&mut self) {
        self.notice = None;
    }

    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    pub(super) fn prepare(&mut self, editor: Option<String>, cache: PathBuf, live: bool) {
        self.editor = editor;
        self.cache = cache;
        self.live = live;
    }

    pub fn on_draft(&self) -> bool {
        matches!(self.screen, Screen::Draft(_))
    }

    pub fn draft(&self) -> Option<(&DraftView, usize)> {
        match &self.screen {
            Screen::Draft(draft) => Some((draft, self.focus)),
            Screen::Queue => None,
        }
    }

    pub async fn open_draft(&mut self, store: &Store) -> Result<(), Error> {
        if self.on_draft() {
            return Ok(());
        }
        let Some(target) = self.rows.get(self.selected).map(|row| row.target_id) else {
            return Ok(());
        };
        if let Some(draft) = store.latest_draft(target).await? {
            self.screen = Screen::Draft(draft);
            self.focus = 0;
            self.scroll = 0;
        }
        Ok(())
    }

    pub fn close_draft(&mut self) {
        self.screen = Screen::Queue;
        self.focus = 0;
        self.scroll = 0;
    }

    fn span(&self) -> usize {
        match &self.screen {
            Screen::Queue => self.rows.len(),
            Screen::Draft(draft) => {
                let limitation = usize::from(draft.limitation.is_some());
                draft.findings.len() + limitation + draft.future.len()
            }
        }
    }

    pub(super) fn note_window(&mut self, width: usize, height: usize) {
        self.view_width = width;
        self.view_height = height;
    }

    pub(super) fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn move_down(&mut self) {
        if self.scroll_by(1) {
            return;
        }
        let Some(last) = self.span().checked_sub(1) else {
            return;
        };
        let on_draft = self.on_draft();
        let moved = {
            let cursor = if on_draft {
                &mut self.focus
            } else {
                &mut self.selected
            };
            if *cursor < last {
                *cursor += 1;
                true
            } else {
                false
            }
        };
        if moved && on_draft {
            self.scroll = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.scroll_by(-1) {
            return;
        }
        let on_draft = self.on_draft();
        let moved = {
            let cursor = if on_draft {
                &mut self.focus
            } else {
                &mut self.selected
            };
            let before = *cursor;
            *cursor = cursor.saturating_sub(1);
            *cursor != before
        };
        if moved && on_draft {
            self.scroll = 0;
        }
    }

    fn scroll_by(&mut self, step: i32) -> bool {
        let Some(limit) = self.scroll_limit() else {
            return false;
        };
        if self.scroll > limit {
            self.scroll = limit;
        }
        if step > 0 && self.scroll < limit {
            self.scroll += 1;
            return true;
        }
        if step < 0 && self.scroll > 0 {
            self.scroll -= 1;
            return true;
        }
        false
    }

    fn scroll_limit(&self) -> Option<usize> {
        if !self.on_draft() || self.view_width == 0 || self.view_height == 0 {
            return None;
        }
        let Screen::Draft(draft) = &self.screen else {
            return None;
        };
        let rows = super::draw::item_rows(draft, self.focus, self.view_width);
        Some(rows.saturating_sub(self.view_height))
    }

    pub async fn decide(&mut self, store: &Store, decision: &str) -> Result<(), Error> {
        let Some(id) = self.finding_id() else {
            return Ok(());
        };
        store.set_decision(&id, decision).await?;
        self.reload(store).await
    }

    pub async fn keep_rest(&mut self, store: &Store) -> Result<(), Error> {
        let Some(draft_id) = self.draft_id().filter(|_| self.finding_id().is_some()) else {
            return Ok(());
        };
        store.keep_undecided(draft_id).await?;
        self.reload(store).await
    }

    pub async fn edit_selected(&mut self, store: &Store) -> Result<(), Error> {
        let Some(piece) = self.piece() else {
            return Ok(());
        };
        let edited = super::edit::run(
            self.editor.as_deref(),
            &self.cache,
            &piece,
            self.live,
            &mut self.file_seq,
            &*self.editor_launch,
        )
        .await?;
        if self.live {
            self.repaint = true;
        }
        match edited {
            super::edit::Edited::Same => Ok(()),
            super::edit::Edited::Message(text) => {
                self.notice = Some(text.to_string());
                Ok(())
            }
            super::edit::Edited::Finding(title, body) => {
                let super::edit::Piece::Finding { id, .. } = &piece else {
                    return Ok(());
                };
                store.save_finding_text(id, &title, &body).await?;
                self.reload(store).await
            }
            super::edit::Edited::Note(text) => {
                let super::edit::Piece::Note {
                    draft_id, index, ..
                } = &piece
                else {
                    return Ok(());
                };
                store
                    .save_future_improvement(*draft_id, *index, &text)
                    .await?;
                self.reload(store).await
            }
        }
    }

    pub async fn include_limitation(&mut self, store: &Store) -> Result<(), Error> {
        let Some(draft_id) = self.draft_id() else {
            return Ok(());
        };
        if store.toggle_limitation(draft_id).await?.is_none() {
            self.notice = Some("this draft has no limitation".to_string());
        }
        self.reload(store).await
    }

    pub(super) fn edits_text(&self) -> bool {
        self.piece().is_some()
    }

    fn piece(&self) -> Option<super::edit::Piece> {
        let (draft, focus) = self.draft()?;
        if let Some(finding) = draft.findings.get(focus) {
            return Some(super::edit::Piece::Finding {
                id: finding.id.clone(),
                title: finding.title.clone(),
                body: finding.body.clone(),
            });
        }
        match super::draw::past_findings(draft, focus)? {
            super::draw::Place::Limitation => None,
            super::draw::Place::Note(index) => Some(super::edit::Piece::Note {
                draft_id: draft.id,
                index,
                text: draft.future.get(index)?.clone(),
            }),
        }
    }

    pub(super) async fn reload(&mut self, store: &Store) -> Result<(), Error> {
        if self.confirming() {
            return Ok(());
        }
        let Screen::Draft(_) = &self.screen else {
            return Ok(());
        };
        let Some(target) = self.rows.get(self.selected).map(|row| row.target_id) else {
            return Ok(());
        };
        if let Some(draft) = store.latest_draft(target).await? {
            self.screen = Screen::Draft(draft);
        }
        Ok(())
    }

    fn finding_id(&self) -> Option<String> {
        let (draft, focus) = self.draft()?;
        Some(draft.findings.get(focus)?.id.clone())
    }

    pub(super) fn draft_id(&self) -> Option<i64> {
        self.draft().map(|(draft, _)| draft.id)
    }
}
