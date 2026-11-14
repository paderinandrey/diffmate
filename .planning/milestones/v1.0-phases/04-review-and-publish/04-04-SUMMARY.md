---
phase: 04-review-and-publish
plan: 04
subsystem: tui
tags: [ratatui, tokio, sqlite, open, enqueue]
requires:
  - phase: 04-review-and-publish
    provides: "Draft decisions, one publication per draft, and Store::discard plus Store::enqueue_review"
provides:
  - "x asks before discard, Keep is highlighted, and Enter on Discard calls Store::discard"
  - "o opens the PR URL with one argv and draws a spawn error"
  - "n enqueues one manual review and explains when watch is not holding the lock"
affects: [04-review-and-publish, tui]
actuals:
  tokens: 4786
  tasks: 2
  commits: 2
plan_head_before: 31f49d5693382370c498e20e3c50519d11589c98
plan_head_after: 9f1a62a51e360b756739e72c0709f269ae386fb4
tech-stack:
  added: []
  patterns:
    - "Confirm highlight moves directionally. Left and Up select Keep. Right and Down select Discard."
    - "The open program defaults to open. Tests inject the executable. The only argument is PrRef::web_url."
    - "A re-run uses the process bound from AppConfig and calls Store::enqueue_review. tui does not run the job."
key-files:
  created:
    - src/tui/confirm.rs
    - src/tui/open.rs
    - src/tui/tests/actions.rs
  modified:
    - src/tui.rs
    - src/tui/model.rs
    - src/tui/keys.rs
    - src/tui/draw.rs
    - src/store/status/queue.rs
key-decisions:
  - "Keep is the highlighted confirm choice. Enter on Keep closes the modal and does not call Store::discard."
  - "Enter on Discard calls Store::discard and reloads queue rows so the target leaves the list."
  - "Opening a PR spawns the platform open program with one argument, PrRef::web_url. A spawn error is drawn."
  - "n calls Store::enqueue_review with JobOrigin::Manual and force false. The explanation is shown when the lock holder is not Role::Watch."
patterns-established:
  - "QueueRow carries PrRef. Discard and open use that value and do not parse the label."
  - "Scripted sessions set the lock path and the opener. Production show binds the runner lock and leaves the opener unset."
requirements-completed: [PUB-03]
coverage:
  - id: D1
    description: "x opens a confirm whose highlighted choice is Keep. Enter on Keep leaves tracking and request_seen unchanged. Enter on Discard sets tracking untracked and request_seen 1. Esc closes with no write. q during confirm quits and leaves a queued publication pending."
    requirement: PUB-03
    verification:
      - kind: unit
        ref: "src/tui/tests/actions.rs#x_discards_only_after_confirm_and_latches_request_seen"
        status: pass
      - kind: unit
        ref: "src/tui/tests/actions.rs#enter_on_keep_leaves_tracking_and_request_seen"
        status: pass
      - kind: unit
        ref: "src/tui/tests/actions.rs#q_during_confirm_quits_and_leaves_a_queued_publication"
        status: pass
    human_judgment: false
  - id: D2
    description: "o spawns the open program with exactly one argument, the PR URL. A missing program draws could not open the pull request and returns Ok."
    requirement: PUB-03
    verification:
      - kind: unit
        ref: "src/tui/tests/actions.rs#o_spawns_open_with_only_the_pr_url"
        status: pass
      - kind: unit
        ref: "src/tui/tests/actions.rs#o_draws_a_spawn_error_without_panicking"
        status: pass
    human_judgment: false
  - id: D3
    description: "n enqueues a manual review with force false, leaves tracking unchanged, and inserts no automatic follow-up. With no watch holder the queue shows the explanation. While watch holds the lock that sentence is absent and the job stays queued."
    requirement: PUB-03
    verification:
      - kind: unit
        ref: "src/tui/tests/actions.rs#n_without_watch_enqueues_a_manual_review_and_explains"
        status: pass
      - kind: unit
        ref: "src/tui/tests/actions.rs#n_while_watch_holds_the_lock"
        status: pass
    human_judgment: false
duration: 30min
completed: 2026-11-10
status: complete
---

# Phase 4 Plan 4: Review and publish Summary

**Queue and draft keys confirm a discard, open the PR with one argument, and enqueue a manual re-run without running it.**

## Performance

- **Duration:** 30 min
- **Started:** 2026-11-10T16:39:10Z
- **Completed:** 2026-11-10T17:09:08Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- `x` replaces the screen with a confirm. Keep is highlighted. Enter on Keep writes nothing. Enter on Discard calls the existing `Store::discard`, which latches `tracking=untracked` and `request_seen=1`, then reloads the queue.
- `o` spawns `open` (or the test program) with one argument, `PrRef::web_url`. A spawn failure is drawn as `could not open the pull request: …` and the key returns Ok.
- `n` calls `Store::enqueue_review` with `JobOrigin::Manual` and `force: false`. Tracking stays unchanged. No automatic follow-up is inserted. The TUI does not start the review. When the lock holder is not `Role::Watch`, the queue shows `diffmate review or diffmate watch must run it`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Confirm before discarding a target** - `9ae6aaf` (feat)
2. **Task 2: Open a PR and enqueue a re-run** - `9f1a62a` (feat)

**Plan metadata:** the docs commit that adds this summary

## Files Created/Modified

