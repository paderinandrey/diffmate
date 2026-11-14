---
phase: 01-single-review
plan: 07
subsystem: review-output
tags: [rust, unified-diff, anchoring, normalization, terminal-sanitizing, proptest, d-04]

requires:
  - phase: 01-single-review
    provides: "01-04 tracer slice (runner, outcome storage, render), 01-05 agent contract, 01-06 git hardening, 01-03 black-box harness"
provides:
  - "DiffIndex: hand-written unified-diff parser mapping every path to its anchorable RIGHT and LEFT line sets (REV-03)"
  - "normalize: out-of-hunk anchors move to the review body with the SPEC warning, unknown previous_finding_id values become null with a warning"
  - "D-04 plain-text output with an Unanchored findings list, limitation and future improvements; visible no_findings result (REV-04)"
  - "sanitize strips C0, C1 and bidirectional controls at print time only; SQLite keeps the raw text"
affects: [01-08, 01-09, 01-10, 01-11, 01-12]

actuals:
  tokens: 10000
  tasks: 3
  commits: 3

plan_head_before: 3baa82a0c21b72132030003fd5919399209b2764
plan_head_after: 9edd707a66de70db13285ea05a2980b8feafd16a
commits: 3

tech-stack:
  added: []
  patterns:
    - "The diff parser consumes hunk bodies by the counts in the hunk header, so content that looks like a header stays content"
    - "normalize is a pure function of (output, DiffIndex, known previous ids); the runner owns the call site, so plan 01-09 only changes its arguments"
    - "Test model generators and renderers live in sibling test modules (tests/model.rs, tests/place.rs) to stay under the 100-line hook"

key-files:
  created:
    - src/diff.rs
    - src/diff/hunk.rs
    - src/diff/path.rs
    - src/diff/tests.rs
    - src/diff/tests/model.rs
    - src/diff/tests/place.rs
    - src/diff/tests/roundtrip.rs
    - src/diff/tests/never_panics.rs
    - src/normalize.rs
    - src/normalize/tests.rs
    - src/normalize/tests/fixture.rs
    - src/normalize/tests/property.rs
    - src/render/sanitize.rs
    - src/render/tests.rs
    - src/render/tests/fixture.rs
    - src/render/tests/sanitize.rs
    - tests/review_output.rs
    - tests/review_output/helpers.rs
    - tests/review_output/render.rs
  modified:
    - src/main.rs
    - src/render.rs
    - src/runner.rs
    - src/runner/steps.rs
    - src/runner/outcome.rs
    - tests/tracer/mod.rs

key-decisions:
  - "Anchors are keyed by the new path, or the old path for deleted files; LEFT lines of a renamed file are recorded under the new path (one path per finding)"
  - "The runner parses the diff inside steps::prepare, right before context::write moves the bytes, and hands the DiffIndex to the success branch; the run directory is already cleaned up by then"
  - "The blank line after the header is printed only when something follows it; unanchored blocks have no trailing blank line, as the D-04 description in the plan reads literally"
  - "U+061C (Arabic letter mark) is stripped together with the listed bidi controls, because it is a Unicode bidirectional control character"

patterns-established:
  - "Black-box output tests share helpers in tests/review_output/helpers.rs (finding_rows, run_warnings, review_one, anchored) loaded with #[path]"

requirements-completed: [REV-02, REV-03, REV-04]

coverage:
  - id: D1
    description: "Any diff git produces maps to exact anchorable line sets per path and side: omitted counts, new, deleted and renamed files, binary and mode-only sections, no-newline markers, quoted and tab-terminated paths, a leading TRUNCATED line, header-looking content"
    requirement: REV-03
    verification:
      - kind: unit
        ref: "src/diff/tests.rs (9 case tests)"
        status: pass
      - kind: property
        ref: "src/diff/tests/roundtrip.rs#the_index_equals_the_model"
        status: pass
      - kind: property
        ref: "src/diff/tests/never_panics.rs#arbitrary_bytes_never_panic and #diff_shaped_text_never_panics"
        status: pass
    human_judgment: false
  - id: D2
    description: "Every stored anchor is inside the diff the agent saw; an out-of-hunk anchor is stored with line and side NULL, keeps its path and records the exact warning; a LEFT anchor on a removed line is kept; an unknown previous_finding_id is nulled with a warning"
    requirement: REV-03
    verification:
      - kind: property
        ref: "src/normalize/tests/property.rs#every_stored_anchor_is_inside_the_diff"
        status: pass
      - kind: integration
        ref: "tests/review_output.rs#out_of_hunk_finding_moves_to_review_body"
        status: pass
      - kind: integration
        ref: "tests/review_output.rs#left_anchor_on_removed_line_is_kept"
        status: pass
      - kind: integration
        ref: "tests/review_output.rs#unknown_previous_finding_id_is_nulled"
        status: pass
    human_judgment: false
  - id: D3
    description: "review prints the D-04 layout exactly (header, anchored blocks, Unanchored findings list, Limitation, Future improvements), warnings go to stderr as warning: lines, and no_findings is visible and never printed for a failure"
    requirement: REV-04
    verification:
      - kind: unit
        ref: "src/render/tests.rs (4 exact-output tests)"
        status: pass
      - kind: integration
        ref: "tests/review_output/render.rs#no_findings_prints_status_and_notes"
        status: pass
    human_judgment: false
  - id: D4
    description: "No C0, C1 or bidirectional control character other than newline and tab reaches the terminal from model-derived text; the findings row keeps the raw text"
    requirement: REV-04
    verification:
      - kind: property
        ref: "src/render/tests/sanitize.rs#nothing_that_drives_the_terminal_survives"
        status: pass
      - kind: integration
        ref: "tests/review_output/render.rs#terminal_control_characters_are_stripped"
        status: pass
    human_judgment: false
  - id: D5
    description: "GitHub accepts LEFT anchors on context lines and the new path of renamed files, and a real clean PR gets no_findings from the model"
    requirement: REV-03
    verification: []
    human_judgment: true
    rationale: "GitHub's acceptance is only observable at publication (Phase 4, research A8) and the model's verdict on a real clean PR belongs to plan 01-12"

