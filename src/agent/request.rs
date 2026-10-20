use crate::cancel::CancelRx;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct AgentRequest {
    pub worktree: PathBuf,
    pub context_dir: PathBuf,
    pub prompt_file: PathBuf,
    pub(super) cancel: Option<CancelRx>,
    pub(super) spawn_hook: Option<UnboundedSender<u32>>,
}

impl AgentRequest {
    pub fn new(worktree: PathBuf, context_dir: PathBuf, prompt_file: PathBuf) -> AgentRequest {
        AgentRequest {
            worktree,
            context_dir,
            prompt_file,
            cancel: None,
            spawn_hook: None,
        }
    }

    #[cfg_attr(not(test), expect(dead_code))]
    pub fn with_cancel(mut self, cancel: CancelRx) -> AgentRequest {
        self.cancel = Some(cancel);
        self
    }

    #[cfg_attr(not(test), expect(dead_code))]
    pub fn with_spawn_hook(mut self, hook: UnboundedSender<u32>) -> AgentRequest {
        self.spawn_hook = Some(hook);
        self
    }
}
