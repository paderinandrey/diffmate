use super::{count, database};
use crate::support::git_in;
use crate::support::sandbox::Sandbox;
use std::fs;

pub fn assert_stored(sandbox: &Sandbox, base: &str, head: &str) {
    let db = database(sandbox);
    assert_eq!(
        count(&db, "SELECT count(*) FROM jobs WHERE state = 'succeeded'"),
        1
    );
    let sql = "SELECT result, head_sha, merge_base_sha FROM runs WHERE state = 'succeeded'";
    let row = |r: &rusqlite::Row| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    };
    assert_eq!(
        db.query_row(sql, [], row).unwrap(),
        ("findings".into(), head.to_string(), base.to_string())
    );
    assert_eq!(count(&db, "SELECT count(*) FROM drafts"), 1);
    let id: String = db
        .query_row("SELECT id FROM findings", [], |r| r.get(0))
        .unwrap();
    assert_eq!(id, "1.1");
    assert_eq!(
        fs::read_dir(sandbox.cache_dir().join("runs"))
            .unwrap()
            .count(),
        0
    );
    let repo = sandbox.state_dir().join("repos/o/r.git");
    let worktrees = git_in(&repo, &sandbox.home, &["worktree", "list", "--porcelain"]);
    assert_eq!(worktrees.matches("worktree ").count(), 1, "{worktrees}");
    let fetched = git_in(
        &repo,
        &sandbox.home,
        &["rev-parse", "refs/diffmate/pull/1/head"],
    );
    assert_eq!(fetched, head);
}
