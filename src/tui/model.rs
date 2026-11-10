use crate::error::Error;
use crate::store::Store;
use crate::store::status::QueueRow;

pub(super) struct Session {
    pub rows: Vec<QueueRow>,
    pub selected: usize,
}

impl Session {
    pub async fn load(store: &Store) -> Result<Self, Error> {
        Ok(Self {
            rows: store.queue_rows().await?,
            selected: 0,
        })
    }

    pub fn move_down(&mut self) {
        let Some(last) = self.rows.len().checked_sub(1) else {
            return;
        };
        if self.selected < last {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}
