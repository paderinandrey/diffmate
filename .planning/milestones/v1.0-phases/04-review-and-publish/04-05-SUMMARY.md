---
phase: 04-review-and-publish
plan: 05
subsystem: tui
tags: [ratatui, tokio, sqlite, gh, publish]
requires:
  - phase: 04-review-and-publish
    provides: "One publication per draft, confirm_publication, execute_publish, and the queue confirm modal"
provides:
  - "a, r, and c create one publication on a current decided draft and the tui runner posts it"
  - "A second publish key and an undecided or submitted draft create nothing further"
  - "u can abandon a queued pending publication after a confirm; q leaves it queued"
affects: [04-review-and-publish, tui, publish]
actuals:
  tokens: 8802
  tasks: 3
  commits: 3
plan_head_before: 32d67a674eb1326671229a575d9b9bd9e3b8b634
plan_head_after: 72c2e7a37c99150fccf00b2e09f5c3d533cd2a1f
tech-stack:
  added: []
  patterns:
    - "Publish keys go through post::send. Undecided findings are refused before pr_view. A head that differs from the run head returns without confirm_publication."
    - "drive acquires runner.lock, announces Role::Tui, runs recovery::run, then serve. serve takes only queued publish jobs."
    - "Confirm carries a purpose. Discard still calls Store::discard. Cancel calls Store::cancel_queued."
key-files:
  created:
    - src/tui/post.rs
    - src/tui/runner.rs
    - src/tui/tests/fake.rs
    - src/tui/tests/refusals.rs
    - src/tui/tests/events.rs
    - src/store/publish/cancel.rs
    - src/tui/tests/cancel.rs
  modified:
    - src/tui.rs
    - src/tui/keys.rs
    - src/tui/model.rs
    - src/tui/draw.rs
    - src/tui/confirm.rs
    - src/store/publish.rs
    - src/publish.rs
    - src/gh.rs
    - src/proc.rs
key-decisions:
  - "a sends APPROVE, r sends REQUEST_CHANGES, and c sends COMMENT. A head that differs from the run head returns without a stale question. Stale confirmation stays in 04-06."
  - "tui serves publish jobs only. The next reconcile or sync job ends the loop. A queued review job stays queued. The queue shows runner: tui while this process holds the lock."
  - "A second publish key while a publication is pending is Blocked. A submitted draft draws this draft is already submitted and posts nothing further."
  - "u asks first. Keep queued is highlighted. The other choice abandons the publication with reason cancelled by me and cancels the queued job. A started publication stays as it is. q leaves it pending."
patterns-established:
  - "The session holds RunnerLock so a later queue frame still says runner: tui."
  - "cancel_queued uses one Immediate transaction. A row that is no longer pending and queued is left unchanged."
requirements-completed: [PUB-04, PUB-05, PUB-06, PUB-13]
coverage:
  - id: D1
    description: "c on a current draft whose finding is kept creates one submitted publication and one create_review. The seeded review job stays queued. The queue frame contains runner: tui."
    requirement: PUB-04
    verification:
      - kind: unit
        ref: "src/tui/tests/publish.rs#comment_key_posts_one_review"
        status: pass
    human_judgment: false
  - id: D2
    description: "An undecided finding blocks c until it is kept. A second c while the publication is pending posts once. After submitted, c draws this draft is already submitted and posts nothing further."
    requirement: PUB-06
    verification:
      - kind: unit
        ref: "src/tui/tests/refusals.rs#undecided_blocks_until_that_finding_is_kept"
        status: pass
      - kind: unit
        ref: "src/tui/tests/refusals.rs#second_c_while_pending_posts_once"
        status: pass
      - kind: unit
        ref: "src/tui/tests/refusals.rs#submitted_draft_does_not_post_again"
        status: pass
    human_judgment: false
  - id: D3
    description: "a and r set APPROVE and REQUEST_CHANGES. A draft with no findings publishes a body that is only the publication marker."
    requirement: PUB-04
    verification:
      - kind: unit
        ref: "src/tui/tests/events.rs#approve_and_request_changes_set_the_event"
        status: pass
      - kind: unit
        ref: "src/tui/tests/events.rs#empty_draft_body_is_only_the_marker"
        status: pass
    human_judgment: false
  - id: D4
    description: "While this process holds runner.lock the queue shows runner: tui and the seeded review job stays queued."
    requirement: PUB-13
    verification:
      - kind: unit
        ref: "src/tui/tests/publish.rs#comment_key_posts_one_review"
        status: pass
    human_judgment: false
  - id: D5
    description: "u opens a confirm whose highlighted choice is Keep queued and Enter writes nothing. The other choice sets the publication to abandoned with reason cancelled by me and the job to cancelled. A running job or a sending publication stays unchanged and the frame says it already started. q leaves the publication pending and the job queued."
    verification:
      - kind: unit
        ref: "src/tui/tests/cancel.rs#keep_queued_is_highlighted_and_enter_writes_nothing"
        status: pass
      - kind: unit
        ref: "src/tui/tests/cancel.rs#the_other_choice_cancels_the_queued_publication"
        status: pass
      - kind: unit
        ref: "src/tui/tests/cancel.rs#started_publication_is_not_cancelled"
        status: pass
      - kind: unit
        ref: "src/tui/tests/cancel.rs#quit_leaves_a_queued_publication_pending"
        status: pass
    human_judgment: false
