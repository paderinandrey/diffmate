---
phase: 01-single-review
plan: 10
subsystem: runner
tags: [rust, tokio, signals, process-group, rustix, cancellation]

requires:
  - phase: 01-single-review
    provides: "01-05 agent spawn (ClaudeCli, process_group(0)), 01-08 runner lock and recovery, 01-09 runner::steps::prepare"
provides:
  - "SIGINT and SIGTERM on a review that runs a job stop the agent's process group (SIGTERM, then SIGKILL after 5 s), record the run failed (interrupted) with no cause, remove the worktree and run directory, print the outcome and exit 1"
  - "agent_pid and agent_pgid stored on the run row as soon as the agent is spawned"
  - "Cancellation seam: a watch channel (cancel::CancelRx, cancel::raced) shared by review, runner and agent adapter"
affects: [01-11, 01-12]

actuals:
  tokens: 6950
  tasks: 2
  commits: 3

plan_head_before: c62e391a577fa02ea278e0f9ecaa37a1cdf41c67
plan_head_after: 0a0d05725e508c88fbf070ead1625ce312f9ff21
commits: 3

tech-stack:
  added: [rustix 1.1.5 (std, process; default features off), tokio signal feature]
  patterns:
    - "Cancellation is a watch::channel(bool); work is dropped with cancel::raced (biased towards cancellation) so kill_on_drop reaps running git and gh children"
    - "Group signalling goes through the GroupSignaller trait; the escalation timer is tested with start_paused and an in-process fake, the real RustixSignaller with a reaped child"
    - "AgentRequest gains builder methods with defaults, so existing AgentRequest::new callers are unchanged"

key-files:
  created:
    - src/cancel.rs
    - src/cancel/tests.rs
    - src/agent/request.rs
    - src/agent/stop.rs
    - src/agent/stop/tests.rs
    - src/agent/claude/cancel_tests.rs
    - src/review/signals.rs
    - src/runner/agent_run.rs
    - src/runner/result.rs
    - src/runner/tests.rs
    - src/render/findings.rs
    - tests/signals.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/agent.rs
    - src/agent/claude.rs
    - src/agent/spawn.rs
    - src/proc.rs
    - src/main.rs
    - src/review.rs
    - src/runner.rs
    - src/runner/outcome.rs
    - src/runner/steps.rs
    - src/render.rs
    - src/store/runs.rs

key-decisions:
  - "A signal during resolution or preparation races the whole step (cancel::raced around resolve and around prepare) instead of checking before each sub-step; dropping the future drops the git or gh child through kill_on_drop"
  - "A run cancelled while preparing or while the agent runs is recorded the same way: failed, failure_kind interrupted, interrupted_cause NULL, detail 'interrupted by signal' (plus cleanup warnings); the job ends failed with 'run N failed (interrupted)'"
  - "stop_group waits only for the direct child after SIGTERM and sends no final SIGKILL once it has exited (as the plan's tests require); a descendant that ignores SIGTERM in a group whose leader already exited would survive"
  - "A cancel before the run exists prints 'Cancelled before the run started; the job failed' instead of the 'recorded as interrupted' line, because no run was recorded"
  - "proc::finish takes impl BorrowMut<Child> so the adapter can keep the child to stop it; the three other callers are unchanged"

patterns-established:
  - "Black-box signal tests send the signal to the diffmate pid with rustix and poll `ps -o stat=` against a 5 s deadline for reaped descendants"

requirements-completed: [REV-02]

coverage:
  - id: D1
    description: "SIGINT and SIGTERM during the agent run stop the agent and its background child, record failed (interrupted) with a NULL cause and the agent PID and PGID, remove the worktree and run directory, print 'Run cancelled; recorded as interrupted' and exit 1"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/signals.rs#sigint_during_agent_run_stops_group_and_records_interrupted"
        status: pass
      - kind: integration
        ref: "tests/signals.rs#sigterm_during_agent_run_is_handled_the_same_way"
        status: pass
    human_judgment: false
  - id: D2
    description: "stop_group sends one SIGTERM when the child exits in time, SIGKILL after exactly AGENT_STOP_GRACE of virtual time otherwise, rejects group ids 0, 1 and above i32::MAX, and treats ESRCH as success"
    requirement: REV-02
    verification:
      - kind: unit
        ref: "src/agent/stop/tests.rs (4 cases, 3 under start_paused)"
        status: pass
    human_judgment: false
  - id: D3
    description: "ClaudeCli::run returns Cancelled when the receiver turns true while the child runs, after the spawn hook received the PID, and never spawns when already cancelled"
    requirement: REV-02
    verification:
      - kind: unit
        ref: "src/agent/claude/cancel_tests.rs (2 cases)"
        status: pass
    human_judgment: false
  - id: D4
    description: "A signal during resolution fails the job with 'interrupted before the run started' and creates no run"
    requirement: REV-02
    verification:
      - kind: unit
        ref: "src/runner/tests.rs#a_cancel_before_the_run_exists_fails_the_job_and_creates_no_run"
        status: pass
    human_judgment: false
  - id: D5
    description: "A signal during preparation (after the run row exists, before the agent) cancels the step, records the run interrupted and cleans up"
    requirement: REV-02
    verification: []
    human_judgment: true
    rationale: "No test holds git, gh or context writing mid-step; the path shares the agent-cancel recording code, which D1 covers, and the pre-run path, which D4 covers"
  - id: D6
    description: "A review that only waits for the lock keeps the default signal behavior"
    requirement: REV-02
    verification: []
    human_judgment: true
    rationale: "Structural: the listeners are installed in review::signals::until_done, called after lock::acquire and recovery::run; no test sends a signal to a waiting review"

