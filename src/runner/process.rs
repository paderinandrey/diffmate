use crate::crash::crash_point;
use crate::store::{RunId, Store};
use tokio::sync::mpsc::UnboundedReceiver;

pub(super) async fn record_agent_process(
    store: &Store,
    run: RunId,
    mut pids: UnboundedReceiver<u32>,
) -> Vec<String> {
    let mut problems = Vec::new();
    while let Some(pid) = pids.recv().await {
        if let Err(error) = store.set_agent_process(run, pid, pid).await {
            problems.push(format!("recording the agent process failed: {error}"));
        } else {
            crash_point!("agent-running");
        }
    }
    problems
}
