use crate::agent::FailureKind;
use crate::cancel::StopReason;

pub(crate) enum Failure {
    Timeout,
    Shutdown,
    UserCancel,
    Crash,
    TransientAgent,
    Terminal(FailureKind),
}

impl Failure {
    pub fn from_stop(stop: Option<StopReason>) -> Self {
        match stop {
            Some(StopReason::Timeout) => Self::Timeout,
            Some(StopReason::Shutdown) => Self::Shutdown,
            Some(StopReason::UserCancel) => Self::UserCancel,
            None => Self::Crash,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Timeout => "timeout",
            Self::Shutdown => "interrupted: shutdown",
            Self::UserCancel => "interrupted: user_cancel",
            Self::Crash => "interrupted: crash",
            Self::TransientAgent => "agent_error",
            Self::Terminal(kind) => kind.as_str(),
        }
    }
}