duration: 38min
completed: 2026-11-10
status: complete
---

# Phase 4 Plan 5: Review and publish Summary

**Publish keys post one review from a decided current draft, and u can abandon a publication that has not started.**

## Performance

- **Duration:** 38 min
- **Started:** 2026-11-10T17:12:24Z
- **Completed:** 2026-11-10T17:50:52Z
- **Tasks:** 3
- **Files modified:** 18

## Accomplishments

- On a current draft whose findings are decided, `a`, `r`, and `c` create one publication and enqueue the publish job. `a` sends `APPROVE`, `r` sends `REQUEST_CHANGES`, and `c` sends `COMMENT`. The scripted `gh` fake records one `create_review`. `sending` is committed inside the existing `execute_publish` before that call.
- An undecided finding draws `undecided findings:` plus the ids and inserts nothing. A second publish key while a publication is pending, sending, or unknown draws the blocked sentence and leaves the first publication. A submitted draft draws `this draft is already submitted`.
- While this process holds `runner.lock`, the queue shows `runner: tui`. `serve` starts and posts queued publish jobs. A queued review job stays queued. `u` asks before cancelling a queued pending publication. The highlighted choice keeps it queued. The other choice calls `Store::cancel_queued` in one transaction.

## Task Commits

Each task was committed atomically:

1. **Task 1: c posts one comment review and leaves review jobs queued** - `422bc41` (feat)
2. **Task 2: Refusals, the other events, and a double key press** - `203b5d7` (feat)
3. **Task 3: Cancel a queued publication, or leave it** - `72c2e7a` (feat)

**Plan metadata:** the docs commit that adds this summary

## Files Created/Modified

- `src/tui/post.rs` - pre-checks, `confirm_publication`, and the refusal sentences
- `src/tui/runner.rs` - lock, `Role::Tui`, recovery, and the publish-only serve loop
- `src/tui/tests/fake.rs` - scripted `Gh` that records `ReviewRequest` and returns review id 9001
- `src/tui/tests/publish.rs` - the tracer: one comment review, review job still queued, `runner: tui`
- `src/tui/tests/refusals.rs` - undecided, second press, and submitted draft
- `src/tui/tests/events.rs` - `APPROVE`, `REQUEST_CHANGES`, and an empty body
- `src/store/publish/cancel.rs` - read-only gate and the cancel transaction
- `src/tui/tests/cancel.rs` - keep, cancel, already started, and quit
- `src/tui.rs` - `a`, `r`, `c`, and `u` in `apply`, plus the scripted publish seam
- `src/tui/keys.rs` - `Approve`, `Request`, `Comment`, and `Unpublish`
- `src/tui/model.rs` - intent, runner lock, and the cancel confirm path
- `src/tui/draw.rs` - `runner: tui` on the queue frame
- `src/tui/confirm.rs` - discard and cancel purposes, with separate labels
- `src/store/publish.rs` - `publish_gate` and `cancel_queued`
- `src/publish.rs`, `src/gh.rs`, `src/proc.rs` - drop `expect(dead_code)` now that tui calls those paths

