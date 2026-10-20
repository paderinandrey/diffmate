use std::collections::HashSet;
use std::iter::Peekable;

#[derive(Default)]
pub(super) struct Lines {
    pub left: HashSet<u32>,
    pub right: HashSet<u32>,
}

pub(super) struct Header {
    old: (u32, u32),
    new: (u32, u32),
}

fn range(text: &str) -> Option<(u32, u32)> {
    match text.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((text.parse().ok()?, 1)),
    }
}

pub(super) fn header(line: &str) -> Option<Header> {
    let rest = line.strip_prefix("@@ -")?;
    let (old, rest) = rest.split_once(" +")?;
    let (new, _) = rest.split_once(" @@")?;
    Some(Header {
        old: range(old)?,
        new: range(new)?,
    })
}

pub(super) fn consume<'a, I: Iterator<Item = &'a str>>(
    header: &Header,
    lines: &mut Peekable<I>,
    out: &mut Lines,
) {
    let ((mut old, mut old_left), (mut new, mut new_left)) = (header.old, header.new);
    while old_left > 0 || new_left > 0 {
        let Some(line) = lines.peek() else { return };
        match line.as_bytes().first() {
            Some(b'\\') => {}
            Some(b'-') => {
                out.left.insert(old);
                (old, old_left) = (old.saturating_add(1), old_left.saturating_sub(1));
            }
            Some(b'+') => {
                out.right.insert(new);
                (new, new_left) = (new.saturating_add(1), new_left.saturating_sub(1));
            }
            Some(b' ') | None => {
                out.left.insert(old);
                out.right.insert(new);
                (old, old_left) = (old.saturating_add(1), old_left.saturating_sub(1));
                (new, new_left) = (new.saturating_add(1), new_left.saturating_sub(1));
            }
            Some(_) => return,
        }
        lines.next();
    }
}
