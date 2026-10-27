use super::model::{File, Kind, Op};
use std::collections::BTreeSet;

pub struct Placed {
    pub old: (u32, u32),
    pub new: (u32, u32),
    pub ops: Vec<Op>,
    pub no_newline: bool,
}

pub fn place(file: &File) -> Vec<Placed> {
    let (mut old, mut new) = (1, 1);
    let mut placed = Vec::new();
    for hunk in &file.hunks {
        (old, new) = (old + hunk.gap, new + hunk.gap);
        let (start_old, start_new) = (old, new);
        for op in &hunk.ops {
            old += u32::from(!matches!(op, Op::Add));
            new += u32::from(!matches!(op, Op::Remove));
        }
        placed.push(Placed {
            old: (start_old, old - start_old),
            new: (start_new, new - start_new),
            ops: hunk.ops.clone(),
            no_newline: hunk.no_newline,
        });
    }
    placed
}

pub fn anchor(file: &File) -> &str {
    if file.kind == Kind::Deleted {
        &file.old
    } else {
        &file.new
    }
}

pub fn expected(file: &File) -> (BTreeSet<u32>, BTreeSet<u32>) {
    let (mut left, mut right) = (BTreeSet::new(), BTreeSet::new());
    for hunk in place(file) {
        let (mut old, mut new) = (hunk.old.0, hunk.new.0);
        for op in hunk.ops {
            if !matches!(op, Op::Add) {
                left.insert(old);
                old += 1;
            }
            if !matches!(op, Op::Remove) {
                right.insert(new);
                new += 1;
            }
        }
    }
    (left, right)
}
