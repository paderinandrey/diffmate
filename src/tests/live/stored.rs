use crate::render::sanitize;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

type Finding = (Option<String>, Option<i64>, Option<String>, String, String);

pub(super) struct StoredRun {
    pub state: String,
    pub result: Option<String>,
    pub head_sha: String,
    pub failure: Option<String>,
    pub findings: Vec<Finding>,
}

impl StoredRun {
    pub fn describe(&self) -> String {
        let head = format!(
            "state {} result {:?} head {} failure {:?}",
            self.state, self.result, self.head_sha, self.failure
        );
        let lines = self.findings.iter().map(|(path, line, side, title, body)| {
            format!("  {path:?}:{line:?} {side:?} {title}\n    {body}")
        });
        let all: Vec<String> = std::iter::once(head).chain(lines).collect();
        sanitize(&all.join("\n"))
    }

    pub fn has_finding_at(&self, expected: &Value) -> bool {
        let path = expected["path"].as_str();
        let (line, side) = (expected["line"].as_i64(), expected["side"].as_str());
        let at =
            |(p, l, s, _, _): &Finding| p.as_deref() == path && *l == line && s.as_deref() == side;
        self.findings.iter().any(at)
    }
}

pub(super) fn latest_run(db: &Path) -> StoredRun {
    let conn = Connection::open(db).unwrap();
    let sql =
        "SELECT id, state, result, head_sha, failure_detail FROM runs ORDER BY id DESC LIMIT 1";
    let (id, state, result, head_sha, failure): (i64, _, _, _, _) = conn
        .query_row(sql, [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap();
    let sql =
        "SELECT path, line, side, title, body FROM findings WHERE run_id = ?1 ORDER BY ordinal";
    let mut stmt = conn.prepare(sql).unwrap();
    let rows = stmt
        .query_map([id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap();
    let findings = rows.map(Result::unwrap).collect();
    StoredRun {
        state,
        result,
        head_sha,
        failure,
        findings,
    }
}
