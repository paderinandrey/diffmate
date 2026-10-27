use crate::agent::claude::ClaudeCli;
use crate::agent::{Agent, AgentOutcome, AgentRequest};
use crate::procenv::InheritedEnv;
use std::fs;
use std::path::Path;

const PROMPT: &str = include_str!("../../../prompts/review.md");
const DIFF: &str = "diff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-# Demo projet\n+# Demo project\n";

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[tokio::test]
#[ignore = "needs the real claude CLI and a logged-in owner; makes one model call"]
async fn isolation_flags_keep_authentication() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let (worktree, context, prompt) = (
        base.join("worktree"),
        base.join("context"),
        base.join("prompt/review.md"),
    );
    put(&worktree.join("README.md"), "# Demo project\n");
    put(
        &context.join("pr.md"),
        "# Fix a typo in the README title\n\nBranches: typo into main.\n",
    );
    put(&context.join("diff.patch"), DIFF);
    put(&context.join("discussion.md"), "No discussion.\n");
    put(&context.join("ci.md"), "CI status is unavailable.\n");
    put(&context.join("rules/CLAUDE.md"), "Keep changes small.\n");
    put(&prompt, PROMPT);

    let cli = ClaudeCli::new(&InheritedEnv::capture()).unwrap();
    let outcome = cli.run(&AgentRequest::new(worktree, context, prompt)).await;
    match outcome {
        AgentOutcome::Succeeded { .. } => {}
        AgentOutcome::Failed { kind, detail, .. } => panic!("{}: {detail:?}", kind.as_str()),
        AgentOutcome::Cancelled { .. } => panic!("cancelled"),
    }
}
