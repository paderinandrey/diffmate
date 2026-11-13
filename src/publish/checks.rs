use crate::error::Error;
use serde::Deserialize;

pub(super) struct Notes {
    pub limitation: Option<String>,
    pub future: Vec<String>,
}

pub(super) struct Unanchored {
    pub path: Option<String>,
    pub title: String,
    pub body: String,
}

pub(super) struct Anchored {
    pub id: String,
    pub path: String,
    pub line: u32,
    pub side: String,
    pub title: String,
    pub body: String,
}

#[derive(Deserialize)]
struct Shot {
    findings: Vec<ShotFinding>,
    limitation: Option<String>,
    future_improvements: Vec<String>,
}

#[derive(Deserialize)]
struct ShotFinding {
    id: String,
    path: Option<String>,
    line: Option<i64>,
    side: Option<String>,
    title: String,
    body: String,
}

pub(super) fn payload(snapshot: &str) -> Result<(Vec<Unanchored>, Vec<Anchored>, Notes), Error> {
    let shot: Shot = serde_json::from_str(snapshot)?;
    let mut loose = Vec::new();
    let mut anchored = Vec::new();
    for finding in shot.findings {
        match (finding.path, finding.line, finding.side) {
            (Some(path), Some(line), Some(side)) => anchored.push(Anchored {
                id: finding.id,
                path,
                line: u32::try_from(line).map_err(line_err)?,
                side,
                title: finding.title,
                body: finding.body,
            }),
            (path, _, _) => loose.push(Unanchored {
                path,
                title: finding.title,
                body: finding.body,
            }),
        }
    }
    let notes = Notes {
        limitation: shot.limitation,
        future: shot.future_improvements,
    };
    Ok((loose, anchored, notes))
}

pub(super) fn pending_review_of_mine(reviews: &[crate::gh::ListedReview], login: &str) -> bool {
    reviews
        .iter()
        .any(|review| review.author == login && review.state == "PENDING")
}

pub(super) fn transient_gh(error: &Error) -> bool {
    let Error::Command { detail, .. } = error else {
        return false;
    };
    match status_token(detail) {
        Some(code) => matches!(code, 408 | 429 | 500..=599),
        None => true,
    }
}

pub(super) fn client_rejected(error: &Error) -> bool {
    let Error::Command { detail, .. } = error else {
        return false;
    };
    matches!(
        status_token(detail),
        Some(code) if (400..500).contains(&code) && !matches!(code, 408 | 429)
    )
}

fn status_token(detail: &str) -> Option<u16> {
    let mut rest = detail;
    while let Some(at) = rest.find("HTTP ") {
        rest = &rest[at + "HTTP ".len()..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 3
            && let Some(token) = rest.get(..digits)
            && let Ok(code) = token.parse::<u16>()
            && (100..600).contains(&code)
        {
            return Some(code);
        }
    }
    None
}

fn line_err(_: std::num::TryFromIntError) -> Error {
    Error::Io {
        what: "publication snapshot".into(),
        source: std::io::Error::other("finding line does not fit"),
    }
}