## Decisions Made

- A fetched head that differs from the run head returns before `confirm_publication`. This plan does not ask the stale-head question. That confirmation is 04-06. `Store::set_known_head` still records the fetched sha.
- `serve` selects jobs whose kind is `publish`, `reconcile`, or `sync`. A non-publish row ends the loop so the process does not spin and does not call `execute_review_job` or `next_runnable_job`. Reconcile and sync execution stay in 04-07 and 04-08.
- If `try_acquire` misses, the publication stays pending and the frame does not claim to be the runner. The session stores the lock so a later queue draw still shows `runner: tui`.
- `u` is global on the queue and the draft, and it does not replace `a`, `r`, `c`, `k`, `d`, `e`, or the D-07 keys. During a confirm, `u` is ignored. The safe label is `Keep queued`. The danger label is `Cancel publication`.
- A publication that is `sending`, or a publish job that is `running`, draws `it already started` and writes nothing. `q` from the queue quits. SIGINT and SIGTERM are unchanged and do not cancel a confirmed publication.
- The double-press test holds `runner.lock` outside the session so the first `c` confirms and the second `c` hits `Blocked`. The test then drops the lock and calls `serve` once.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split new files so each stays under the 100-insertion hook**
- **Found during:** Task 1, Task 2, and Task 3
- **Issue:** hk rejects a staged file with more than 100 insertions. One module for keys plus serve, and one test module for the tracer, the refusals, and cancel, would each exceed that cap.
- **Fix:** `src/tui/post.rs` owns the key path and `src/tui/runner.rs` owns serve. Tests are split across `fake.rs`, `publish.rs`, `refusals.rs`, `events.rs`, and `cancel.rs`. The cancel transaction lives in `src/store/publish/cancel.rs`.
- **Files modified:** the files named above
- **Verification:** `git diff --cached --numstat` stayed at or under 100 insertions on every staged source file. The large-files hook passed on each commit.
- **Committed in:** `422bc41`, `203b5d7`, and `72c2e7a`

**2. [Rule 1 - Bug] Dropped `expect(dead_code)` that production tui now reaches**
- **Found during:** Task 1
- **Issue:** Clippy denied unfulfilled `expect(dead_code)` on `Gh::create_review`, `publication_unavailable`, `unsupported`, `run_tool_with_stdin`, and the `publish` module export.
- **Fix:** Removed those expects. `current_login` and `list_my_reviews` still expect dead code.
- **Files modified:** `src/gh.rs`, `src/proc.rs`, `src/publish.rs`
- **Verification:** `cargo clippy --locked --all-targets --all-features -- -D warnings` passed in the task 1 hook.
- **Committed in:** `422bc41`

**3. [Rule 1 - Bug] Replaced `assert!(comments.is_empty())`**
- **Found during:** Task 2
- **Issue:** The first commit attempt failed in clippy on `clippy::assert_is_empty` in `src/tui/tests/events.rs`. The index stayed staged and HEAD stayed `422bc41`.
- **Fix:** The assertion is `assert_eq!(gh.seen()[0].comments, Vec::new())`. The same subject was committed after the fix. `--no-verify` was not used.
- **Files modified:** `src/tui/tests/events.rs`
- **Verification:** The task 2 hook ran 449 tests, 11 skipped, and committed `203b5d7`.
- **Committed in:** `203b5d7`

**4. [Rule 1 - Bug] `classify` returned `Result` and never failed**
- **Found during:** Task 3
- **Issue:** `clippy::unnecessary_wraps` denied the gate classifier.
- **Fix:** `classify` returns `Gate` directly.
- **Files modified:** `src/store/publish/cancel.rs`
- **Verification:** Clippy passed, then the cancel and publish filters passed, then the task 3 hook ran 453 tests, 11 skipped.
- **Committed in:** `72c2e7a`

### Held for a later plan

