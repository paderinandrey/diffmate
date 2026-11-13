mod remote;

use super::super::support::{KEPT, batch, open};
use crate::error::Error;
use std::future::ready;

pub(super) use super::super::support::cell;
use crate::gh::{CommentLink, ThreadState};
pub(super) use remote::Remote;

const SEED: &str = concat!(
    "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, github_review_id, created_at, updated_at) VALUES ('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 1, 1, 1, 'submitted', 'COMMENT', 'abc123', '",
    r#"{"findings":[{"id":"f1","path":"src/a.rs","line":4,"side":"RIGHT","title":"Title","body":"Body text"}],"limitation":null,"future_improvements":[]}"#,
    "', '9001', 1, 1);\n",
    "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, origin, publication_id) VALUES ('sync', 1, 'queued', 1, 0, 1, 1, 'manual', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa');\n",
    "INSERT INTO findings (id, run_id, draft_id, ordinal, severity, title, body, decision, thread_status) VALUES ('f2', 1, 1, 2, 'low', 'Loose', 'body', 'kept', 'none');"
);

pub(super) async fn seeded() -> (tempfile::TempDir, crate::store::Store, i64) {
    let (tmp, store) = open().await;
    batch(&store, KEPT).await;
    batch(&store, SEED).await;
    let job = cell(&store, "SELECT id FROM jobs WHERE kind = 'sync'").await;
    (tmp, store, job)
}

pub(super) fn sample() -> Vec<CommentLink> {
    vec![
        link("c1", "t1", "<!-- diffmate:finding=f1 -->"),
        link("c2", "t2", "<!-- diffmate:finding=f2 -->"),
    ]
}

fn link(comment_id: &str, thread_id: &str, body: &str) -> CommentLink {
    CommentLink {
        comment_id: comment_id.into(),
        thread_id: thread_id.into(),
        body: body.into(),
    }
}

pub(super) fn resolved() -> ThreadState {
    ThreadState {
        thread_id: "t1".into(),
        resolved: true,
    }
}

pub(super) fn unavailable() -> Error {
    Error::Command {
        program: "gh",
        args: String::new(),
        detail: "publication is not available on this gh".into(),
    }
}

pub(super) fn miss<T>() -> std::future::Ready<Result<T, Error>> {
    ready(Err(unavailable()))
}
