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

fn line_err(_: std::num::TryFromIntError) -> Error {
    Error::Io {
        what: "publication snapshot".into(),
        source: std::io::Error::other("finding line does not fit"),
    }
}
