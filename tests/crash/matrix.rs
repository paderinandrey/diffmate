use super::*;
use helpers::crash_at_ready;
#[path = "matrix/assertions.rs"]
mod assertions;

fn killed_at(point: &str) {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    let hold = (point == "agent-running").then(|| sandbox.claude.hold_run(1));
    crash_at_ready(sandbox.diffmate().args(["review", URL]), point, || {
        if let Some(hold) = &hold {
            hold.wait_started();
        }
    });
    if let Some(hold) = hold {
        hold.release();
    }
    if point == "agent-start" {
        assert!(sandbox.claude.calls().is_empty());
    }
    let resumed = sandbox.run(&["review", URL]);
    assert!(resumed.status.success(), "{resumed:?}");
    assertions::restarted(&sandbox, point, &head, &resumed);
    assert_clean(&sandbox);
}

macro_rules! crash_case {
    ($name:ident, $point:literal) => {
        #[test]
        fn $name() {
            killed_at($point);
        }
    };
}

crash_case!(enqueued, "enqueued");
crash_case!(locked, "locked");
crash_case!(recovery_db, "recovery-db");
crash_case!(recovered, "recovered");
crash_case!(job_started, "job-started");
crash_case!(fetched, "fetched");
crash_case!(revision, "revision");
crash_case!(run_created, "run-created");
crash_case!(worktree, "worktree");
crash_case!(context, "context");
crash_case!(agent_start, "agent-start");
crash_case!(agent_running, "agent-running");
crash_case!(cleaned, "cleaned");
crash_case!(stored, "stored");
