use proptest::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Context,
    Add,
    Remove,
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub gap: u32,
    pub ops: Vec<Op>,
    pub no_newline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Added,
    Deleted,
    Modified,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct File {
    pub kind: Kind,
    pub old: String,
    pub new: String,
    pub hunks: Vec<Hunk>,
}

fn path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z0-9 éß中._-]{1,6}", 1..3).prop_map(|parts| parts.join("/"))
}

fn hunk() -> impl Strategy<Value = Hunk> {
    let op = prop_oneof![Just(Op::Context), Just(Op::Add), Just(Op::Remove)];
    (0..4u32, prop::collection::vec(op, 1..8), any::<bool>()).prop_map(|(gap, ops, no_newline)| {
        Hunk {
            gap,
            ops,
            no_newline,
        }
    })
}

fn build(index: usize, parts: (usize, String, String, Vec<Hunk>)) -> File {
    let (kind, old, new, mut hunks) = parts;
    let kind = [Kind::Added, Kind::Deleted, Kind::Modified, Kind::Renamed][kind];
    let new = format!("f{index}/{new}");
    let old = match kind {
        Kind::Renamed => format!("f{index}o/{old}"),
        _ => new.clone(),
    };
    if hunks.is_empty() && kind != Kind::Renamed {
        hunks.push(Hunk {
            gap: 0,
            ops: vec![Op::Context],
            no_newline: false,
        });
    }
    let only = match kind {
        Kind::Added => Some(Op::Add),
        Kind::Deleted => Some(Op::Remove),
        _ => None,
    };
    for hunk in &mut hunks {
        if let Some(op) = only {
            hunk.ops.iter_mut().for_each(|slot| *slot = op);
        }
    }
    File {
        kind,
        old,
        new,
        hunks,
    }
}

pub fn files() -> impl Strategy<Value = Vec<File>> {
    let one = (
        0..4usize,
        path(),
        path(),
        prop::collection::vec(hunk(), 0..4),
    );
    prop::collection::vec(one, 1..4).prop_map(|parts| {
        let numbered = parts.into_iter().enumerate();
        numbered.map(|(index, part)| build(index, part)).collect()
    })
}
