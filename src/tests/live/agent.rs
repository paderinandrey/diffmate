use crate::agent::claude::{ClaudeCli, TestOverrides};
use crate::agent::{Agent, AgentOutcome, AgentRequest, FailureKind};
use crate::procenv::InheritedEnv;
use std::fs;
use std::path::Path;

const PROMPT: &str = include_str!("../../../prompts/review.md");
const CONTRACT: &str = include_str!("../../../tests/fixtures/claude/contract.json");
const DIFF: &str = "diff --git a/calc.py b/calc.py\nnew file mode 100644\n--- /dev/null\n+++ b/calc.py\n@@ -0,0 +1,2 @@\n+def last_item(items):\n+    return items[len(items)]\n";

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[tokio::test]
#[ignore = "needs the real claude CLI and a logged-in owner; makes one model call"]
async fn live_max_turns_one_is_failed_not_no_findings() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let (worktree, context, prompt) = (
        base.join("worktree"),
        base.join("context"),
        base.join("prompt/review.md"),
    );
    put(
        &worktree.join("calc.py"),
        "def last_item(items):\n    return items[len(items)]\n",
    );
    put(
        &context.join("pr.md"),
        "# Add last_item\n\nBranches: add-last into main.\n",
    );
    put(&context.join("diff.patch"), DIFF);
    put(&context.join("discussion.md"), "No discussion.\n");
    put(&context.join("ci.md"), "CI status is unavailable.\n");
    put(&prompt, PROMPT);
    let overrides = TestOverrides {
        extra_args: vec!["--max-turns".into(), "1".into()],
        ..TestOverrides::default()
    };
    let cli = ClaudeCli::new(&InheritedEnv::capture())
        .unwrap()
        .with_overrides(overrides);

    let outcome = cli.run(&AgentRequest::new(worktree, context, prompt)).await;

    let contract: serde_json::Value = serde_json::from_str(CONTRACT).unwrap();
    let expected = contract["max_turns"]["subtype"].as_str().unwrap();
    match outcome {
        AgentOutcome::Failed { kind, detail, .. } => {
            println!("failed ({}): {detail:?}", kind.as_str());
            assert!(matches!(kind, FailureKind::AgentError));
            assert_eq!(detail.subtype.as_deref(), Some(expected));
        }
        AgentOutcome::Succeeded { raw, .. } => panic!("recorded as succeeded: {raw}"),
        AgentOutcome::Cancelled { .. } => panic!("cancelled"),
    }
}
