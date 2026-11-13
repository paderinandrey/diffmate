use super::fixture::{self, open_session};
use super::publish::paths;
use super::queue::{self, HEAD, open};
use crate::config::Settings;
use crate::tui::confirm::Offer;
use crate::tui::post;

#[tokio::test]
async fn a_confirmation_for_another_draft_is_refused() {
    let (tmp, store) = open().await;
    queue::seed(&store, 1).await;
    let paths = paths(&tmp);
    let mut session = open_session(&store, &paths.lock()).await;
    session.open_draft(&store).await.unwrap();
    let gh = super::fake::Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let offer = Offer {
        draft: 2,
        event: "COMMENT".to_string(),
        sha: HEAD.to_string(),
        publication: None,
    };
    post::commit(
        &mut session,
        &store,
        &fixture::mode(&gh, &git, &paths, &settings),
        &offer,
        None,
    )
    .await
    .unwrap();
    assert_eq!(session.notice(), Some("the open draft changed"));
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        0
    );
    assert_eq!(gh.seen().len(), 0);
}
