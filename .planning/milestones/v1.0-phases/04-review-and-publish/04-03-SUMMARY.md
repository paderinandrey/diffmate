---
phase: 04-review-and-publish
plan: 03
subsystem: publish
tags: [gh, sqlite, rusqlite, review, snapshot]
requires:
  - phase: 04-review-and-publish
    provides: "Schema user_version 5, drafts with limitation_included, and finding decisions"
provides:
  - "confirm_publication inserts one pending publication and one queued publish job, or neither"
  - "execute_publish commits sending, posts one review from the confirm snapshot, then stores submitted"
  - "A second confirm while pending, sending, or unknown creates nothing, and a submitted draft stays submitted"
affects: [04-review-and-publish, tui, reconcile]
actuals:
  tokens: 12285
  tasks: 3
  commits: 3
plan_head_before: 0183dc7c2fa899fff5aeaaa9d2b308280f7c1d38
plan_head_after: 32de3c8f87406b2375041d50cd97a754b2da18f7
tech-stack:
  added: []
  patterns:
    - "create_review posts JSON on stdin to gh api. The path is built only from PrRef.owner, PrRef.repo, and PrRef.number."
    - "The review body and inline comments are read from the snapshot stored in the confirm transaction."
    - "Publish tests call confirm_publication and execute_publish with a typed Gh fake. create_review is not a live POST."
key-files:
  created:
    - src/publish/post.rs
    - src/publish/body.rs
    - src/publish/checks.rs
    - src/gh/reviews.rs
    - src/store/publish.rs
  modified:
    - src/gh.rs
    - src/proc.rs
    - src/main.rs
    - src/store.rs
key-decisions:
  - "One confirm writes the snapshot, the pending publication, and the queued publish job in one Immediate transaction."
  - "execute_publish commits sending before create_review. The wire event, body, and inline comments come from that snapshot."
  - "A second confirm returns Blocked while any publication for the target is pending, sending, or unknown. A submitted draft returns DraftSubmitted. Rejected and abandoned do not block."
  - "A unique conflict on jobs.publication_id returns Blocked and rolls the new confirm back."
patterns-established:
  - "Inline comments end with the finding marker. The publication marker is the last line of the body."
  - "Unanchored kept and edited findings, an included limitation, and future improvements are body sections. Dropped findings are absent."
requirements-completed: [PUB-04, PUB-05, PUB-06]
coverage:
  - id: D1
    description: "One kept anchored finding becomes one inline comment on the run head SHA. sending is stored before create_review. Success stores the GitHub review id and submitted. A forced job-insert failure leaves no publication and no publish job."
    requirement: PUB-04
    verification:
      - kind: unit
        ref: "src/publish/tests/mod.rs#one_kept_finding_reaches_submitted"
        status: pass
      - kind: unit
        ref: "src/publish/tests/mod.rs#confirm_rolls_back_when_the_job_insert_fails"
        status: pass
    human_judgment: false
  - id: D2
    description: "Anchored kept and edited findings are inline only. Unanchored kept and edited findings, an included limitation, and future improvements are in the body. Dropped findings and an excluded limitation are absent. APPROVE and REQUEST_CHANGES use the run head."
    requirement: PUB-04
    verification:
      - kind: unit
        ref: "src/publish/tests/body.rs#body_omits_dropped_and_excluded_limitation"
        status: pass
      - kind: unit
        ref: "src/publish/tests/body.rs#body_includes_limitation_before_improvements"
        status: pass
      - kind: unit
        ref: "src/publish/tests/gates.rs#approve_and_request_changes_use_the_run_head"
        status: pass
    human_judgment: false
  - id: D3
    description: "Undecided findings return ConfirmError::Undecided in display order and insert nothing. A draft with no findings confirms, and its body is the publication marker only."
    requirement: PUB-05
    verification:
      - kind: unit
        ref: "src/publish/tests/gates.rs#pre_checks_create_nothing"
        status: pass
      - kind: unit
        ref: "src/publish/tests/gates.rs#empty_draft_confirms"
        status: pass
    human_judgment: false
  - id: D4
    description: "A second confirm while a publication is pending, sending, or unknown creates nothing. A submitted draft is refused after a later text edit. Rejected and abandoned allow another confirm. The stored snapshot and the posted payload ignore that later edit."
    requirement: PUB-06
    verification:
      - kind: unit
        ref: "src/publish/tests/once.rs#second_confirm_creates_nothing"
        status: pass
      - kind: unit
        ref: "src/publish/tests/once.rs#submitted_draft_is_refused"
        status: pass
      - kind: unit
        ref: "src/publish/tests/once.rs#rejected_allows_another_confirm"
        status: pass
      - kind: unit
        ref: "src/publish/tests/snap.rs#snapshot_ignores_later_edits"
        status: pass
      - kind: unit
        ref: "src/publish/tests/snap.rs#duplicate_publish_job_is_blocked"
        status: pass
    human_judgment: false
