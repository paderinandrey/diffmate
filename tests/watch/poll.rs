#[path = "poll/absent.rs"]
mod absent;
#[path = "poll/debounce.rs"]
mod debounce;
#[path = "poll/draft.rs"]
mod draft;
#[path = "poll/follow.rs"]
mod follow;
#[path = "poll/head.rs"]
mod head;
#[path = "poll/manual.rs"]
mod manual;
#[path = "poll/ready.rs"]
mod ready;
#[path = "poll/sync.rs"]
pub mod sync;

use super::{URL, debug_enqueue};
use crate::support::sandbox::Sandbox;

pub fn config(sandbox: &Sandbox, text: &str) {
    let dir = sandbox.home.join(".config/diffmate");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("config.toml"), text).unwrap();
}

pub fn search_hit(draft: bool) -> String {
    format!(
        "[{{\"number\":1,\"repository\":{{\"nameWithOwner\":\"o/r\"}},\"url\":\"{URL}\",\"isDraft\":{draft},\"state\":\"OPEN\"}}]"
    )
}

pub fn seed_tracked(sandbox: &Sandbox) {
    assert!(debug_enqueue(sandbox, URL, &[]).status.success());
    let db = rusqlite::Connection::open(sandbox.db_path()).unwrap();
    db.execute_batch(
        "DELETE FROM jobs;
         UPDATE targets SET tracking = 'tracked', request_seen = 1;",
    )
    .unwrap();
}
