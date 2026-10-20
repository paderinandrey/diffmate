---
phase: 01-single-review
plan: 09
subsystem: context
tags: [rust, context-files, gh, graphql, check-runs, git-ls-tree, size-caps, previous-findings]

requires:
  - phase: 01-single-review
    provides: "01-04 tracer (context::write, runner split), 01-06 git hardening (Git trait, GitCli builder), 01-07 anchoring (DiffIndex, normalize), 01-08 runner lock"
provides:
  - "Context directory with pr.md, diff.patch, discussion.md, ci.md, previous.json and rules/{CLAUDE,AGENTS,CONTRIBUTING,REVIEW}.md, each stating when its data is unavailable or truncated"
  - "Fixed size caps with a first-line TRUNCATED marker; diff.patch cut on a hunk boundary and parsed by the runner exactly as written"
  - "previous.json from the latest successful earlier run of the same target, and its finding IDs fed to normalize"
  - "rules/ read at the merge-base through Git::tree_entry and Git::blob, symlinks resolved one hop"
  - "Read-only gh: Gh::discussion (one GraphQL query), Gh::check_runs and Gh::annotations (REST GETs)"
affects: [01-10, 01-11, 01-12]

actuals:
  tokens: 21400
  tasks: 3
  commits: 3

plan_head_before: b2802e226233d3d5b287dd28b4fa3b75c6d51611
plan_head_after: ef0899f6deec7bff88b411b91de4e7f99d41d1bd
commits: 3

tech-stack:
  added: []
  patterns:
    - "Context writers return text and never fail the run for gh problems: a gh failure is stated inside the file (Discussion unavailable, CI status unknown)"
    - "cap_split(name, text, total, cap) returns the marker line and the kept prefix; callers decide where the first-line marker goes (pr.md, rules with a symlink note)"
    - "Serde shapes for GraphQL live in gh/discussion/raw/shapes.rs and convert into plain public types; nodes are Option so a null node is skipped"
    - "Hand-written typed fakes for Gh in tests use a Ready<Result<..>> return with allow(refining_impl_trait), which keeps the fake short"

key-files:
  created:
    - src/context/caps.rs
    - src/context/files.rs
    - src/context/pr.rs
    - src/context/previous.rs
    - src/context/rules.rs
    - src/context/discussion.rs
    - src/context/ci.rs
    - src/gh/meta.rs
    - src/gh/discussion.rs
    - src/gh/discussion/raw.rs
    - src/gh/discussion/raw/shapes.rs
    - src/gh/checks.rs
    - src/git/build.rs
    - src/git/tree.rs
    - src/store/previous.rs
    - src/runner/resolve.rs
    - tests/context.rs
    - tests/context/helpers.rs
    - tests/context_rules.rs
  modified:
    - src/context.rs
    - src/gh.rs
    - src/git.rs
    - src/git/cli.rs
    - src/store.rs
    - src/runner.rs
    - src/runner/steps.rs
    - src/recovery/fake.rs

key-decisions:
  - "Tree sizes come from ls-tree --long and TreeEntry carries size, so a rules file longer than its cap states the real total in the TRUNCATED marker while blob reads at most the cap"
  - "Context warnings on the run are limited to truncation (any written file whose first line is TRUNCATED:); gh failures are not run warnings because the files already say so and existing outputs would otherwise change"
  - "graphql owner and repo go through -f (raw strings) and only the number through -F, because -F would turn a repository named 123 or true into a typed value"
  - "A symlink in the rules set is followed only when its target is one plain component ([A-Za-z0-9._-], not . or ..) that is a regular file at the merge-base; every other case is written as not followed, with the target escaped"
  - "A rules read error (git failure) becomes NAME is unavailable: REASON in the file instead of failing the review"

patterns-established:
  - "Run-wide context assembly: runner::steps::prepare builds ContextInputs and gets back Prepared { index, previous_ids, warnings }"

requirements-completed: [REV-02]

