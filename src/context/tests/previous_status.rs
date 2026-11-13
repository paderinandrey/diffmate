use super::fake_gh::{FakeGh, pr};
use super::previous_shapes::finding;
use crate::app::ProcessInfo;
use crate::context::previous;
use crate::gh::ThreadState;
use crate::paths::Paths;
use crate::store::Store;
use crate::store::records::{DraftRecord, Metrics, Revision};
use crate::store::{jobs::EnqueueRequest, records::JobOrigin};
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn linked_thread_is_resolved_and_a_body_finding_stays_none() {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    let process = ProcessInfo {
        pid: 1,
        started_at_ms: 0,
    };
    let request = EnqueueRequest {
        origin: JobOrigin::Manual,
        force: false,
    };
    let job = store
        .enqueue_review(&pr(), &process, request)
        .await
        .unwrap()
        .job;
    let sha = "b".repeat(40);
    let revision = Revision {
        head_sha: sha.clone(),
        merge_base_sha: sha.clone(),
        base_tip_sha: sha,
        base_ref: "main".into(),
    };
    let run = store.insert_running_run(&job, &revision).await.unwrap();
    let draft = DraftRecord {
        original_output: json!({}),
        limitation: None,
        future_improvements: vec![],
        findings: vec![finding(Some(3), "Anchored"), finding(None, "Body")],
    };
    store
        .finish_succeeded(run, job.job_id, draft, vec![], Metrics::default())
        .await
        .unwrap();
    let anchored = format!("{run}.1");
    let id = anchored.clone();
    store
        .call(move |conn| {
            conn.execute(
                "UPDATE findings SET github_thread_id = 't1' WHERE id = ?1",
                [id],
            )
        })
        .await
        .unwrap();
    let dir = tmp.path().join("miss");
    fs::create_dir(&dir).unwrap();
    let missed = read(&store, &dir, &FakeGh::default(), (job.target_id, run + 1)).await;
    assert_eq!(missed["findings"][0]["thread_status"], "unknown");
    assert_eq!(missed["findings"][0]["decision"], "undecided");
    assert_eq!(missed["findings"][1]["thread_status"], "none");
    let gh = FakeGh::default();
    let rows = vec![ThreadState {
        thread_id: "t1".into(),
        resolved: true,
    }];
    *gh.threads.lock().unwrap() = Some(rows);
    let dir = tmp.path().join("hit");
    fs::create_dir(&dir).unwrap();
    let document = read(&store, &dir, &gh, (job.target_id, run + 1)).await;
    assert_eq!(document["findings"][0]["id"], anchored);
    assert_eq!(document["findings"][0]["thread_status"], "resolved");
    assert_eq!(document["findings"][0]["decision"], "undecided");
    assert_eq!(document["findings"][1]["thread_status"], "none");
    assert_eq!(document["findings"][1]["finding"]["line"], Value::Null);
}

async fn read(store: &Store, dir: &std::path::Path, gh: &FakeGh, ids: (i64, i64)) -> Value {
    previous::write(dir, store, gh, &pr(), ids).await.unwrap();
    serde_json::from_str(&fs::read_to_string(dir.join("previous.json")).unwrap()).unwrap()
}