duration: 14min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 07: Anchoring and D-04 output Summary

**Hunk-index anchoring with a hand-written unified-diff parser (proptested against a git-format model), SPEC normalization with recorded warnings, and the D-04 plain-text output with bidi-safe terminal sanitizing**

## Performance

- **Duration:** about 14 min (start time was not recorded; counted from the end of plan 01-06 at 18:33Z)
- **Tasks:** 3
- **Files:** 25 changed in the plan range (19 created, 6 modified in src and tests), 1190 insertions and 84 deletions

## Accomplishments

- `diff::parse` reads whatever `git diff --no-ext-diff --no-textconv` prints. Hunk bodies are consumed by the old and new counts, so a removed line `-- a` (rendered `--- a`) or an added `++ b` stays content. Omitted counts mean one, `@@ -0,0 +1,3 @@` is a new file, `+++ /dev/null` keys a deleted file by its old path, a rename with hunks is keyed by the new path, binary, mode-only and pure-rename sections add nothing, `\ No newline at end of file` does not shift numbers, C-style quoted paths decode octal bytes as UTF-8, one trailing tab is removed, and lines before the first `diff --git` (the TRUNCATED marker) are ignored. Line arithmetic is saturating, so no input panics.
- The round-trip proptest renders generated files (added, deleted, modified, renamed; paths with spaces and non-ASCII characters; optional missing trailing newline; header-looking content) in git's format and requires the index to equal the model's RIGHT and LEFT sets per anchor path, and no phantom paths. Mutation check (observed, reverted): removing the tab strip made the round trip and the tab case fail, so neither is vacuous.
- `normalize(output, index, previous_ids)` keeps findings in order and in number, moves an out-of-hunk anchor to the body (line and side None, path kept) with `finding N: PATH:LINE (SIDE) is not inside a hunk of diff.patch; moved to the review body`, and nulls an unknown `previous_finding_id` with the SPEC-derived warning. The runner calls it with an empty previous-ID set on `AgentOutcome::Succeeded`; the warnings are stored in `runs.warnings`, returned in `ReviewResult::Succeeded` and printed on stderr. `original_output` keeps the agent's unchanged JSON.
- `render::review` prints the D-04 layout. `sanitize` additionally removes the bidirectional controls and runs only at print time; the black-box test shows the findings row keeps the ESC sequence and U+202E.

## Task Commits

1. **Task 1: Unified-diff hunk index with property tests** - `b04d6ac` (feat)
2. **Task 2: Normalization wired into the run (REV-03)** - `679a08e` (feat)
3. **Task 3: D-04 output and terminal sanitizing (REV-04)** - `9edd707` (feat)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Decisions Made

- See `key-decisions` in the frontmatter.
- Flagged for the owner (plan section "Flagged assumptions"): LEFT anchors print like RIGHT ones (`path:line [severity] title`), so the side is visible only in the database. Unanchored blocks are printed without a blank line between them, which reads the plan literally but is dense for multi-line bodies; one blank line after each unanchored block is a one-line change if wanted.
- `normalize` also clears `side` whenever `line` ends up null, because the findings table has `CHECK ((line IS NULL) = (side IS NULL))` and a bad combination would abort the storing transaction.

## Verification

Observed in this session:

- `mise run --force test -- -E 'test(diff::)'`: 12 passed (before Task 1 commit).
- `mise run --force test -- -E 'test(normalize::) | binary(review_output) | binary(review)'`: 10 passed (Task 2).
- `mise run --force test -- -E 'test(render::) | binary(review_output)'`: 11 passed (Task 3).
- `mise run --force test`: 103 run, 103 passed, 3 skipped (the live probes).
- `mise run --force lint`: exit 0 (rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks, actionlint).
- `mise run coverage`: 93.18% line coverage (gate 80%); no gate failure printed.
- Acceptance greps: `grep -c 'pub(crate) struct DiffIndex' src/diff.rs` = 1, `grep -c 'normalize(' src/runner.rs` = 1, `grep -c 'Unanchored findings:' src/render.rs` = 1; every file the plan touches is at most 100 lines (render.rs 97, runner.rs 92, review_output.rs 63, steps.rs 94); no unwrap, expect or panic outside test code in the new modules.
- The hk pre-commit hook (lint, large-files, secrets, docs, test, actions) passed on all three commits without `--no-verify`.
- Not run: `mise run vuln` and `mise run deps:check` (no crate was added), `mise run test:e2e`, `mise run docs:check` outside the hook.

