use crate::tui::spawn::{Editor, EditorCommand};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[tokio::test]
async fn dropping_the_editor_reaps_the_child() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("ed");
    std::fs::write(
        &script,
        "#!/bin/sh\necho $$ > \"$1\"\nwhile true; do :; done\n",
    )
    .unwrap();
    let mut perms = std::fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o700);
    std::fs::set_permissions(&script, perms).unwrap();
    let note = dir.path().join("note");
    let argv = vec![script.display().to_string()];
    let pid = editor_pid(&argv, &note).await;
    let raw = i32::try_from(pid).unwrap();
    let checked = rustix::process::Pid::from_raw(raw).unwrap();
    assert!(rustix::process::test_kill_process(checked).is_err());
}

async fn editor_pid(argv: &[String], note: &Path) -> u32 {
    let launch = EditorCommand.launch(argv, note);
    tokio::pin!(launch);
    for _ in 0..10_000 {
        if let Ok(text) = std::fs::read_to_string(note)
            && let Ok(pid) = text.trim().parse::<u32>()
            && pid > 1
        {
            return pid;
        }
        tokio::select! {
            biased;
            ended = &mut launch => panic!("editor ended: {ended:?}"),
            () = tokio::task::yield_now() => {}
        }
    }
    panic!("editor pid");
}
