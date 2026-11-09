use super::super::URL;
use crate::support::fake_gh::pr_view_json;
use crate::support::sandbox::Sandbox;

pub fn push(sandbox: &Sandbox, parent: &str) -> String {
    let upstream = sandbox.upstream("o", "r");
    upstream.git(&["branch", "feature", parent]);
    upstream.pr_head(1, "feature", &[("src/lib.rs", "fn changed() {}\n")])
}

pub fn show(sandbox: &Sandbox, head: &str) {
    sandbox.gh.replace_route(
        &format!("pr view {URL}"),
        &pr_view_json("o", "r", 1, head, "main", false),
    );
}