duration: 25min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 10: Signal handling for a running review Summary

**SIGINT and SIGTERM on a running `review` stop the agent's whole process group (SIGTERM, SIGKILL after a paused-time-tested 5 s grace), record the run failed (interrupted) with the agent PID and PGID, remove the worktree and exit 1, proven by two black-box tests that check the fake agent and its background child are gone**

## Performance

- **Duration:** about 25 min (start time was not recorded; counted from the first dependency change to the final commit, 2026-10-20 about 22:25 to 22:47 local)
- **Tasks:** 2 (3 commits: one per task and one small follow-up)
- **Files:** 29 changed in the plan range (12 created, 17 modified), 842 insertions and 155 deletions

## Accomplishments

- `src/agent/stop.rs`: `stop_group` sends SIGTERM to the group, waits for the direct child or `AGENT_STOP_GRACE` (5 s), sends SIGKILL on timeout, then awaits the child. `RustixSignaller` calls `kill_process_group` (no `unsafe`), maps ESRCH to success and refuses group ids 0, 1 and anything above `i32::MAX`. `halt` falls back to `start_kill` when signalling fails.
- `ClaudeCli::run` returns `AgentOutcome::Cancelled`. After `spawn` the adapter sends the PID through the spawn hook, races `proc::finish` against the cancel receiver and, on cancellation, calls `halt`. A request that is already cancelled never spawns.
- `review` installs SIGINT and SIGTERM listeners only after the lock and recovery (a review that merely waits keeps default behavior). The first signal sets a `watch` channel; the job future keeps being polled until it finishes; later signals are absorbed.
- The runner races `resolve` and `prepare` against the channel, passes the receiver and a spawn hook to the agent, and records `agent_pid`/`agent_pgid` through `Store::set_agent_process` while the agent runs. A cancelled run goes through `finish_failed` with `FailureKind::Interrupted`, no cause and detail `interrupted by signal`, then the normal cleanup; the result is `ReviewResult::Cancelled`, rendered as `OWNER/REPO#N: Run cancelled; recorded as interrupted` on stderr with exit code 1.

## Task Commits

1. **Task 1: group stop with bounded escalation, cancellation in the agent adapter** - `80accbb` (feat)
2. **Task 2: signal handler, PID recording, cleanup and output** - `1bb4344` (feat)
3. **Follow-up: keep `record_agent_process` in `src/runner.rs`** (plan acceptance greps for `set_agent_process` there) - `0a0d057` (refactor)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Decisions Made

- See `key-decisions` in the frontmatter.
- rustix is the only new crate. Reason for the PR description: `libc::killpg` needs `unsafe`, which is forbidden, and tokio's `start_kill` reaches only the direct child. Default features are off; features `std` and `process`.
- tokio stays at 1.53.3: 1.54.0 was published on 2026-10-19, less than 3 days before today. rustix 1.1.5 (2026-09-16) is the newest eligible release.

## Verification

Observed in this session:

- `mise run --force test -- -E 'test(agent::stop::) | test(agent::) | test(cancel::)'` (Task 1): 49 passed.
- `mise run --force test -- -E 'binary(signals) | binary(review) | binary(recovery) | binary(concurrency)'` (Task 2): 7 passed; the two signal tests were run three more times, all passed.
- Mutation check, observed and reverted: replacing the group stop with `start_kill` on the direct child only fails both signal tests ("process N is still running" for the background child).
- `mise run --force test`: 154 run, 154 passed, 3 skipped (the live probes); also run by the hk pre-commit hook on all three commits.
- `mise run --force lint`: exit 0 (rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks, actionlint).
- `mise run coverage`: 91.39% line coverage (gate 80%).
- `mise run deps:check`: exit 0.
- Acceptance greps: `kill_process_group` in stop.rs = 2, `start_paused = true` in stop/tests.rs = 3, `set_agent_process` in runner.rs = 1, `recorded as interrupted` in render.rs = 1; every file the plan touches is at most 100 lines (largest: src/runner.rs 99, src/agent/claude.rs 99, src/agent/stop/tests.rs 99).
- Not run: `mise run test:e2e`, `mise run docs:check` outside the hook. `mise run vuln` fails on licenses for a pre-existing reason (see Issues Encountered); advisories, bans and sources are ok.

