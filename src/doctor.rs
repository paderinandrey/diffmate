mod claude;
mod gh_auth;
mod lock_line;
mod schema;
mod scratch;
mod storage;
mod which;

use crate::app::AppConfig;
use crate::render::sanitize;
use std::process::ExitCode;

pub(crate) struct Check {
    passed: bool,
    text: String,
}

impl Check {
    pub(super) fn pass(text: impl Into<String>) -> Self {
        Self {
            passed: true,
            text: text.into(),
        }
    }

    pub(super) fn fail(text: impl Into<String>) -> Self {
        Self {
            passed: false,
            text: text.into(),
        }
    }

    pub(super) fn with_extra(self, extra: &str) -> Self {
        Self::fail(format!("{}; {extra}", self.text))
    }
}

fn print_line(text: &str) {
    let clean = sanitize(text).replace(['\n', '\r'], " ");
    println!("{clean}");
}

pub(crate) async fn run(config: &AppConfig) -> ExitCode {
    let checks = [
        gh_auth::check(config).await,
        claude::check(config).await,
        storage::check(config).await,
        lock_line::check(config).await,
        schema::check(config).await,
    ];
    let passed = checks.iter().all(|check| check.passed);
    for check in &checks {
        print_line(&check.text);
    }
    if passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
