use super::Layout;
use super::files::write;

const PROMPT: &str = include_str!("../../../../prompts/review.md");

pub(super) fn write_all(layout: &Layout, canaries: &[(&str, String)]) {
    let find = |name: &str| {
        canaries
            .iter()
            .find(|(have, _)| *have == name)
            .unwrap()
            .1
            .clone()
    };
    let dir = &layout.context;
    write(&dir.join("pr.md"), "# Title\nFix typo\n");
    write(&dir.join("diff.patch"), "diff --git a/app.py b/app.py\n");
    write(&dir.join("discussion.md"), "No discussion.\n");
    write(&dir.join("ci.md"), "CI status unknown\n");
    write(&dir.join("previous.json"), "{\"previous_run\": null}\n");
    let root = format!("# Context\n{}\n", find("context_root_claude_md"));
    write(&dir.join("CLAUDE.md"), &root);
    let rules = format!("# Rules\n{}\n", find("context_rules_claude_md"));
    write(&dir.join("rules/CLAUDE.md"), &rules);
    write(&layout.prompt.join("review.md"), PROMPT);
}
