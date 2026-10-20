use super::caps::{DISCUSSION_CAP, cap_text, reason};
use crate::gh::{Comment, Discussion, Gh, Review, Thread};
use crate::pr_url::PrRef;

const PAGES: &str = "TRUNCATED: discussion.md lists only the first 100 comments, 50 reviews, 100 review threads and 50 comments per thread; GitHub holds more.\n";

fn comment(c: &Comment) -> String {
    format!(
        "### {} at {}\n\n{}\n",
        c.author,
        c.created_at,
        c.body.trim_end()
    )
}

fn review(r: &Review) -> String {
    let (who, state, when) = (&r.author, &r.state, &r.submitted_at);
    format!("### {who} [{state}] at {when}\n\n{}\n", r.body.trim_end())
}

fn thread(t: &Thread) -> String {
    let path = t.path.as_deref().unwrap_or("(no file)");
    let place = t
        .line
        .map_or_else(|| path.to_string(), |line| format!("{path}:{line}"));
    let state = if t.resolved {
        "[resolved]"
    } else {
        "[unresolved]"
    };
    let outdated = if t.outdated { " (outdated)" } else { "" };
    let notes: Vec<_> = t.comments.iter().map(comment).collect();
    format!("### {place} {state}{outdated}\n\n{}", notes.join("\n"))
}

fn section(title: &str, items: &[String]) -> String {
    let body = if items.is_empty() {
        "None.\n".to_string()
    } else {
        items.join("\n")
    };
    format!("## {title}\n\n{body}\n")
}

fn render(d: &Discussion) -> String {
    let marker = if d.truncated { PAGES } else { "" };
    let comments = section(
        "Comments",
        &d.comments.iter().map(comment).collect::<Vec<_>>(),
    );
    let reviews = section("Reviews", &d.reviews.iter().map(review).collect::<Vec<_>>());
    let threads = section(
        "Review threads",
        &d.threads.iter().map(thread).collect::<Vec<_>>(),
    );
    format!("{marker}# Discussion\n\n{comments}{reviews}{threads}")
}

pub(super) async fn text<H: Gh>(gh: &H, pr: &PrRef) -> String {
    match gh.discussion(pr).await {
        Ok(found) => cap_text("discussion.md", &render(&found), DISCUSSION_CAP),
        Err(error) => format!("Discussion unavailable: {}\n", reason(&error)),
    }
}
