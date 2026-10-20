use super::ClaudeCli;
use crate::agent::probe::PROBE_OK_TEXT;
use crate::agent::{Agent, AgentOutcome, AgentRequest};
use crate::procenv::InheritedEnv;
use rustix::process::{Pid, test_kill_process};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::TempDir;
use tokio::sync::{mpsc, watch};

fn install_fake(dir: &Path) -> ClaudeCli {
    let script = format!(
        "#!/bin/sh\n[ \"$1\" = \"--version\" ] && {{ echo '2.1.304 (Claude Code)'; exit 0; }}\n\
         [ -z \"$(cat)\" ] && {{ echo '{PROBE_OK_TEXT}' >&2; exit 1; }}\nexec sleep 300\n"
    );
    let bin = dir.join("fake-bin");
    fs::create_dir(&bin).unwrap();
    fs::write(bin.join("claude"), script).unwrap();
    fs::set_permissions(bin.join("claude"), fs::Permissions::from_mode(0o755)).unwrap();
    let inherited = std::env::var_os("PATH").unwrap();
    let dirs = std::iter::once(bin).chain(std::env::split_paths(&inherited));
    let path = std::env::join_paths(dirs).unwrap();
    ClaudeCli::new(&InheritedEnv::from_pairs(vec![("PATH".into(), path)])).unwrap()
}

fn request(dir: &Path) -> AgentRequest {
    let (worktree, context) = (dir.join("worktree"), dir.join("context"));
    fs::create_dir_all(&worktree).unwrap();
    fs::create_dir_all(&context).unwrap();
    fs::create_dir_all(dir.join("prompt")).unwrap();
    let prompt = dir.join("prompt/review.md");
    fs::write(&prompt, "prompt").unwrap();
    AgentRequest::new(worktree, context, prompt)
}

#[tokio::test]
async fn cancelling_a_running_agent_stops_it_after_reporting_its_pid() {
    let tmp = TempDir::new().unwrap();
    let cli = install_fake(tmp.path());
    let (stop, cancel) = watch::channel(false);
    let (hook, mut pids) = mpsc::unbounded_channel();
    let request = request(tmp.path())
        .with_cancel(cancel)
        .with_spawn_hook(hook);
    let canceller = async {
        let pid = pids.recv().await.unwrap();
        stop.send_replace(true);
        pid
    };

    let (outcome, pid) = tokio::join!(cli.run(&request), canceller);

    assert!(matches!(outcome, AgentOutcome::Cancelled { .. }));
    let pid = Pid::from_raw(i32::try_from(pid).unwrap()).unwrap();
    assert!(test_kill_process(pid).is_err());
}

#[tokio::test]
async fn an_already_cancelled_request_never_starts_the_agent() {
    let tmp = TempDir::new().unwrap();
    let cli = install_fake(tmp.path());
    let (_stop, cancel) = watch::channel(true);
    let (hook, mut pids) = mpsc::unbounded_channel();
    let request = request(tmp.path())
        .with_cancel(cancel)
        .with_spawn_hook(hook);

    let outcome = cli.run(&request).await;

    assert!(matches!(outcome, AgentOutcome::Cancelled { .. }));
    drop(request);
    assert_eq!(pids.recv().await, None);
}