- `src/tui/confirm.rs` - prompt, Keep and Discard labels, directional highlight
- `src/tui/open.rs` - `open_pr` spawns the program with one URL argument and waits
- `src/tui/tests/actions.rs` - discard, open, and re-run scripted tests
- `src/tui/model.rs` - confirm, `open_pull`, `rerun`, runner binding
- `src/tui/keys.rs` - `x`, `o`, `n`, and confirm keys
- `src/tui/draw.rs` - confirm modal and the queue notice
- `src/tui.rs` - apply arms, production runner bind, scripted action seam
- `src/store/status/queue.rs` - `QueueRow.pr`
- `src/tui/tests/mod.rs` - `mod actions`

## Decisions Made

- Keep is the highlighted choice, drawn as `[Keep]`. Left and Up move toward Keep. Right and Down move toward Discard. Enter on Keep does not call the store.
- Accepting Discard calls the existing `Store::discard` and then reloads `queue_rows`. The latch is the store method's latch.
- The browser program is `open` unless a test sets another executable. Production does not set the opener. The argument vector is the URL alone.
- Re-run uses `ProcessInfo` from `AppConfig` in `show`, and the scripted session's process in tests. The explanation is set only when `lock::holder` is not `Role::Watch`.
- Publish keys `a`, `r`, and `c` stay unwired. Reconcile and abandon stay unwired. `q` leaves a queued publication in place.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Reload the queue after a successful discard**
- **Found during:** Task 1 (confirm before discarding a target)
- **Issue:** The plan said the accept path calls `Store::discard` and no other store method. Leaving the in-memory queue unchanged would keep showing a target the store has already latched as untracked.
- **Fix:** After `discard` returns Ok, `accept_confirm` reloads `queue_rows` through the existing status query.
- **Files modified:** `src/tui/model.rs`
- **Verification:** The discard test sees the target leave the queue and reads `tracking=untracked`, `request_seen=1`.
- **Committed in:** `9ae6aaf` (part of task 1)

**2. [Rule 3 - Blocking] Route keys, draw the notice, and carry PrRef on the queue row**
- **Found during:** Task 1 and Task 2
- **Issue:** The plan's file list omitted `src/tui.rs`, `src/tui/draw.rs`, and `src/store/status/queue.rs`. Those files own apply, drawing, and the queue row.
- **Fix:** Added the confirm and action arms, drew the confirm modal and the queue notice, and stored `PrRef` on `QueueRow` so discard and open do not parse the label.
- **Files modified:** `src/tui.rs`, `src/tui/draw.rs`, `src/store/status/queue.rs`
- **Verification:** The action tests drive `x`, `o`, and `n` through the scripted loop and read the store.
- **Committed in:** `9ae6aaf` and `9f1a62a`

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** Both keep the existing store methods and the existing queue query. No new crate, no schema change, no publish or reconcile keys.

## TDD Gate Compliance

Task 2 declared `tdd="true"`. The plan type stays `execute`, and config `tdd_mode` is false. Task 1 stayed a tracer.

RED evidence was classified `RED_EVIDENCE_OK` before implementation. The classifier reason was `target_test_failed`. nextest exited 100: 7 tests, 3 passed, 4 failed. The target test `tui::tests::actions::o_spawns_open_with_only_the_pr_url` expected the argv log `1` and `https://github.com/o/r/pull/1`. The log was empty because `o` was still ignored. The other three new tests failed on the planned assertions: job count 0 versus 1, a missing spawn-error sentence, and a missing explanation sentence.

A separate `test(04-04)` commit is missing. The hk pre-commit hook runs the full suite and rejects a commit whose tests fail. `--no-verify` is forbidden. The failing tests and the implementation landed together in `9f1a62a` (`feat(tui): open a PR and enqueue a re-run`). That is a missing RED commit. The hook was not bypassed.

GREEN is that same commit. The filtered action suite then passed 7 tests. The commit hook ran the full suite: 443 passed, 11 skipped. No refactor commit.

⚡ Tracer verified end-to-end — expanding. The task 1 discard tests passed in the task 1 hook and again inside the 7-test action run. `AUTO_CHAIN` and `AUTO_CFG` were false. `HUMAN_VERIFY_MODE` is `end-of-phase`, and the tracer verify is automated, so execution continued without a human checkpoint.

## Issues Encountered

The RED commit could not land while the new tests failed, because the pre-commit hook runs `mise run test`. The evidence report was classified, then deleted, and was not committed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 04-05. Publish keys `a`, `r`, and `c` are still unwired. Reconcile and abandon remain plan 04-07. `PUB-03` is also declared by 04-07, so this plan does not mark that requirement complete in REQUIREMENTS.md until 04-07 has a summary.

## Self-Check: PASSED

- FOUND: `src/tui/confirm.rs`
- FOUND: `src/tui/open.rs`
- FOUND: `src/tui/tests/actions.rs`
- FOUND: `src/tui.rs`
- FOUND: `src/tui/draw.rs`
- FOUND: `src/tui/keys.rs`
- FOUND: `src/tui/model.rs`
- FOUND: `src/store/status/queue.rs`
- FOUND: `src/tui/tests/mod.rs`
- FOUND commits: `9ae6aaf` and `9f1a62a` are ancestors of HEAD

## Verification

- `mise run --force test -- -E 'test(actions::)' --no-tests=fail` passed: 7 tests, 0 failed, before the task 2 commit.
- The task 2 pre-commit hook ran `mise run test`: 443 passed, 11 skipped.
- The same hook ran `cargo fmt --all --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, and `cargo machete`.
- `mise run coverage` and `mise run test:e2e` were not run.

---
*Phase: 04-review-and-publish*
*Completed: 2026-11-10*