coverage:
  - id: D1
    description: "The context directory holds pr.md, diff.patch, discussion.md, ci.md, previous.json and the four rules files, with discussion and CI taken from gh and PR data from gh pr view"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/context.rs#context_files_hold_pr_data_discussion_and_ci"
        status: pass
    human_judgment: false
  - id: D2
    description: "Missing CI data and no check runs are written as CI status unknown, never as passing; at most 10 failed checks get annotations; a short list or a cut is marked TRUNCATED"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/context.rs#ci_unavailable_is_written_as_unknown"
        status: pass
      - kind: unit
        ref: "src/context/tests/ci_cases.rs and ci_unknown.rs (7 cases)"
        status: pass
    human_judgment: false
  - id: D3
    description: "rules/ is read from the merge-base with ls-tree and cat-file, a symlink resolves one hop inside the tree, absent, unsafe, directory and oversized entries are stated, and the PR cannot change its own rules"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/context_rules.rs#rules_are_read_at_merge_base"
        status: pass
      - kind: integration
        ref: "tests/context_rules.rs#unsafe_missing_and_oversized_rules_are_marked_not_followed"
        status: pass
      - kind: unit
        ref: "src/git/tree_tests.rs (3 cases)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Per-file caps with a first-line TRUNCATED marker; diff.patch is cut on a hunk boundary and the cut diff adds no anchor the full diff lacks"
    requirement: REV-02
    verification:
      - kind: property
        ref: "src/context/tests/cut_property.rs#the_kept_part_is_a_bounded_prefix_that_adds_no_anchor"
        status: pass
      - kind: unit
        ref: "src/context/tests/caps_cases.rs (6 cases)"
        status: pass
    human_judgment: false
  - id: D5
    description: "previous.json lists the latest successful earlier run's findings with decision and thread status, and a previous_finding_id naming one of them is stored without a warning"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/context.rs#previous_json_lists_earlier_findings"
        status: pass
      - kind: unit
        ref: "src/context/tests/previous_shapes.rs"
        status: pass
    human_judgment: false
  - id: D6
    description: "All GitHub access in the review is read-only: pr view, one GraphQL query without a mutation, and REST GETs without -X, --method, -f or -F"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/context.rs#gh_calls_are_read_only"
        status: pass
    human_judgment: false
  - id: D7
    description: "The GraphQL query, the check-runs and annotations endpoints and the response shapes work against the real GitHub API"
    requirement: REV-02
    verification: []
    human_judgment: true
    rationale: "Every test uses a fake gh with hand-written responses; the real query and REST shapes are exercised only by the live checks in plan 01-12"

duration: 17min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 09: Context files Summary

**Size-capped pr.md and diff.patch (hunk-boundary cut), discussion.md and ci.md from read-only gh calls with "CI status unknown" never read as passing, previous.json from the latest successful run feeding normalize, and rules/ read at the merge-base with one-hop symlinks**

## Performance

- **Duration:** 17 min (2026-10-20T19:11Z to 19:28Z)
- **Tasks:** 3
- **Files:** 38 changed in the plan range (30 created, 8 modified), 2024 insertions and 216 deletions

## Accomplishments

