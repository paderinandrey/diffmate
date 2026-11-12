use crate::error::Error;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommentLink {
    pub comment_id: String,
    pub thread_id: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreadState {
    pub thread_id: String,
    pub resolved: bool,
}

pub(super) fn comments(stdout: &[u8], review_id: &str) -> Result<Vec<CommentLink>, Error> {
    let wanted = review_id.parse::<i64>().ok();
    let mut links = Vec::new();
    for thread in nodes(&pull(stdout)?, "reviewThreads") {
        let Some(thread_id) = thread["id"].as_str() else {
            continue;
        };
        for node in nodes(&thread, "comments") {
            if node["pullRequestReview"]["databaseId"].as_i64() != wanted {
                continue;
            }
            if let Some(link) = one(&node, thread_id) {
                links.push(link);
            }
        }
    }
    Ok(links)
}

pub(super) fn statuses(stdout: &[u8]) -> Result<Vec<ThreadState>, Error> {
    let mut rows = Vec::new();
    for node in nodes(&pull(stdout)?, "reviewThreads") {
        let Some(thread_id) = node["id"].as_str() else {
            continue;
        };
        let Some(resolved) = node["isResolved"].as_bool() else {
            continue;
        };
        rows.push(ThreadState {
            thread_id: thread_id.to_string(),
            resolved,
        });
    }
    Ok(rows)
}

fn one(node: &Value, thread_id: &str) -> Option<CommentLink> {
    Some(CommentLink {
        comment_id: node["databaseId"].as_i64()?.to_string(),
        thread_id: thread_id.to_string(),
        body: node["body"].as_str()?.to_string(),
    })
}

fn pull(stdout: &[u8]) -> Result<Value, Error> {
    let value: Value = serde_json::from_slice(stdout)?;
    value
        .pointer("/data/repository/pullRequest")
        .cloned()
        .ok_or_else(|| Error::Command {
            program: "gh",
            args: "api graphql".into(),
            detail: "the response holds no pull request".into(),
        })
}

fn nodes(value: &Value, field: &str) -> Vec<Value> {
    value[field]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{comments, statuses};

    #[test]
    fn comments_keep_the_submitted_review() {
        let raw = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[{"id":"t1","comments":{"nodes":[{"databaseId":7,"body":"<!-- diffmate:finding=f1 -->","pullRequestReview":{"databaseId":9001}}]}},{"id":"t2","comments":{"nodes":[{"databaseId":8,"body":"other","pullRequestReview":{"databaseId":3}}]}}]}}}}}"#;
        let links = comments(raw.as_bytes(), "9001").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].comment_id, "7");
        assert_eq!(links[0].thread_id, "t1");
    }

    #[test]
    fn statuses_read_is_resolved() {
        let raw = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[{"id":"t1","isResolved":false}]}}}}}"#;
        let rows = statuses(raw.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].resolved);
        assert_eq!(rows[0].thread_id, "t1");
    }
}
