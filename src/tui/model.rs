use crate::error::Error;
use crate::store::Store;
use crate::store::status::{DraftView, QueueRow};

enum Screen {
    Queue,
    Draft(DraftView),
}

pub(super) struct Session {
    pub rows: Vec<QueueRow>,
    pub selected: usize,
    screen: Screen,
    focus: usize,
}

impl Session {
    pub async fn load(store: &Store) -> Result<Self, Error> {
        Ok(Self {
            rows: store.queue_rows().await?,
            selected: 0,
            screen: Screen::Queue,
            focus: 0,
        })
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
}
