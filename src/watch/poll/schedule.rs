use crate::app::AppConfig;
use crate::gh::Gh;
use crate::store::Store;
use std::future::Future;
use std::time::Duration;
use tokio::time::{Instant, sleep};

pub(in crate::watch) struct Schedule<'a, H> {
    config: &'a AppConfig,
    store: &'a Store,
    gh: &'a H,
    last: Option<Instant>,
}

impl<'a, H: Gh> Schedule<'a, H> {
    pub fn new(config: &'a AppConfig, store: &'a Store, gh: &'a H) -> Self {
        Self {
            config,
            store,
            gh,
            last: None,
        }
    }

    pub fn delay(&self) -> Duration {
        self.last.map_or(Duration::ZERO, |last| {
            self.config
                .settings
                .poll_interval()
                .saturating_sub(last.elapsed())
        })
    }

    pub async fn poll(&mut self) {
        if let Err(error) = super::once(
            self.gh,
            self.store,
            self.config.settings.debounce(),
            &self.config.process,
        )
        .await
        {
            crate::watch::events::print(&format!("poll failed: {error}"));
        }
        self.last = Some(Instant::now());
    }

    pub async fn during<F: Future>(&mut self, job: F) -> F::Output {
        let mut job = std::pin::pin!(job);
        loop {
            tokio::select! {
                biased;
                result = &mut job => return result,
                () = async {
                    sleep(self.delay()).await;
                    self.poll().await;
                } => {}
            }
        }
    }
}
