---
phase: 03-queue
plan: 07
subsystem: database
tags: [sqlite, request-seen, debounce, tracking, rust]

requires:
  - phase: 03-queue
    provides: "Schema version 3 with targets.request_seen and targets.queued_head"
provides:
  - "Store::apply_poll updates tracking and request_seen in one IMMEDIATE transaction"
  - "Store::discard sets untracked and request_seen and drops only an unstarted automatic job"
  - "One debounced automatic review job per target, reset when the head changes"
  - "Store::schedule_after_run queues the head the runner stored with set_known_head"
affects: [03-queue, watch, poll]
actuals:
  tokens: 10190
  tasks: 2
  commits: 3
plan_head_before: 59f0a343ac3388e0bc10cd38697dbb22a14951a3
plan_head_after: 7956e632db6b72ba439a7f647c199422497360a0
commits: 3
tech-stack:
  added: []
  patterns:
    - "Poll tracking and job arming share one IMMEDIATE transaction and never write targets.head_sha"
    - "The waiting head is targets.queued_head, or the latest run head when queued_head is null"
    - "A running review or a queued review with a null next_attempt_at blocks a second job and is not rewritten"
key-files:
  created:
    - src/store/poll.rs
    - src/store/poll/tracking.rs
    - src/store/poll/known.rs
    - src/store/poll/schedule.rs
    - src/store/poll/schedule/rows.rs
    - src/store/poll/schedule/write.rs
  modified:
    - src/store.rs
key-decisions:
  - "queued_head stays on the target. The poll compares a new head to it, and falls back to the latest run head when it is null, so an in-flight retry is not reset"
  - "A different head updates that one queued automatic job: queued_head replaced, retry_count 0, next_attempt_at now plus debounce. The same head leaves the deadline alone"
  - "schedule_after_run reads the head set_known_head already stored. A null head, a running review, a null next_attempt_at, or a run for that head inserts nothing"
patterns-established:
  - "Store::apply_poll, Store::discard, and Store::schedule_after_run are the only queue transitions for tracking and debounce"
requirements-completed: [QUE-03, QUE-06, QUE-07]
coverage:
  - id: D1
    description: "A poll that no longer contains a tracked open PR sets request_seen to 0 and leaves tracking tracked. Closed or merged sets untracked and does not change request_seen. Skip and an omitted target are unchanged."
    requirement: QUE-03
    verification:
      - kind: unit
        ref: "src/store/poll/tests/seen.rs#absence_writes_only_the_column_for_that_variant"
        status: pass
    human_judgment: false
  - id: D2
    description: "enqueue_review leaves tracking and request_seen as they were, including a brand-new URL. A later poll that includes the PR while request_seen is 0 sets tracked and request_seen 1. A repeat request does not change tracking."
    requirement: QUE-06
    verification:
      - kind: unit
        ref: "src/store/poll/tests/enqueue.rs#enqueue_review_does_not_change_tracking_or_request_seen"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/seen.rs#a_new_request_tracks_and_a_repeat_does_not_change_tracking"
        status: pass
    human_judgment: false
  - id: D3
    description: "discard sets untracked and request_seen 1, inserting the target if needed, and deletes only a queued automatic review with retry_count 0 and no run. A still-visible request does not track it again. An absent open poll clears request_seen, and the next seen poll tracks it."
    requirement: QUE-07
    verification:
      - kind: unit
        ref: "src/store/poll/tests/discard.rs#discard_hides_a_request_until_it_leaves_and_returns"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/discard.rs#discard_deletes_only_an_unstarted_automatic_job"
        status: pass
    human_judgment: false
  - id: D4
    description: "A new head on a tracked ready target inserts one automatic job due at now plus debounce. The same head does not move the deadline or retry_count. A different head resets that one job, including a retry that already has a run. A null deadline, a running job, a draft, an untracked target, a missing head, or an existing run for that head does not insert. Absent open uses the same rule only while the target stays tracked. Neither path writes head_sha or head_checked_at."
    requirement: QUE-03
    verification:
      - kind: unit
        ref: "src/store/poll/tests/schedule.rs#a_new_head_is_armed_once_and_the_same_head_waits"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/schedule.rs#a_different_head_resets_the_waiting_job_and_its_budget"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/schedule.rs#the_same_head_keeps_a_retry_deadline"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/block.rs#an_immediate_or_running_job_blocks_another"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/block.rs#draft_untracked_missing_and_reviewed_heads_do_not_queue"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/absent.rs#absent_open_arms_only_a_target_that_stays_tracked"
        status: pass
    human_judgment: false
  - id: D5
    description: "After a successful automatic run, schedule_after_run reads targets.head_sha. When that head differs from the finished run and no run exists for it, one debounce job is inserted and the finished run stays succeeded. A matching retry keeps its deadline. A null next_attempt_at or a running review blocks the insert."
    requirement: QUE-03
    verification:
      - kind: unit
        ref: "src/store/poll/tests/follow.rs#a_head_stored_after_success_gets_one_follow_up_job"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/follow.rs#schedule_after_run_holds_resets_or_blocks"
        status: pass
      - kind: unit
        ref: "src/store/poll/tests/hold.rs#a_waiting_retry_for_the_stored_head_keeps_its_deadline"
        status: pass
    human_judgment: false
