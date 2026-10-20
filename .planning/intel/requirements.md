# Requirements

Source for all entries: /Users/apaderin/dev/diffmate/SPEC.md, section "Delivery slices". Slice order is the intended phase order. Acceptance criteria are the verbatim "Done when" items of each slice. Product-level goals are in context.md (topic "Goals").

## REQ-single-review
- source: /Users/apaderin/dev/diffmate/SPEC.md (Delivery slices, 1. Single review)
- slice order: 1
- description: Each slice ends with a scenario I can check by hand. Publication rules are specified now and implemented in slice 4. Slice 1 first task, since the agent contract depends on it:
  - isolation checks for the Claude CLI and git;
  - the CLI result envelope fields (`is_error`, `subtype`, `structured_output`, `total_cost_usd`, `duration_ms`);
  - the full output schema through `--json-schema` on the target CLI version: `$defs`, `anyOf`, `minLength`, `maxItems`, nullable fields, and extraction of `structured_output`. Local validation with the full schema stays mandatory either way. If the CLI needs a simplified schema, the CLI form becomes a separate file with a test that checks it against the full schema.

  Then `diffmate review <url>` and `diffmate status`: enqueue a `review` job, take the runner lock, perform recovery, resolve the revision, fetch into the bare clone, create the worktree and context files, check CLI flag support, run the agent with the isolation flags, validate and normalize the output, store the run and its draft, clean up, and print the findings.
- acceptance:
  - A PR with an obvious bug produces a finding anchored to the right line.
  - A clean PR produces `no_findings`.
  - An agent error with exit code 0 (for example, a turn limit) produces `failed`, not `no_findings`.
  - Claude isolation, checked with a canary string instead of a real secret, while authentication keeps working:
    - a `CLAUDE.md` in the worktree containing the canary is not loaded;
    - a `CLAUDE.md` with an `@` import of a file outside the worktree containing the canary does not leak it;
    - `.claude/settings.json` with a hook, a plugin, and `.mcp.json` from the PR do not run or load.
  - Git isolation: a smudge filter and a `core.fsmonitor` command defined in a test global git config, and a `.gitattributes` that selects the filter, do not run during fetch, worktree checkout, or diff.
  - A PR from a fork is reviewed the same way.
  - A missing CLI flag fails the run before the agent starts.
  - A second `review` while the first holds the lock waits, and its job runs after the first; `status` works meanwhile.
- scope: `diffmate review <url>`, `diffmate status`, Claude CLI isolation, git isolation, output schema validation, runner lock, bare clone and worktree

## REQ-reliability
- source: /Users/apaderin/dev/diffmate/SPEC.md (Delivery slices, 2. Reliability)
- slice order: 2
- description: Idempotent runs, crash recovery, freshness, timeouts, `--force`.
- acceptance:
  - Killing diffmate at any step and starting it again leaves no duplicate runs, no orphaned worktrees, and no stale worktree metadata, and interrupted runs show as `failed (interrupted: crash)`.
  - Pushing to the PR during a run makes the draft stale.
  - Advancing the base branch, without a push to the PR, does not make the draft stale.
  - Running `review` twice on the same head reuses the run unless `--force` is given.
  - An invalid `previous_finding_id` is nulled with a warning and the run still succeeds.
  - A PR whose review crashes diffmate every time is retried only within the retry budget, across restarts, and then reported as failed.
  - Recovery after a stop whose cause was stored but whose cleanup did not finish keeps the cause: `shutdown` re-queues an automatic job without using the budget, `user_cancel` ends the job without a retry.
  - Stopping `watch` with SIGTERM during an automatic run records `failed (interrupted: shutdown)`, removes the worktree, and re-queues the job without using the retry budget.
  - Ctrl-C on a `review` that acts as the runner stops and waits for the agent, records `failed (interrupted: user_cancel)` (never `agent_error`), leaves no agent or `rg` processes in its group, removes the worktree, and says so before exiting.
  - Without `watch`, a `review` that attached to a `tui` re-run executes it, and does not start orphaned or automatic jobs.
  - Two `review` commands for the same URL produce one run, a `review` started while a `tui` re-run is queued attaches to it, and `review --force` during a running non-forced job produces a second run.
- scope: idempotency, crash recovery, draft freshness, timeouts, `--force`, signal handling, job attachment

## REQ-queue
- source: /Users/apaderin/dev/diffmate/SPEC.md (Delivery slices, 3. Queue)
- slice order: 3
- description: `diffmate watch` with the queue rules above, jobs handed over from `review`, and macOS notifications for ready drafts and abandoned automatic reviews.
- acceptance:
  - Requesting my review on a PR leads to a ready draft and a notification without any action from me.
  - Two pushes within the debounce window produce one run.
  - A PR stays tracked after I submit a review and my request disappears.
  - A draft PR is not reviewed until it is marked ready.
  - A review request to my team does not queue the PR.
  - A PR I reviewed manually is not reviewed again on new pushes until my review is requested on it.
  - A discarded PR with a still visible request stays untracked; removing and re-adding the request tracks it again.
  - An agent failure is retried, and after the last retry a notification names the reason.
  - `review` while `watch` runs hands the job to `watch` and prints the findings when it ends.
  - Starting `watch` while `review` holds the lock makes `watch` wait, not exit.
- scope: `diffmate watch`, tracking, debounce, retries, notifications, job handoff

## REQ-review-and-publish
- source: /Users/apaderin/dev/diffmate/SPEC.md (Delivery slices, 4. Review and publish)
- slice order: 4
- description: `diffmate tui` shows targets and drafts. I can set decisions, edit findings and future improvements, include the limitation, open the PR in the browser, re-run, reconcile, abandon an `unknown` publication, and publish with `a` (approve), `r` (request changes), `c` (comment).
- acceptance:
  - Publishing creates one GitHub review on the reviewed commit, with kept and edited findings as inline comments carrying finding markers, and the publication marker in the body.
  - Publishing with undecided findings is refused until every finding has a decision.
  - Pressing a publish key twice creates one GitHub review, and publishing an already submitted draft is refused.
  - Publishing a stale draft asks for confirmation first, and asks again if the head moves before the runner sends it.
  - With an existing pending review of mine on the PR, publish stops with a clear message and changes nothing on GitHub.
  - Publishing while `watch` is in the middle of an agent run completes without waiting for that run.
  - A crash before `sending` is committed resumes safely after restart and publishes exactly once.
  - A crash after `sending` is committed leaves the publication `unknown`, with no automatic second POST.
  - Declining a `needs_confirmation` publication cancels the job and abandons the publication; redelivering the old job does not revive it.
  - While `tui` holds the lock it shows that it acts as the runner and does not run `review` jobs.
  - A simulated timeout on the GitHub call leaves the publication `unknown`, and a new attempt stays blocked until reconciliation finds the marker or I abandon it.
  - A failed comment sync leaves the publication `submitted`, inline thread statuses `unknown`, and is retried without a new POST.
- scope: `diffmate tui`, decisions, editing, publication state machine, reconciliation, comment sync

## REQ-daily-use
- source: /Users/apaderin/dev/diffmate/SPEC.md (Delivery slices, 5. Daily use)
- slice order: 5
- description: launchd agent for `watch` (plist lives in dotfiles), install instructions, and `diffmate doctor`.
- acceptance:
  - Done when `watch` survives a reboot and `doctor` explains every failed check.
- scope: launchd agent for `watch`, install instructions, `diffmate doctor`