## Deviations from Plan

**1. [Rule 3 - Blocking] Extra files for the 100-line commit hook**
- **Issue:** the parser, the test model and the renderer tests do not fit the planned files under 100 lines.
- **Fix:** added `src/diff/hunk.rs`, `src/diff/tests/{model,place,roundtrip,never_panics}.rs`, `src/normalize/tests/{fixture,property}.rs`, `src/render/sanitize.rs`, `src/render/tests/{fixture,sanitize}.rs`, `tests/review_output/{helpers,render}.rs` (the last two loaded with `#[path]`, as `tests/agent_contract.rs` does). Public paths named in the plan interfaces hold: `crate::diff::{DiffIndex, parse}`, `crate::normalize::{Normalized, normalize}`, `crate::render::{review, sanitize}`.
- **Committed in:** b04d6ac, 679a08e, 9edd707

**2. [Rule 3 - Blocking] The runner change lives in steps.rs and outcome.rs too**
- **Issue:** the diff bytes are held in `steps::prepare`, the success branch is in `runner/outcome.rs`, and `runner.rs` was at 96 lines.
- **Fix:** `prepare` returns `(AgentRequest, DiffIndex)`; the prepare-and-run block with its prepare-failure outcome moved from `runner.rs` into `steps::run_agent`; `outcome::store` takes the normalized findings and warnings (and the old always-null previous_finding_id handling there is gone). `runner.rs` still holds the `normalize(` call. `tests/tracer/mod.rs` gained `with_pr_head` for a custom PR head.
- **Committed in:** 679a08e

**3. [Rule 3 - Blocking] Temporary dead-code expectation**
- **Issue:** Task 1 commits `mod diff` before anything outside tests uses it.
- **Fix:** `#[cfg_attr(not(test), expect(dead_code))]` in Task 1, removed in Task 2 when `normalize` started using it.
- **Committed in:** b04d6ac, 679a08e

**4. [Rule 2 - Missing critical] U+061C stripped as a bidirectional control**
- **Issue:** the plan lists U+200E, U+200F, U+202A to U+202E and U+2066 to U+2069, while its own truth says no Unicode bidirectional control character may reach the terminal; the Arabic letter mark U+061C is also one.
- **Fix:** added to `sanitize` and to the proptest oracle.
- **Committed in:** 9edd707

**5. [Rule 1 - Bug] Task 2 black-box test could not assert the Unanchored list yet**
- **Issue:** the plan puts the `Unanchored findings:` assertion in Task 2's test, but the layout is Task 3's work and each commit must pass the test hook.
- **Fix:** the Task 2 commit asserted the inline tracer form (`src/lib.rs [high] Bad bound`) and Task 3 changed the same test to require the `Unanchored findings:` block.
- **Committed in:** 679a08e, 9edd707

**Total deviations:** 5 (3 blocking, 1 missing-critical, 1 test sequencing). **Impact:** none on behavior beyond the plan; AgentOutcome, ReviewOutput, FindingRecord and the Git and Gh traits are unchanged.

## Issues Encountered

- `ReviewResult` still carries `#[expect(dead_code)]`: nothing reads `run_id` (the D-04 header does not print it), so the attribute stays for the first plan that does.
- A mutation run left a `proptest-regressions/` directory; it was deleted and nothing from it was committed.
- `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were already modified or untracked and were left out of every commit.

## User Setup Required

None.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-29 (anchors checked against the hunk index, proptest over generated outputs, corrections recorded as warnings), T-01-30 (sanitize strips C0, C1 and bidi controls, proptest over arbitrary strings plus a black-box ESC and U+202E test), T-01-31 (parser never panics, proptest over arbitrary bytes and diff-shaped text with overflowing line numbers; the capped diff arrives with plan 01-09).

## Next Phase Readiness

- Plan 01-09: replace the empty `HashSet::new()` in `runner.rs` with the previous-run finding IDs and feed the capped diff to `diff::parse` in `steps::prepare` (the DiffIndex is built from the same bytes `context::write` writes, so a truncated diff needs no extra work).
- Plan 01-12: owns the live checks that a real bug PR anchors to the right line and a real clean PR gives `no_findings`; plan 01-02 still waits on the owner and `tests/fixtures/live-prs.json` was not touched.
- Phase 4 publication must confirm GitHub accepts LEFT anchors on context lines and the new path for renamed files (research A8).

## Self-Check: PASSED

All created files exist and commits b04d6ac, 679a08e and 9edd707 are ancestors of HEAD (verified with `[ -f ]` and `git merge-base --is-ancestor` in the closing step).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