## Deviations from Plan

**1. [Rule 3 - Blocking] Extra files for the 100-line limit and for sharing the cancel seam**
- **Issue:** `src/agent.rs`, `src/runner.rs`, `src/render.rs` and `src/runner/steps.rs` would pass 100 lines.
- **Fix:** added `src/agent/request.rs` (AgentRequest and its builders, re-exported from `crate::agent`), `src/cancel.rs` and `src/cancel/tests.rs` (CancelRx, cancelled, raced), `src/runner/result.rs` (ReviewResult, job_failed, cancelled_before_run), `src/runner/agent_run.rs` (run_agent moved out of steps.rs), `src/render/findings.rs` (finding and note printing moved out of render.rs), `src/review/signals.rs`, `src/runner/tests.rs`, `src/agent/claude/cancel_tests.rs`. Public paths named in the plan interfaces hold: `agent::stop::{AGENT_STOP_GRACE, GroupSignaller, StopSignal, RustixSignaller, stop_group}`, `AgentRequest::{with_cancel, with_spawn_hook}`, `ReviewResult::Cancelled { run_id: Option<RunId> }`, `Store::set_agent_process`.
- **Committed in:** 80accbb, 1bb4344

**2. [Rule 3 - Blocking] Existing exhaustive matches on AgentOutcome**
- **Issue:** the plan expected no exhaustive matches; three envelope test helpers (`support.rs`, `tests.rs`, `property_tests.rs`) match on it.
- **Fix:** each match arm also covers `Cancelled`.
- **Committed in:** 80accbb

**3. [Rule 3 - Blocking] proc::finish signature**
- **Issue:** the adapter must keep the child to stop it after racing the output collection; `finish` consumed it.
- **Fix:** `finish` takes `impl BorrowMut<Child>`; the three other callers still pass an owned `Child`.
- **Committed in:** 80accbb

**4. [Rule 1 - Design] Key link from claude.rs to stop_group**
- **Issue:** the plan's key link names `src/agent/claude.rs`, but `ClaudeCli::run` delegates spawning to `src/agent/spawn.rs`.
- **Fix:** the cancellation call (`halt`, which calls `stop_group`) lives in `spawn.rs` and `stop.rs`; claude.rs only maps the `None` result to `Cancelled`.

**5. [Rule 1 - Design] Pre-run cancel message**
- **Issue:** with no run row, "recorded as interrupted" would be false.
- **Fix:** `Cancelled { run_id: None }` prints `OWNER/REPO#N: Cancelled before the run started; the job failed`. The acceptance grep for `recorded as interrupted` still finds one line.
- **Committed in:** 1bb4344

**6. [Rule 3 - Blocking] Added unit tests beyond the plan**
- **Fix:** `src/runner/tests.rs`, `src/cancel/tests.rs` and two render cases cover the pre-run cancel, the race helper and both Cancelled outputs; they keep coverage at 91%.

**Total deviations:** 6 (4 blocking, 2 design). **Impact:** none on behavior beyond the plan; interfaces keep their names.

## Issues Encountered

- `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were already modified or untracked and were left out of every commit.
- `mise run vuln` reports `licenses FAILED`: `borrow-or-share 0.2.4` (MIT-0), a transitive crate of `jsonschema`, is not in the deny.toml allow list. It predates this plan and is unrelated to rustix; logged in `.planning/phases/01-single-review/deferred-items.md`, not fixed.
- Recovery still always records `crash`; `shutdown` and `user_cancel` causes and the retry rules stay in Phase 2 (REL-08, REL-09).

## User Setup Required

None.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-43 (group SIGTERM then SIGKILL after 5 s, direct child awaited; both signal tests check the agent and its background child are gone), T-01-44 (a cancelled run is stored interrupted before any exit code is read; AgentOutcome::Cancelled never reaches the classifier), T-01-45 (PGID is the pid of the child spawned with process_group(0); 0, 1 and ids above i32::MAX are rejected in stop_group and in RustixSignaller), T-01-46 (PID and PGID stored on the run row, asserted in the signal tests), T-01-SC (rustix with default features off).

## Next Phase Readiness

- Plan 01-11 can keep calling `AgentRequest::new` and `ClaudeCli::run`; both are unchanged in signature. `execute_review_job` gained a `&CancelRx` argument.
- The `Cancelled` result for a signal during preparation (D5) and the waiting-review default (D6) have no black-box test.
- Plan 01-02 still waits on the owner; `tests/fixtures/live-prs.json` was not touched.

## Self-Check: PASSED

All 12 created files exist and commits 80accbb, 1bb4344 and 0a0d057 are ancestors of HEAD (verified with `[ -f ]` and `git merge-base --is-ancestor` in the closing step).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
