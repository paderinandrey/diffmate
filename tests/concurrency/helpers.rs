use crate::support::fake_gh::pr_view_json;
use crate::support::sandbox::Sandbox;
use crate::tracer::{BASE_LIB, HEAD_LIB};
use std::process::{Child, Stdio};

pub fn url(number: u64) -> String {
    format!("https://github.com/o/r/pull/{number}")
}

pub fn with_two_prs(sandbox: &Sandbox) {
    let upstream = sandbox.upstream("o", "r");
    upstream.commit(&[("src/lib.rs", BASE_LIB)], "base");
    upstream.push_branch("main");
    for number in [1, 2] {
        let head = upstream.pr_head(number, "main", &[("src/lib.rs", HEAD_LIB)]);
        let json = pr_view_json("o", "r", number, &head, "main", false);
        sandbox.gh.route(&format!("pr view {}", url(number)), &json);
    }
}

pub fn spawn_review(sandbox: &Sandbox, number: u64) -> Child {
    let mut command = sandbox.diffmate();
    command.args(["review", &url(number)]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().unwrap()
}
