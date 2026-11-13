use std::future::Future;
use std::pin::Pin;

pub(super) type ProbeFuture<'a> = Pin<Box<dyn Future<Output = Probe> + Send + 'a>>;

#[derive(Clone)]
pub(super) enum Probe {
    Ready,
    Unfinished,
    Failed(String),
}

pub(super) trait AuthProbe: Send + Sync {
    fn status(&self) -> ProbeFuture<'_>;
}

pub(super) trait AgentProbe: Send + Sync {
    fn version(&self) -> ProbeFuture<'_>;
    fn help(&self) -> ProbeFuture<'_>;
}
