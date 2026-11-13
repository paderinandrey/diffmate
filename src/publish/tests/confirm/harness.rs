use super::super::support::{cell, open};
use super::seed;
use crate::gh::ListedReview;
use crate::paths::Paths;
use crate::tui::Fake;

pub(super) fn review(author: &str, state: &str) -> ListedReview {
    ListedReview {
        id: "7".into(),
        author: author.into(),
        state: state.into(),
        body: "leave this".into(),
    }
}

pub(super) async fn open_gh(head: &str) -> (tempfile::TempDir, crate::store::Store, Paths, Fake) {
    let (tmp, store) = open().await;
    seed(&store).await;
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    (tmp, store, paths, Fake::new(head))
}

pub(super) async fn pair(store: &crate::store::Store) -> (String, String) {
    let publication = cell::<String>(store, "SELECT state FROM publications").await;
    let job = cell::<String>(store, "SELECT state FROM jobs WHERE kind = 'publish'").await;
    (publication, job)
}
