#[path = "signals/helpers.rs"]
mod helpers;
#[path = "signals/process.rs"]
mod process;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::interrupted_by;
use rustix::process::Signal;

#[test]
fn sigint_during_agent_run_stops_group_and_records_interrupted() {
    interrupted_by(Signal::INT, "user_cancel", None);
}

#[test]
fn sigterm_during_agent_run_is_handled_the_same_way() {
    interrupted_by(Signal::TERM, "shutdown", None);
}

#[test]
fn sigint_then_sigterm_keeps_user_cancel() {
    interrupted_by(Signal::INT, "user_cancel", Some(Signal::TERM));
}