duration: 36min
completed: 2026-11-10
status: complete
---

# Phase 4 Plan 3: Review and publish Summary

**One decided draft posts one GitHub review from its confirm snapshot, and a second confirm creates nothing.**

## Performance

- **Duration:** 36 min
- **Started:** 2026-11-10T15:59:13Z
- **Completed:** 2026-11-10T16:35:00Z
- **Tasks:** 3
- **Files modified:** 24

## Accomplishments

- `confirm_publication` inserts a pending publication and a queued publish job in one Immediate transaction, or it inserts neither.
- `execute_publish` commits `sending` before `create_review`. The event, body, and inline comments come from the snapshot taken at confirm. Success stores the GitHub review id and `submitted`.
- Undecided findings, a submitted draft, and a target that already has a pending, sending, or unknown publication create no new row.

## Task Commits

Each task was committed atomically:

1. **Task 1: One comment review reaches submitted** - `240b09f` (feat)
2. **Task 2: Body shape, events, and a refused undecided draft** - `60ae802` (feat)
3. **Task 3: A second confirm and a submitted draft create nothing** - `32de3c8` (feat)

**Plan metadata:** the docs commit that adds this summary

## Files Created/Modified

- `src/store/publish.rs` - `confirm_publication` and `ConfirmError`
- `src/store/publish/insert.rs` - Immediate transaction and the submitted, blocking, and undecided gates
- `src/store/publish/save.rs` - snapshot JSON, publication id, publish job, and the unique-index mapping
- `src/publish/post.rs` - `sending`, then one `create_review`, then `submitted`
- `src/publish/body.rs` - body sections and inline comment text
- `src/publish/checks.rs` - snapshot payload: inline findings, body findings, limitation, and future improvements
- `src/gh/reviews.rs` - `create_review`, `list_my_reviews`, and `current_login`
- `src/proc.rs` - `run_tool_with_stdin` for the review JSON
- `src/publish/tests/` - scripted Gh fake and the publication cases

## Decisions Made

- The publication id is `lower(hex(randomblob(16)))`. The finding marker is only on an inline comment. The publication marker is the last part of the body.
- The body order is unanchored kept and edited findings, then `Limitation: ` plus the sentence when `limitation_included` is 1, then a `Future improvements` heading, then the publication marker.
- `commit_id` is the run head SHA. `confirmed_sha` is stored on the publication and is not sent as `commit_id`.
- `ConfirmError::Database` keeps a sqlite failure and its cause. The planned refusals stay `Undecided`, `DraftSubmitted`, and `Blocked`.
- New `Gh` methods default to a command error so the existing fakes still compile. `GhCli` overrides them.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing error cause] Sqlite failures from confirm keep their cause**
- **Found during:** Task 1 (One comment review reaches submitted)
- **Issue:** The planned `ConfirmError` variants were only `Undecided`, `DraftSubmitted`, and `Blocked`. A sqlite failure inside the confirm transaction would have had nowhere to go that still carried the cause.
- **Fix:** Added `ConfirmError::Database` as a transparent wrapper of the store error.
- **Files modified:** `src/store/publish.rs`
- **Verification:** `confirm_rolls_back_when_the_job_insert_fails` returns an error and leaves zero publications and zero publish jobs. Clippy `-D warnings` passed.
- **Committed in:** `240b09f` (part of task commit)

