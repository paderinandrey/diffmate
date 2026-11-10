use crate::app::ProcessInfo;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::store::Store;
use crate::store::jobs::EnqueueRequest;
use crate::store::records::JobOrigin;
use crate::store::status::{DraftView, QueueRow};
use std::path::PathBuf;

enum Screen {
    Queue,
    Draft(DraftView),
}

pub(super) struct Session {
    pub rows: Vec<QueueRow>,
    pub selected: usize,
    screen: Screen,
    focus: usize,
    notice: Option<String>,
    editor: Option<String>,
    cache: PathBuf,
    live: bool,
    confirm: Option<super::confirm::Confirm>,
    lock_path: PathBuf,
    opener: Option<String>,
    process: ProcessInfo,
}

impl Session {
    pub async fn load(store: &Store) -> Result<Self, Error> {
        Ok(Self {
            rows: store.queue_rows().await?,
            selected: 0,
            screen: Screen::Queue,
            focus: 0,
            notice: None,
            editor: None,
            cache: PathBuf::new(),
            live: false,
            confirm: None,
            lock_path: PathBuf::new(),
            opener: None,
            process: ProcessInfo {
                pid: std::process::id(),
                started_at_ms: 0,
            },
        })
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
        if let Err(error) = super::open::open_pr(self.browser(), &url).await {
            self.notice = Some(format!("could not open the pull request: {error}"));
        }
        Ok(())
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
        self.confirm = Some(super::confirm::Confirm::discard(&label));
    }

    pub(super) fn highlight_confirm(&mut self, safe: bool) {
        if let Some(confirm) = &mut self.confirm {
            confirm.highlighted_safe = safe;
        }
    }

    pub(super) fn cancel_confirm(&mut self) {
        self.confirm = None;
    }

    pub(super) async fn accept_confirm(&mut self, store: &Store) -> Result<(), Error> {
        let Some(confirm) = self.confirm.take() else {
            return Ok(());
        };
        if confirm.highlighted_safe {
            return Ok(());
        }
        let Some(pr) = self.selected_pr() else {
            return Ok(());
        };
        store.discard(&pr).await?;
        self.rows = store.queue_rows().await?;
        Ok(())
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
        }
        Ok(())
    }

    pub fn close_draft(&mut self) {
        self.screen = Screen::Queue;
        self.focus = 0;
    }

    fn span(&self) -> usize {
        match &self.screen {
            Screen::Queue => self.rows.len(),
            Screen::Draft(draft) => draft.findings.len() + draft.future.len(),
        }
    }

    pub fn move_down(&mut self) {
        let Some(last) = self.span().checked_sub(1) else {
            return;
        };
        let cursor = if self.on_draft() {
            &mut self.focus
        } else {
            &mut self.selected
        };
        if *cursor < last {
            *cursor += 1;
        }
    }

    pub fn move_up(&mut self) {
        let cursor = if self.on_draft() {
            &mut self.focus
        } else {
            &mut self.selected
        };
        *cursor = cursor.saturating_sub(1);
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
        let edited =
            super::edit::run(self.editor.as_deref(), &self.cache, &piece, self.live).await?;
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

    fn piece(&self) -> Option<super::edit::Piece> {
        let (draft, focus) = self.draft()?;
        if let Some(finding) = draft.findings.get(focus) {
            return Some(super::edit::Piece::Finding {
                id: finding.id.clone(),
                title: finding.title.clone(),
                body: finding.body.clone(),
            });
        }
        let index = focus.checked_sub(draft.findings.len())?;
        Some(super::edit::Piece::Note {
            draft_id: draft.id,
            index,
            text: draft.future.get(index)?.clone(),
        })
    }

    async fn reload(&mut self, store: &Store) -> Result<(), Error> {
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

    fn draft_id(&self) -> Option<i64> {
        self.draft().map(|(draft, _)| draft.id)
    }
}
