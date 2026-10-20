pub mod agent_call;
pub mod stored;

use crate::support::fake_gh::pr_view_json;
use crate::support::sandbox::Sandbox;
use rusqlite::{Connection, OpenFlags};
use std::process::{Output, Stdio};

pub const URL: &str = "https://github.com/o/r/pull/1";
pub const BASE_LIB: &str = "fn a() {}\nfn b() {}\nfn c() {}\n";
pub const HEAD_LIB: &str = "fn a() {}\nfn b() {}\nfn c() { 1 }\n";

pub fn database(sandbox: &Sandbox) -> Connection {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY;
    Connection::open_with_flags(sandbox.db_path(), flags).unwrap()
}

pub fn count(db: &Connection, sql: &str) -> i64 {
    db.query_row(sql, [], |row| row.get(0)).unwrap()
}

pub fn review(sandbox: &Sandbox) -> Output {
    let mut command = sandbox.diffmate();
    command
        .args(["review", URL])
        .env("GH_TOKEN", "not-for-the-agent");
    command.stdin(Stdio::null()).output().unwrap()
}

pub fn with_pr(sandbox: &Sandbox) -> (String, String) {
    let upstream = sandbox.upstream("o", "r");
    let base = upstream.commit(&[("src/lib.rs", BASE_LIB)], "base");
    upstream.push_branch("main");
    let head = upstream.pr_head(1, "main", &[("src/lib.rs", HEAD_LIB)]);
    let json = pr_view_json("o", "r", 1, &head, "main", false);
    sandbox.gh.route(&format!("pr view {URL}"), &json);
    (base, head)
}