**2. [Rule 3 - Blocking lint] Publish symbols are allowed to be unused by `main` until a later plan calls them**
- **Found during:** Task 1 (One comment review reaches submitted)
- **Issue:** Clippy denies warnings, and `main` does not call `execute_publish` or the new `Gh` methods yet. Plan 04-05 owns the TUI keys.
- **Fix:** `#[cfg_attr(not(test), expect(dead_code))]` on the publish modules and the new trait defaults. A test calls the defaults so the test build still lints them.
- **Files modified:** `src/publish.rs`, `src/store/publish.rs`, `src/gh.rs`, `src/gh/reviews.rs`, `src/proc.rs`
- **Verification:** `cargo clippy --locked --all-targets --all-features -- -D warnings` passed in the task 1 hook.
- **Committed in:** `240b09f` (part of task commit)

---

**Total deviations:** 2 auto-fixed (1 Rule 2, 1 Rule 3)
**Impact on plan:** Both keep the confirm error and the warning-free build correct. No new crate, no schema bump, and no TUI key.

## TDD Gate Compliance

Tasks 2 and 3 were written test-first. `workflow.tdd_mode` is false, and the plan type is `execute`.

| Task | RED observed | GREEN commit |
|------|----------------|--------------|
| 2 | `body_omits_dropped_and_excluded_limitation` failed because the body was only the publication marker. `body_includes_limitation_before_improvements` failed because `Limitation: missing CI` was absent. | `60ae802` |
| 3 | `snapshot_ignores_later_edits` failed because the inline comment contained the edited live text. `duplicate_publish_job_is_blocked` failed with `UNIQUE constraint failed: jobs.publication_id` instead of `Blocked`. | `32de3c8` |

Separate `test(04-03)` commits were not created. The pre-commit hook runs the full nextest suite, and `--no-verify` is forbidden, so a commit of failing tests cannot land. The failing assertions were observed, then the implementation and the tests were committed together. The gate commits `test(04-03):` and `feat(04-03):` are absent. The feature commits use the subjects the plan states.

Semantic assessment: both RED runs compiled, executed the named tests, and failed on the planned assertions. The other new tests that already matched task 1 behavior passed in the same runs.

## Issues Encountered

The 100-line insertion hook required the new tests to live in sibling modules under `src/publish/tests/` (`body`, `flow`, `gates`, `once`, `snap`) instead of one file. `mod decisions` in `src/store.rs` was left in place.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plans 04-04 and 04-05 can call `confirm_publication` from the TUI. Keys `a`, `r`, and `c` are still unwired. Head comparison, a pending review of mine, and reconcile stay in later plans. Schema `user_version` is still 5.

## Self-Check: PASSED

- FOUND: `src/publish/post.rs`, `src/publish/body.rs`, `src/publish/checks.rs`, `src/gh/reviews.rs`, `src/store/publish.rs`
- FOUND commits: `240b09f`, `60ae802`, `32de3c8` are ancestors of HEAD

## Verification

- `mise run --force test -- -E 'test(publish::)' --no-tests=fail` passed: 13 tests, 0 failed, before the task 3 commit.
- The task 3 pre-commit hook ran `mise run test`: 436 passed, 11 skipped.
- The same hook ran `cargo fmt --all --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, and `cargo machete`.
- `mise run coverage` and `mise run test:e2e` were not run.

---
*Phase: 04-review-and-publish*
*Completed: 2026-11-10*
