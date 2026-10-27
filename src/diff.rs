mod hunk;
mod path;
#[cfg(test)]
mod tests;

use crate::agent::Side;
use hunk::Lines;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DiffIndex {
    files: HashMap<String, Lines>,
}

impl DiffIndex {
    pub fn contains(&self, path: &str, side: Side, line: u32) -> bool {
        self.files.get(path).is_some_and(|lines| match side {
            Side::Left => lines.left.contains(&line),
            Side::Right => lines.right.contains(&line),
        })
    }
}

#[derive(Default)]
struct Section {
    old: Option<String>,
    new: Option<String>,
}

pub(crate) fn parse(diff: &[u8]) -> DiffIndex {
    let text = String::from_utf8_lossy(diff);
    let mut lines = text.lines().peekable();
    let mut index = DiffIndex::default();
    let mut section: Option<Section> = None;
    while let Some(line) = lines.next() {
        if line.starts_with("diff --git ") {
            section = Some(Section::default());
        } else if let Some(section) = section.as_mut() {
            if let Some(rest) = line.strip_prefix("--- ") {
                section.old = path::target(rest, 'a');
            } else if let Some(rest) = line.strip_prefix("+++ ") {
                section.new = path::target(rest, 'b');
            } else if let Some(header) = hunk::header(line) {
                let mut found = Lines::default();
                hunk::consume(&header, &mut lines, &mut found);
                let key = section.new.as_ref().or(section.old.as_ref());
                if let Some(key) = key {
                    let entry = index.files.entry(key.clone()).or_default();
                    entry.left.extend(found.left);
                    entry.right.extend(found.right);
                }
            }
        }
    }
    index
}
