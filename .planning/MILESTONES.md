# Milestones

## v1.0 SPEC (Shipped: 2026-11-14)

**Phases completed:** 5 phases, 43 plans, 74 tasks

**Closeout:** verified. Known verification overrides: 0 newly acknowledged, 0 carried forward.

The accomplishment lines below are the plan summaries as written. Two of them still name Claude and an Anthropic block. The shipped review agent is the grok CLI. Phase 2 was accepted on 2026-11-14 after the five Grok live checks passed (nextest c3965e45). The ignored Claude isolation checks stay the slice 1 record. `MIT-0` is already allowed in `deny.toml`; the phase 01 deferred item was marked resolved before this close.

**Key accomplishments:**
- Claude CLI 2.1.304 spike: the full output schema works through --json-schema, the isolation flags hold on a local fake Anthropic endpoint, and the CLI's own git calls need a PATH git wrapper (owner decision A1)
- Three owner-approved fixture PRs (obvious off-by-one, clean README change, fork PR) were verified with read-only gh calls and recorded in tests/fixtures/live-prs.json with head SHAs and the calc.py line 7 RIGHT expected anchor for plan 01-12's live checks
- Sandbox with temporary HOME, file-configured fake claude and gh, a github.com-to-local git shim and a bare-repository builder that publishes PR heads only under refs/pull/N/head
- diffmate review URL works end to end: CLI, SQLite job, gh, bare clone and worktree, isolated claude with the PATH git wrapper, full-schema validation, one-transaction run, draft and findings, cleanup on every exit, proven by one black-box tracer test
- Version and flag probe before every run (minimum Claude CLI 2.1.304, exact production vector, fail closed as cli_error), full D-01 envelope classification proven against the spike's golden envelopes, and the schema form, environment allowlist and production vector pinned to contract.json
- diffmate's own git use is hardened to the SPEC Git section and proven against a poisoned global config (five config routes, mandatory plain-git control), with refspec-safe branch names, symlinks checked out as plain files, and a fork PR whose head lives only under refs/pull/N/head
- Hunk-index anchoring with a hand-written unified-diff parser (proptested against a git-format model), SPEC normalization with recorded warnings, and the D-04 plain-text output with bidi-safe terminal sanitizing
- Exclusive File::try_lock runner lock with a waiting loop, crash recovery that runs before any job, and a read-only `diffmate status`, proven by a FIFO-synchronized two-process test and a SIGKILL recovery test
- Size-capped pr.md and diff.patch (hunk-boundary cut), discussion.md and ci.md from read-only gh calls with "CI status unknown" never read as passing, previous.json from the latest successful run feeding normalize, and rules/ read at the merge-base with one-hop symlinks
- SIGINT and SIGTERM on a running `review` stop the agent's whole process group (SIGTERM, SIGKILL after a paused-time-tested 5 s grace), record the run failed (interrupted) with the agent PID and PGID, remove the worktree and exit 1, proven by two black-box tests that check the fake agent and its background child are gone
- REV-06, REV-07 and REV-08 are proven on CLI 2.1.304 with canaries and positive controls against a local recording Anthropic endpoint, and one live call shows authentication still works under the production flags
- Four #[ignore] live tests drive review::run with the real claude and gh against the owner's fixture PRs and the turn-limit case, all four passed on their single run, and every phase gate except `mise run vuln` (pending the MIT-0 owner decision) passes
- Review cancellation stores its first cause before signalling the agent, preserving user_cancel, shutdown and timeout through outcome recording and cleanup.
- Status compares each saved draft with one live PR head, labels fallback checks with their stored age, and displays persisted retry budgets without writing state.
- Recovery and live outcomes preserve stop causes, persist bounded retry schedules, and keep automatic retries on the recorded revision even after the PR head moves.
- Repeated reviews reuse a successful same-head draft, concurrent requests share an active job, and waiting processes return stored results or stop without cancelling work.
- The runner records the last observed PR head, successful reviews report stale drafts on their first line, and failed rechecks preserve the draft with a durable warning.
- Preparation and agent execution share the configured timeout, and the runner persists its cause before stopping the agent and recording `failed (timeout)`.
- Watch executes the due review queue with timestamped events, durable shutdown and lock-holder roles; attached reviews preserve retry delays and leave unrelated work queued.
- Fifteen feature-only crash points and twenty-two process tests prove restart cleanup, bounded retries and retained stop causes. Implementation is committed; the final live gate is blocked by an Anthropic account restriction.
- Paged `gh search prs --review-requested=@me` on the Gh trait, keeping drafts and rejecting search rows whose URL, number, and repository disagree
- Watch polls direct review requests on the configured interval and arms one debounced automatic job through the existing store
- Automatic success and give-up call osascript with sanitized titles, and a failed notifier leaves the job unchanged
- Hidden `diffmate debug discard` calls `Store::discard`, and a running watch tracks the PR again only after the request leaves and returns
- An empty successful poll does not start a second runner: the waiting review prints the findings, and watch does not search until it holds the lock
- Store transitions for request_seen, discard, and one debounced automatic job per head, including a follow-up after the runner stores a new head
- A transient agent_error for HTTP 408, 429, or 500-599 uses the existing timeout retry budget, and status 401 and 403 stay terminal.
- Schema version 5 with a version-4 snapshot, a newest-first `diffmate tui` queue, and a draft screen in review print order, on ratatui 0.30.2.
- The owner can keep, drop, or edit a finding in EDITOR, and can include the limitation, before any publish key exists.
- One decided draft posts one GitHub review from its confirm snapshot, and a second confirm creates nothing.
- Queue and draft keys confirm a discard, open the PR with one argument, and enqueue a manual re-run without running it.
- Publish keys post one review from a decided current draft, and u can abandon a publication that has not started.
- A stale or moved head asks before send, decline sticks, and a pending review of mine is left untouched
- A crash or timeout after create_review leaves one unknown publication, and y or b settles it without a second POST
- Watch publishes and syncs comments during an agent run, retries a failed sync without another POST, and refreshes previous.json thread status from GraphQL.
- tui serve runs a due comment sync through execute_sync, then still reconciles and publishes later jobs without a second review POST.
- The queue and diffmate status keep the latest review run after a later publish or sync job, including succeeded (findings: N).
- Read-only `diffmate doctor` prints gh auth, Claude, storage, the runner lock, and schema, and exits 0 only when every check passes
- Doctor names a held watch lock, schema age, a loose directory, and a missing gh or claude without migrating, chmodding, or taking the lock
- `docs/install.md` tells a fresh Mac how to build watch, install a GUI launchd agent with string umask 0077, update it with kickstart, remove it with bootout, and empty the two logs

---
