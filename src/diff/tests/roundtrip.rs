use super::super::parse;
use super::model::{File, Kind, Op, files};
use super::place::{Placed, anchor, expected, place};
use proptest::prelude::*;

const CONTENT: [&str; 5] = ["code", "-- a", "++ b", "@@ -1 +1 @@", "diff --git a b"];

fn name(path: &str, prefix: &str, sign: &str) -> String {
    if !path.is_ascii() {
        let octal: String = path
            .bytes()
            .map(|b| {
                if b.is_ascii() {
                    char::from(b).to_string()
                } else {
                    format!("\\{b:03o}")
                }
            })
            .collect();
        format!("{sign} \"{prefix}/{octal}\"\n")
    } else if path.contains(' ') {
        format!("{sign} {prefix}/{path}\t\n")
    } else {
        format!("{sign} {prefix}/{path}\n")
    }
}

fn span((start, count): (u32, u32)) -> String {
    match count {
        0 => format!("{},0", start - 1),
        1 => start.to_string(),
        _ => format!("{start},{count}"),
    }
}

fn body(hunk: &Placed) -> Vec<String> {
    let mut lines = vec![format!("@@ -{} +{} @@", span(hunk.old), span(hunk.new))];
    for (n, op) in hunk.ops.iter().enumerate() {
        let sign = match op {
            Op::Context => ' ',
            Op::Add => '+',
            Op::Remove => '-',
        };
        lines.push(format!("{sign}{}", CONTENT[n % CONTENT.len()]));
    }
    if hunk.no_newline {
        lines.push("\\ No newline at end of file".to_string());
    }
    lines
}

fn render(file: &File) -> String {
    let mut lines = vec![format!("diff --git a/{} b/{}", file.old, file.new)];
    lines.push(match file.kind {
        Kind::Added => "new file mode 100644".to_string(),
        Kind::Deleted => "deleted file mode 100644".to_string(),
        Kind::Modified => "index 1111111..2222222 100644".to_string(),
        Kind::Renamed => format!("rename from {}\nrename to {}", file.old, file.new),
    });
    let hunks = place(file);
    if !hunks.is_empty() {
        let old = match file.kind {
            Kind::Added => "--- /dev/null\n".to_string(),
            _ => name(&file.old, "a", "---"),
        };
        let new = match file.kind {
            Kind::Deleted => "+++ /dev/null\n".to_string(),
            _ => name(&file.new, "b", "+++"),
        };
        lines.push(format!("{old}{new}").trim_end_matches('\n').to_string());
    }
    lines.extend(hunks.iter().flat_map(body));
    lines.join("\n") + "\n"
}

proptest! {
    #[test]
    fn the_index_equals_the_model(files in files(), truncated in any::<bool>()) {
        let marker = if truncated { "TRUNCATED: cut\n" } else { "" };
        let text: String = files.iter().map(render).collect();
        let index = parse(format!("{marker}{text}").as_bytes());
        let with_hunks = files.iter().filter(|file| !file.hunks.is_empty());
        prop_assert_eq!(index.files.len(), with_hunks.count());
        for file in &files {
            let (left, right) = expected(file);
            for n in 0..=60 {
                use crate::agent::Side;
                prop_assert_eq!(index.contains(anchor(file), Side::Left, n), left.contains(&n));
                prop_assert_eq!(index.contains(anchor(file), Side::Right, n), right.contains(&n));
            }
        }
    }
}
