mod shapes;

use super::{Comment, Discussion, Review, Thread};
use crate::error::Error;
use crate::gh::meta::Author;
use shapes::{Envelope, Page, RawComment, RawReview, RawThread};

fn who(author: Option<Author>) -> String {
    author.map_or_else(|| "ghost".into(), |author| author.login)
}

impl From<RawComment> for Comment {
    fn from(raw: RawComment) -> Comment {
        Comment {
            author: who(raw.author),
            body: raw.body,
            created_at: raw.created_at,
        }
    }
}

impl From<RawReview> for Review {
    fn from(raw: RawReview) -> Review {
        Review {
            author: who(raw.author),
            state: raw.state,
            body: raw.body,
            submitted_at: raw.submitted_at.unwrap_or_else(|| "not submitted".into()),
        }
    }
}

fn items<T, U: From<T>>(page: Page<T>, truncated: &mut bool) -> Vec<U> {
    *truncated |= page.page_info.has_next_page;
    page.nodes.into_iter().flatten().map(U::from).collect()
}

pub(super) fn parse(stdout: &[u8]) -> Result<Discussion, Error> {
    let envelope: Envelope = serde_json::from_slice(stdout)?;
    let pull = envelope.data.repository.and_then(|r| r.pull_request);
    let pull = pull.ok_or_else(|| Error::Command {
        program: "gh",
        args: "api graphql".into(),
        detail: "the response holds no pull request".into(),
    })?;
    let mut truncated = false;
    let comments = items(pull.comments, &mut truncated);
    let reviews = items(pull.reviews, &mut truncated);
    let mut threads = Vec::new();
    for raw in items::<_, RawThread>(pull.review_threads, &mut truncated) {
        threads.push(Thread {
            resolved: raw.is_resolved,
            outdated: raw.is_outdated,
            path: raw.path,
            line: raw.line,
            comments: items(raw.comments, &mut truncated),
        });
    }
    Ok(Discussion {
        comments,
        reviews,
        threads,
        truncated,
    })
}
