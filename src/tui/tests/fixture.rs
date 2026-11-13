use super::fake::Fake;
use super::queue::HEAD;
use crate::config::Settings;
use crate::store::Store;
use crate::tui::model::Session;
use crate::tui::post::Mode;
use std::path::Path;

pub(super) const PUB: &str = "0123456789abcdef0123456789abcdef";
const SHOT: &str =
    r#"{"event":"COMMENT","findings":[],"limitation":null,"future_improvements":[]}"#;

pub(super) async fn plant(store: &Store, publication: &str, job: &str) {
    let head = HEAD;
    let publication = publication.to_string();
    let job = job.to_string();
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{head}', 1);
                 INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
                 INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'no_findings', '{head}', 'base', 'base', 'main', 0);
                 INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
                 INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at) VALUES ('{PUB}', 1, 1, 1, '{publication}', 'COMMENT', '{head}', '{SHOT}', 1, 1);
                 INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, publication_id) VALUES (2, 'publish', 1, '{job}', 1, 0, 2, 2, '{PUB}');"
            ))
        })
        .await
        .unwrap();
}

pub(super) async fn cell(store: &Store, sql: &str) -> String {
    let sql = sql.to_string();
    store
        .call(move |conn| conn.query_row(&sql, [], |row| row.get::<_, String>(0)))
        .await
        .unwrap()
}

pub(super) async fn open_session(store: &Store, lock: &Path) -> Session {
    let mut session = Session::load(store).await.unwrap();
    session.bind_runner(
        crate::app::ProcessInfo {
            pid: 7,
            started_at_ms: 8,
        },
        lock.to_path_buf(),
    );
    session
}

pub(super) fn mode<'a, H, G>(
    gh: &'a H,
    git: &'a G,
    paths: &'a crate::paths::Paths,
    settings: &'a Settings,
) -> Mode<'a, H, G> {
    Mode::On {
        gh,
        git,
        paths,
        settings,
    }
}

pub(super) async fn serve(store: &Store, paths: &crate::paths::Paths) -> Session {
    let mut session = open_session(store, &paths.lock()).await;
    let gh = Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    super::super::service(&mut session, store, &mode(&gh, &git, paths, &settings))
        .await
        .unwrap();
    session
}
