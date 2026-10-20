mod caps;
mod files;
mod pr;
mod previous;
mod rules;
#[cfg(test)]
mod tests;

use crate::error::Error;
use crate::gh::PrMeta;
use crate::git::Git;
use crate::paths::create_private_dir_async;
use crate::pr_url::PrRef;
use crate::store::records::Revision;
use crate::store::{RunId, Store, TargetId};
use caps::{DIFF_CAP, cap_diff};
use files::put;
use std::collections::HashSet;
use std::path::Path;

pub(crate) use files::write_prompt;

pub(crate) struct ContextInputs<'a, G> {
    pub dir: &'a Path,
    pub pr: &'a PrRef,
    pub meta: &'a PrMeta,
    pub revision: &'a Revision,
    pub diff: Vec<u8>,
    pub repo: &'a Path,
    pub target: TargetId,
    pub run: RunId,
    pub git: &'a G,
    pub store: &'a Store,
}

pub(crate) struct WrittenContext {
    pub diff_as_written: Vec<u8>,
    pub previous_ids: HashSet<String>,
    pub warnings: Vec<String>,
}

fn note_truncation(warnings: &mut Vec<String>, name: &str, content: impl AsRef<[u8]>) {
    if content.as_ref().starts_with(b"TRUNCATED:") {
        warnings.push(format!(
            "{name} exceeded its size cap; the agent saw only the start of it"
        ));
    }
}

pub(crate) async fn write<G: Git>(inputs: ContextInputs<'_, G>) -> Result<WrittenContext, Error> {
    let made = create_private_dir_async(inputs.dir.to_path_buf()).await;
    made.map_err(Error::io("create context directory"))?;
    let mut warnings = Vec::new();
    let pr_md = pr::render(inputs.pr, inputs.meta, inputs.revision);
    note_truncation(&mut warnings, "pr.md", &pr_md);
    put(&inputs.dir.join("pr.md"), pr_md).await?;
    let diff_as_written = cap_diff(&inputs.diff, DIFF_CAP);
    note_truncation(&mut warnings, "diff.patch", &diff_as_written);
    put(&inputs.dir.join("diff.patch"), &diff_as_written).await?;
    let merge_base = inputs.revision.merge_base_sha.as_str();
    let at = (inputs.repo, merge_base);
    rules::write(inputs.git, at, inputs.dir, &mut warnings).await?;
    let ids = (inputs.target, inputs.run);
    let previous_ids = previous::write(inputs.dir, inputs.store, ids).await?;
    Ok(WrittenContext {
        diff_as_written,
        previous_ids,
        warnings,
    })
}