duration: 34min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 07: Request seen and debounce Summary

**Store transitions for request_seen, discard, and one debounced automatic job per head, including a follow-up after the runner stores a new head**

## Performance

- **Duration:** 34 min
- **Started:** 2026-11-09T10:46:49Z
- **Completed:** 2026-11-09T11:20:29Z
- **Tasks:** 2
- **Files modified:** 20

## Accomplishments

- `Store::apply_poll` inserts a newly seen PR as untracked with `request_seen` 0, then sets `tracked` and `request_seen` 1 only when `request_seen` was 0. A repeat request does not change tracking. `Absent::Open` clears `request_seen` only. `Absent::Closed` sets `untracked` and does not write `request_seen`. `Absent::Skip` and a target missing from both lists are unchanged. Unknown absent URLs are ignored.
- `Store::discard` inserts the target if needed, sets `untracked` and `request_seen` 1, and deletes a queued automatic review only when `retry_count` is 0 and no run points at that job. A running job, a manual job, a job that already has a run, and a retry with `retry_count` greater than 0 stay. `enqueue_review` does not change either column.
- A tracked, non-draft target with a head and no active review job gets one automatic job: `origin` automatic, `force` 0, `retry_count` 0, `next_attempt_at` = now + debounce, and `targets.queued_head` set to that head. The same head does not move the deadline. A different head on the one queued automatic job with a non-null `next_attempt_at` keeps that job, sets `queued_head`, sets `retry_count` to 0, and sets a new deadline, even when a run already exists. When `queued_head` is null, the latest run head is the head being waited on, so a same-head retry is not reset.
- A queued job with a null `next_attempt_at`, a non-automatic queued review, or a running review is left alone and blocks a second job. A draft, an untracked target, a missing head, or any run already stored for that head does not insert. `Absent::Open` with a ready head uses the same rule only if the target is still tracked after the tracking update.
- `Store::schedule_after_run` reads `targets.head_sha`. It no-ops when that column is null, a review is running, a queued job has a null `next_attempt_at`, or a run for that head already exists and no waiting automatic job needs a reset. When the stored head differs from the finished run and no queued job exists, it inserts one debounce job. The finished run stays succeeded. Neither path updates `head_sha` or `head_checked_at`.

## Task Commits

Each task was committed atomically:

1. **Task 1: request_seen, discard, and missed-target transitions** - `331b253` (feat)
2. **Task 2: Debounce jobs on next_attempt_at** - `05d1e99` (feat), `7956e63` (test)

## Files Created/Modified