`serve` does not execute `reconcile` or `sync`. The plan sentence says those kinds run here. Their executors are 04-07 and 04-08. Returning when the next service job is one of those kinds keeps the loop from spinning and keeps review jobs out of this process. `user_version` stays 5. No crate was added.

---

**Total deviations:** 4 auto-fixed (3 bug, 1 blocking) and 1 later-plan hold
**Impact on plan:** The splits satisfy the commit hook. The dead-code and clippy fixes keep the existing publish path. Reconcile and sync stay unwired on purpose.

## TDD Gate Compliance

Tasks 2 and 3 declared `tdd="true"`. The plan type stays `execute`. Config `tdd_mode` is false. Task 1 stayed a tracer. The user required one feat commit per named subject, and the hk hook runs the full suite, so a commit of failing tests cannot land. `--no-verify` is forbidden.

Task 2 tests were green on the first run. Task 1 had already implemented `a`, `r`, the refusal sentences, and the blocked and submitted paths. There is no separate RED commit. That green result is the behavior the plan asked Task 2 to cover.

Task 3 RED was observed before the production cancel path existed. `mise run --force test -- -E 'test(tui::tests::cancel)' --no-tests=fail` exited 100. `keep_queued_is_highlighted_and_enter_writes_nothing` failed because the frame had no `[Keep queued]`. `the_other_choice_cancels_the_queued_publication` left `pending/queued`. `started_publication_is_not_cancelled` had no `already started` sentence. `quit_leaves_a_queued_publication_pending` passed, because `q` already left the database unchanged. Those failures are assertion failures, not compile errors. Implementation and the tests then landed together in `72c2e7a` (`feat(tui): cancel a queued publication on purpose`).

GREEN for both tasks is the feat commit named in the plan. No refactor commit.

⚡ Tracer verified end-to-end — expanding. The task 1 hook ran the full suite, including `comment_key_posts_one_review`. `AUTO_CHAIN` and `AUTO_CFG` were false. `HUMAN_VERIFY_MODE` is `end-of-phase`, and the tracer verify is automated, so execution continued without a human checkpoint.

## Issues Encountered

The task 2 commit failed once on `clippy::assert_is_empty` and was retried as `203b5d7` after the assertion change. No other hook failure landed a commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 04-06. Stale-head confirmation is still unwired: a mismatched head returns without asking. Reconcile and sync execution remain 04-07 and 04-08. `PUB-13` is also declared by 04-08, so this plan leaves that requirement pending in REQUIREMENTS.md until 04-08 has a summary. `PUB-04`, `PUB-05`, and `PUB-06` are ready to mark because 04-03 already has a summary.

## Self-Check: PASSED

- FOUND: `src/tui/post.rs`
- FOUND: `src/tui/runner.rs`
- FOUND: `src/tui/tests/fake.rs`
- FOUND: `src/tui/tests/publish.rs`
- FOUND: `src/tui/tests/refusals.rs`
- FOUND: `src/tui/tests/events.rs`
- FOUND: `src/store/publish/cancel.rs`
- FOUND: `src/tui/tests/cancel.rs`
- FOUND: `src/tui.rs`
- FOUND: `src/tui/keys.rs`
- FOUND: `src/tui/model.rs`
- FOUND: `src/tui/draw.rs`
- FOUND: `src/tui/confirm.rs`
- FOUND: `src/store/publish.rs`
- FOUND commits: `422bc41`, `203b5d7`, and `72c2e7a` are ancestors of HEAD

## Verification

- Task 1 hook: full nextest, 444 passed, 11 skipped.
- Task 2 hook: full nextest, 449 passed, 11 skipped.
- `mise run --force test -- -E 'test(tui::tests::publish) or test(tui::tests::cancel)' --no-tests=fail` passed: 5 tests, 0 failed, before the task 3 commit. The plan filter `test(tui::tests::publish)` is included and ran `comment_key_posts_one_review`.
- Task 3 hook: full nextest, 453 passed, 11 skipped. Lint, secrets, large-files, docs, and actions passed.
- `mise run test:e2e` was not run. `create_review` in these tests is the scripted fake.
