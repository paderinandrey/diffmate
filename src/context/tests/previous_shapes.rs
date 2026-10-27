use crate::agent::{Severity, Side};
use crate::app::ProcessInfo;
use crate::context::previous;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::Store;
use crate::store::records::{DraftRecord, FindingRecord, Metrics, Revision};
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;

fn finding(line: Option<u32>, title: &str) -> FindingRecord {
    FindingRecord {
        severity: Severity::High,
        path: Some("src/lib.rs".into()),
        line,
        side: line.map(|_| Side::Right),
        title: title.into(),
        body: "Body.".into(),
        previous_finding_id: None,
    }
}

async fn written(store: &Store, dir: &std::path::Path, ids: (i64, i64)) -> (Value, Vec<String>) {
    let mut ids_found: Vec<_> = previous::write(dir, store, ids)
        .await
        .unwrap()
        .into_iter()
        .collect();
    ids_found.sort();
    let text = fs::read_to_string(dir.join("previous.json")).unwrap();
    (serde_json::from_str(&text).unwrap(), ids_found)
}

#[tokio::test]
async fn previous_json_is_unavailable_for_a_first_run_and_lists_earlier_findings_after() {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let process = ProcessInfo {
        pid: 1,
        started_at_ms: 0,
    };
    let job = store.enqueue_review(&pr, &process).await.unwrap();
    let sha = "a".repeat(40);
    let revision = Revision {
        head_sha: sha.clone(),
        merge_base_sha: sha.clone(),
        base_tip_sha: sha.clone(),
        base_ref: "main".into(),
    };
    let run = store.insert_running_run(&job, &revision).await.unwrap();
    let first = tmp.path().join("first");
    fs::create_dir(&first).unwrap();

    let (document, ids) = written(&store, &first, (job.target_id, run)).await;

    let reason = "no earlier successful run of this target";
    assert_eq!(document, json!({"available": false, "reason": reason}));
    assert_eq!(ids, Vec::<String>::new());
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
    let second = tmp.path().join("second");
    fs::create_dir(&second).unwrap();

    let (document, ids) = written(&store, &second, (job.target_id, run + 1)).await;

    assert_eq!(ids, [format!("{run}.1"), format!("{run}.2")]);
    assert_eq!(document["available"], true);
    assert_eq!(document["run_id"], run);
    assert_eq!(document["head_sha"], sha);
    let anchored = &document["findings"][0];
    assert_eq!(anchored["id"], format!("{run}.1"));
    assert_eq!(anchored["decision"], "undecided");
    assert_eq!(anchored["thread_status"], "unknown");
    assert_eq!(anchored["finding"]["line"], 3);
    assert_eq!(anchored["finding"]["title"], "Anchored");
    assert_eq!(document["findings"][1]["thread_status"], "none");
    assert_eq!(document["findings"][1]["finding"]["line"], Value::Null);
}