- `src/store.rs` - Declares `poll`. Dead code is allowed outside tests until a later plan calls it.
- `src/store/poll.rs` - `SeenRequest`, `Absent`, `KnownTarget`, `known_targets`, `discard`, `apply_poll`, `schedule_after_run`
- `src/store/poll/tracking.rs` - Tracking updates and the discard delete
- `src/store/poll/known.rs` - `known_targets` query
- `src/store/poll/schedule.rs` - Eligibility for a seen row and for `Absent::Open`, plus `schedule_after_run`
- `src/store/poll/schedule/rows.rs` - Chooses the live review job and the waiting head
- `src/store/poll/schedule/write.rs` - Inserts or resets the one automatic job and writes `queued_head`
- `src/store/poll/tests/` - SQLite cases for tracking, discard, debounce, and the post-run follow-up

## Decisions Made

- The head a debounce job is waiting on is `targets.queued_head`. A retry created before that column was set has it null, so the comparison falls back to the latest run head. The same head leaves `next_attempt_at` and `retry_count` unchanged.
- Reset and insert write `queued_head` only. `head_sha` and `head_checked_at` stay the runner's columns. `schedule_after_run` uses the value `set_known_head` already committed.
- An immediate queued job, including debug enqueue, is any queued review with a null `next_attempt_at`. It is not updated and it blocks a second job. A running review is the same, and `queued_head` is not written for it.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split modules so the 100-line hook accepts each commit**
- **Found during:** Task 1 (request seen and discard) and Task 2 (debounce jobs)
- **Issue:** `hk` rejects a commit that adds more than 100 lines to one file outside `.planning/` and `Cargo.lock`. Tracking, scheduling, and the tests do not fit in the files named by the plan.
- **Fix:** Moved `known_targets` to `poll/known.rs`, job SQL to `schedule/rows.rs` and `schedule/write.rs`, and tests into `fixture`, `jobs`, `runs`, `heads`, `seen`, `discard`, `enqueue`, `schedule`, `block`, `absent`, `follow`, and `hold`.
- **Files modified:** those modules under `src/store/poll/`
- **Verification:** `git diff --cached --numstat` showed no added file over 100 lines, and the pre-commit `lint:large-files` step passed
- **Committed in:** `331b253`, `05d1e99`, and `7956e63`

**2. [Rule 3 - Blocking] Allow dead_code on the poll module outside tests**
- **Found during:** Task 1
- **Issue:** Clippy denies warnings. `watch` does not call `apply_poll`, `discard`, or `known_targets` yet, so the non-test binary reported the new API as unused.
- **Fix:** `#[cfg_attr(not(test), allow(dead_code))]` on `mod poll`, the same shape plan 03-02 used for search. The allows on `Gh::search_review_requested` were left in place.
- **Files modified:** `src/store.rs`
- **Verification:** `cargo clippy --locked --all-targets --all-features -- -D warnings` exited 0
- **Committed in:** `331b253`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both were required by the repo hooks. The store rules match the plan. No new crate, no SPEC edit, and no GitHub calls.

## Issues Encountered

`src/store/poll/tests/schedule.rs` was left untracked on `05d1e99`. The hook still compiled it from the working tree, and those three tests passed there. `7956e63` commits the file. Nothing else was missing.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- A later poll can pass seen and absent rows into `Store::apply_poll` with an explicit `now_ms` and `debounce_ms`. It must not write `targets.head_sha`.
- After a successful automatic review, call `Store::schedule_after_run` only once `set_known_head` has committed. If the post-agent `pr view` failed, the stored head is still the reviewed head and this function does not arm a follow-up; the next poll does.
- Remove the `dead_code` allow on `mod poll` when production calls these methods.

## Self-Check: PASSED

- FOUND: `331b253`
- FOUND: `05d1e99`
- FOUND: `7956e63`
- FOUND: `src/store/poll.rs`
- FOUND: `src/store/poll/tracking.rs`
- FOUND: `src/store/poll/schedule.rs`
- FOUND: `src/store/poll/tests/schedule.rs`