- `context::write` is an orchestrator over submodules. It writes pr.md (description through `cap_split` with 64 KiB, marker as the file's first line), diff.patch (`cap_diff`, 1 MiB, cut before the last line that starts with `diff --git ` or `@@ ` within the cap), previous.json, rules/, discussion.md and ci.md, and returns the diff exactly as written, the previous finding IDs and truncation warnings. The runner parses that diff and passes the IDs to `normalize`, so a `previous_finding_id` from previous.json is kept and an unknown one is nulled.
- previous.json is `{"available": false, "reason": "no earlier successful run of this target"}` for a first run, otherwise run_id, head_sha and findings with id, the original finding, decision and thread_status (`unknown` for anchored findings, `none` for body findings). `Store::latest_succeeded_before` reads the newest succeeded run of the target below the current run id.
- rules/ comes from the merge-base: `Git::tree_entry` (`ls-tree --long`) and `Git::blob` (`cat-file blob` with a bounded capture). A regular file gets its content, a root-level symlink to a plain single-component regular file gets a first line naming the link and the target followed by the target's content, and absent, unsafe-symlink, second-hop, directory and submodule entries are stated in the file. A PR that rewrites AGENTS.md cannot change what the agent reads (the test fails if the head SHA is read instead).
- discussion.md and ci.md come from one GraphQL query (comments, reviews, review threads, each with `hasNextPage`) and REST GETs (check runs for the head SHA, annotations for at most 10 failed checks). A gh failure writes `Discussion unavailable: REASON` or `CI status unknown: REASON` (bounded to 300 characters, control characters replaced); zero check runs writes `No check runs were reported for SHA. CI status unknown.`; no line ever says the checks passed.

## Task Commits

1. **Task 1: size caps, the hunk-boundary diff cut and previous.json, wired into the runner** - `907704b` (feat)
2. **Task 2: rules/ read at the merge-base** - `f6a348d` (feat)
3. **Task 3: discussion.md and ci.md from read-only gh calls** - `ef0899f` (feat)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Decisions Made

- See `key-decisions` in the frontmatter.
- Flagged for the owner (plan "Flagged assumptions"): the cap values (pr.md body 64 KiB, diff.patch 1 MiB, discussion.md 256 KiB, ci.md 128 KiB, each rules file 128 KiB) are planner choices under CONTEXT.md Claude's Discretion and are constants in `src/context/caps.rs`.
- A truncated file adds a run warning (`FILE was truncated; the agent saw only part of it`), which prints on stderr and is stored with the run. That includes a discussion.md whose only truncation is GitHub having more pages than the query asks for.

## Verification

Observed in this session:

- `mise run --force test -- -E 'test(context::) | binary(review_output) | binary(review)'` (Task 1): 16 passed.
- `mise run --force test -- -E 'binary(context_rules) | test(git::)'` (Task 2): 14 passed.
- `mise run --force test -- -E 'binary(context) | binary(context_rules) | test(context::)'` (Task 3 filter, run as part of the whole suite).
- `mise run --force test`: 141 run, 141 passed, 3 skipped (the live probes).
- `mise run --force lint`: exit 0 (rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks, actionlint).
- `mise run coverage`: exit 0, 91.37% line coverage (gate 80%).
- Mutation checks, each observed and reverted: dropping the `diff --git `/`@@ ` condition from the boundary search fails the cut property test and the boundary unit test; reading rules at the head SHA instead of the merge-base fails `rules_are_read_at_merge_base` (`pr rules` instead of `base rules`).
- Acceptance greps: `DIFF_CAP: usize = 1_048_576` in caps.rs = 1; `previous_ids` in runner.rs = 1; `tree_entry` in git.rs = 1; `not present at the merge-base` in rules.rs = 1; `CI status unknown` in ci.rs = 2; `mutation` in gh/discussion.rs = 0; every file the plan touches is at most 100 lines (largest: tests/context.rs 96, src/recovery/fake.rs 95, src/runner.rs 96).
- The hk pre-commit hook (lint, large-files, secrets, docs, test, actions) passed on all three commits without `--no-verify`.
- Not run: `mise run vuln` and `mise run deps:check` (no crate was added), `mise run test:e2e`, `mise run docs:check` outside the hook. The real GitHub GraphQL and REST responses were not exercised (see coverage D7).

## Deviations from Plan

**1. [Rule 3 - Blocking] Extra files for the 100-line commit hook and acceptance criteria**
- **Issue:** the planned files do not fit under 100 lines once rustfmt has run.
- **Fix:** added `src/context/{files,pr}.rs` (file writer and pr.md), `src/gh/meta.rs` (PrMeta moved out of gh.rs), `src/gh/discussion/raw.rs` and `raw/shapes.rs` (serde shapes), `src/git/build.rs` (GitCli, its builder and `os` moved out of git.rs), `src/store/previous.rs` (instead of growing runs.rs), `src/runner/resolve.rs` (the resolve step moved out of steps.rs), `src/context/tests/*` (eight test modules), `tests/context/helpers.rs` (loaded with `#[path]`). `TreeEntry` is defined in `src/git/tree.rs` and re-exported from `crate::git`. Public paths named in the plan interfaces hold: `crate::context::{ContextInputs, WrittenContext, write}`, `crate::git::{Git, TreeEntry}`, `crate::gh::{Gh, Discussion, CheckRuns, Annotation}`, `Store::latest_succeeded_before`.
- **Committed in:** 907704b, f6a348d, ef0899f

**2. [Rule 3 - Blocking] ContextInputs grew one task at a time**
- **Issue:** clippy denies unread struct fields, so `repo`, `git` and `gh` could not exist before their consumers.
- **Fix:** `ContextInputs<'a>` in Task 1, `<'a, G>` in Task 2, `<'a, G, H>` in Task 3; the final shape equals the interfaces block. `cap_text` carried `cfg_attr(not(test), expect(dead_code))` for Task 1 and 2 and lost it in Task 3 when discussion.md started using it.
- **Committed in:** 907704b, f6a348d, ef0899f

**3. [Rule 1 - Bug] The plan said there are no other Git implementations**
- **Issue:** `src/recovery/fake.rs` implements `Git`, so the two new trait methods broke its compile.
- **Fix:** added `tree_entry` and `blob` to the fake, returning `Ready` with `#[allow(refining_impl_trait)]` to stay under 100 lines.
- **Committed in:** f6a348d

**4. [Rule 2 - Missing critical] -f for graphql owner and name, size on TreeEntry, hex check on the SHA**
- **Issue:** the plan passes `-F owner=... -F name=...`; `-F` converts values (`true`, `123`, `@file`) and a repository with such a name would break or read a file. The plan's `TreeEntry { mode, oid }` cannot give the real total of an oversized rules file for the marker. The head SHA goes into a REST path.
- **Fix:** owner and name use `-f`, only the number uses `-F`; `TreeEntry` gained `size` (from `ls-tree --long`); `check_runs` rejects a non-hexadecimal SHA before building the path.
- **Committed in:** ef0899f (graphql, SHA), f6a348d (size)

**5. [Rule 2 - Missing critical] Symlink targets are escaped; read errors become file text**
- **Issue:** a non-followed symlink target is attacker-controlled text written into a rules file, and a git error while reading one rules file would fail the whole review.
- **Fix:** the target is written with `escape_debug`; an error becomes `NAME is unavailable: REASON` (bounded, control characters replaced).
- **Committed in:** f6a348d

**6. [Rule 1 - Design] Added unit tests for discussion and CI rendering and a fifth rules test**
- **Fix:** `src/context/tests/{discussion_cases,discussion_limits,ci_cases,ci_unknown,fake_gh}.rs`, `src/gh/discussion/tests.rs`, `src/git/tree_tests.rs` and `unsafe_missing_and_oversized_rules_are_marked_not_followed` cover the cases the four plan tests do not (more than 10 failed checks, short list, page limits, null authors and nodes, second-hop and out-of-tree symlinks, oversized rules file). The no-check-runs integration test was dropped to keep tests/context.rs at 96 lines; the unit test `no_check_runs_and_a_failed_call_both_read_as_unknown` covers it.

**Total deviations:** 6 (3 blocking, 1 bug, 2 missing-critical; the sixth is added coverage). **Impact:** none on behavior beyond the SPEC; interfaces keep their names.

## Issues Encountered

- `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were already modified or untracked and were left out of every commit.
- `ReviewResult` still carries `#[expect(dead_code)]` for `run_id`; this plan does not read it either.

## User Setup Required

None.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-37 (rules read with ls-tree and cat-file at the merge-base; `rules_are_read_at_merge_base` fails if the head SHA is used), T-01-38 (symlinks resolved one hop to a plain root-level regular file; `..`, absolute, nested and second-hop targets are not followed), T-01-39 (caps with TRUNCATED markers, bounded blob capture, at most 10 annotation calls), T-01-40 (unknown CI data is "CI status unknown", no pass line exists, asserted in the unit and integration tests), T-01-41 (`gh_calls_are_read_only` over the logged argv), T-01-42 (accepted: PR text is written to files only, never into argv).

## Next Phase Readiness

- Plan 01-10 (signals) can build on `runner::steps::prepare` returning `Prepared`; nothing in this plan touched the lock, recovery or `src/review.rs`.
- Plan 01-12 should exercise the real GraphQL query and the check-runs and annotations endpoints once (coverage D7); the fake gh answers hand-written JSON only.
- Plan 01-02 still waits on the owner; `tests/fixtures/live-prs.json` was not touched.

## Self-Check: PASSED

All 19 created files listed in key-files exist and commits 907704b, f6a348d and ef0899f are ancestors of HEAD (verified with `[ -f ]` and `git merge-base --is-ancestor` in the closing step).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
