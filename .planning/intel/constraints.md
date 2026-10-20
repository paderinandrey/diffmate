# Constraints

All entries are from /Users/apaderin/dev/diffmate/SPEC.md unless stated otherwise. Content is verbatim from the source. The "Transitions" table (SPEC.md lines 117-143, job x publication transitions including crashes) is not copied here; read it in the source.

## CON-prop-1: Required property 1 - revision recorded per run
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 1)
- type: nfr
- content: Every run records the revision it reviewed. Every finding belongs to exactly one run.

## CON-prop-2: Required property 2 - freshness check before publication
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 2)
- type: nfr
- content: Before publication diffmate fetches the PR head. If the draft is stale, publication needs an explicit confirmation or a re-run.

## CON-prop-3: Required property 3 - crash safety without duplicates
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 3)
- type: nfr
- content: Restarting after a crash at any point does not create duplicate runs, drafts, or worktrees, and does not create a second GitHub review for the same draft. diffmate does not promise an atomic transaction across SQLite and GitHub; it relies on the publication ID, the markers, reconciliation, and my confirmation.

## CON-prop-4: Required property 4 - failures are never "no findings"
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 4)
- type: nfr
- content: An agent or CLI failure is never reported as "no findings", and a failed automatic review is never silent.

## CON-prop-5: Required property 5 - publication requires user action
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 5)
- type: nfr
- content: Publication requires my action and a decision on every finding. The agent never calls GitHub APIs.

## CON-prop-6: Required property 6 - agent isolation
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 6)
- type: nfr
- content: The agent cannot run commands, write files, reach the network, use my MCP servers, or load instructions, imports, hooks, or plugins from the PR, within the limits of the Claude CLI isolation flags (see "Agent contract"). Text from the PR is untrusted input and cannot change the review rules.

## CON-prop-7: Required property 7 - existing pending review untouched
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 7)
- type: nfr
- content: If I already have a pending review on a PR in GitHub, diffmate does not publish, and does not extend, change, or delete that review.

## CON-prop-8: Required property 8 - single mutating process
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 8)
- type: nfr
- content: Only one process at a time changes runs, worktrees, clones, or publications (see "Concurrency").

## CON-prop-9: Required property 9 - bounded retries after crashes
- source: /Users/apaderin/dev/diffmate/SPEC.md (Required properties, 9)
- type: nfr
- content: A crash never makes diffmate restart the same review endlessly: interrupted reviews are retried only within the retry budget.

## CON-run-states: Run state machine
- source: /Users/apaderin/dev/diffmate/SPEC.md (States, Run)
- type: protocol
- content:
  ```text
  queued → running → succeeded (findings | no_findings)
                   → failed (cli_error | agent_error | timeout | invalid_output | interrupted)
  ```
  - `findings` and `no_findings` are results of a successful analysis. `failed` is never shown as `no_findings`.
  - `interrupted` carries a cause:
    - `shutdown`: a planned stop; the runner received SIGTERM (for example, launchd at logout or reboot).
    - `user_cancel`: SIGINT on a `review` process that runs the job.
  - Signals by role: SIGTERM → `shutdown` for any process; SIGINT → `shutdown` for `watch` and `tui`, `user_cancel` for a `review` that runs the job, and for a `review` that only waits it ends the wait without cancelling the job.
    - `crash`: recovery found a `running` run without a stored stop cause.
  - On `shutdown` or `user_cancel` the signal handler only stores the cause and requests cancellation. The main loop then stops the agent (see "Agent contract"), records the run, removes the worktree, and releases the lock. If that cleanup does not finish, recovery completes it and keeps the stored cause.
  - Retries of automatic runs: see "Queue rules".

## CON-target-states: Target lifecycle and tracking
- source: /Users/apaderin/dev/diffmate/SPEC.md (States, Target)
- type: protocol
- content:
  - Lifecycle: `open ⇄ closed`, `open → merged`, read from GitHub. A reopened PR returns to `open`.
  - Tracking: `untracked ⇄ tracked`.
    - A direct review request makes the target tracked, whether it was created manually or not.
    - Tracking continues after my review request disappears (for example, after I submit a review).
    - Tracking stops when the PR is merged or closed, or when I discard the target.
    - A discarded target becomes tracked again only after a re-request: a successful poll without my request followed by a successful poll with it. A request that stays visible after the discard does not count. A removal and re-request that both happen between two polls are missed; this is an accepted MVP limitation.
    - Reopening a PR does not cancel a discard. A reopened PR becomes tracked again only through the same request rules.
    - A manual run never changes tracking.

## CON-draft-freshness: Draft freshness
- source: /Users/apaderin/dev/diffmate/SPEC.md (States, Draft freshness)
- type: protocol
- content: A draft is `current` while the last known PR head equals the run's head SHA, and `stale` otherwise. Views show freshness from the last known head together with the time it was checked. Publication always fetches the head again. The base branch moving on does not make a draft stale.

## CON-publication-states: Publication state machine
- source: /Users/apaderin/dev/diffmate/SPEC.md (States, Publication state)
- type: protocol
- content:
  ```text
  pending → sending → submitted
                    → rejected    (GitHub returned a definitive client error, such as 422)
                    → unknown     (network error, timeout, or a server error)
  pending → abandoned             (the job stopped before sending, or I declined; the reason is stored)
  unknown → submitted             (reconciliation found the marker)
  unknown → abandoned             (I confirmed in the TUI that nothing was published)
  ```
  - Invariant: `pending` means sending has not started. `sending` means the outcome may already be unknown. `sending` is committed to SQLite before the POST starts, so the boundary is that commit, not the POST itself: a crash between the two also yields `unknown`.
  - Recovery leaves `pending` publications alone, so their job may still send safely after all checks run again. A publication found in `sending` becomes `unknown`. There is never an automatic second POST.
  - A missing marker never moves `unknown` to `rejected`. Not finding the review does not prove it was not created. Only my explicit confirmation ends an `unknown` publication without a GitHub review.
  - `abandoned` is final. Redelivering an old job never revives an abandoned publication.
  - A publication in `pending`, `sending`, or `unknown` blocks any new publication for the same target.
  - A draft is published at most once. A `submitted` publication blocks any further publication of that draft, and editing the draft afterwards does not lift the block; publishing again requires a new run and its new draft. A `rejected` or `abandoned` publication allows a new attempt on the same draft.
  - Checking these rules and creating the publication happen in one SQLite transaction, so two different jobs cannot both pass the check.
  - The publication holds an immutable snapshot taken at confirmation: the event, the selected findings with their decisions and text, the limitation if included, and the future improvements. Later edits to the draft never change a confirmed publication.

## CON-job-states: Job state machine
- source: /Users/apaderin/dev/diffmate/SPEC.md (States, Job)
- type: protocol
- content:
  ```text
  queued → running → succeeded
                   → failed
                   → needs_confirmation   (publish only: the head moved past the confirmed SHA)
  needs_confirmation → queued             (I confirm again, for the same draft and the new head SHA)
  needs_confirmation → cancelled          (I decline; the publication becomes abandoned)
  queued → cancelled                      (publish only: I cancel before it starts; the publication becomes abandoned)
  ```
  - A job's state describes how the command was processed. The outcome of a GitHub publication lives in the publication, not in the job.
  - `needs_confirmation` is shown as waiting for my action, and I can cancel it.
  - A `queued` `publish` job and its `pending` publication stay visible in `tui`, and I can cancel them while the job has not started: the job becomes `cancelled` and the publication `abandoned`. Closing a waiting `tui` does not cancel a confirmed publication.
  - A job records the PID and start time of the process that enqueued it, as information only. A job is orphaned when that process no longer exists.
  - A `publish` job carries the publication ID created at confirmation. Redelivering the job reuses that ID, and enqueuing a job with a publication ID that already exists creates nothing, so a double key press or a repeated enqueue yields one review.
  - A `review` job creates a run. Its result is the run's state.
  - After `interrupted: shutdown`, an automatic `review` job returns to `queued` without using the retry budget, and a manual one ends `failed`.
  - After `interrupted: user_cancel`, the job ends `failed` and is not retried, automatic or not; I cancelled it on purpose.
  - Merging happens atomically when a `review` job is enqueued: a request attaches to a matching active job of the same target (see "One model for every process"). Deduplication against finished work happens when the job starts: the runner fetches the current head again, and if a successful run already exists for it and the job has no `--force`, the job succeeds by reusing that run. Two `review` commands for the same URL therefore produce one run.
  - Recovery of `running` jobs:
    - `publish` with its publication in `pending`: back to `queued`; all checks run again before sending;
    - `publish` with its publication in `sending`: `failed` with reason "outcome unknown", and the publication becomes `unknown`;
    - `review`: its run gets the stored stop cause, or `crash` when none was stored, and the job then follows the rule for that cause (see "Run"): requeued without using the budget, retried within the budget, or ended `failed`;
    - `reconcile` and `sync`: back to `queued`.

## CON-concurrency: Concurrency and runner lock
- source: /Users/apaderin/dev/diffmate/SPEC.md (Concurrency)
- type: protocol
- content:
  - The runner is the process that holds an exclusive OS file lock (`flock`) on `~/Library/Application Support/diffmate/runner.lock`. The OS releases the lock when the process exits. A PID file alone is never treated as a lock.
  - Only the runner runs agents, performs recovery and cleanup, fetches into clones, publishes, reconciles, and syncs comments.
  - Any process may read SQLite and make read-only GitHub calls. `status` never performs recovery.
  - SQLite runs in WAL mode so readers do not block the runner.
  - The agent runs as a separate subprocess. While it runs, the runner keeps serving its job queue: publication, reconciliation, and comment sync jobs touch only GitHub and SQLite, never clones or worktrees, so they do not wait for a 15-minute review.
  - Other processes learn the result of a job from its state in SQLite.

## CON-recovery: Recovery procedure
- source: /Users/apaderin/dev/diffmate/SPEC.md (Concurrency, Recovery)
- type: protocol
- content: Recovery runs every time a process becomes the runner, before it executes any job. It is correct only because the lock guarantees that no other process is running agents or sending publications at that moment. Recovery:
  1. moves `running` runs to `failed (interrupted: <stored cause>)`, or `failed (interrupted: crash)` when no stop cause was stored;
  2. moves `sending` publications to `unknown`;
  3. resolves `running` jobs as described in "Job";
  4. removes worktrees of runs that are not `running` and runs `git worktree prune`;
  5. enqueues a `reconcile` job for every `unknown` publication, including those it just moved from `sending`, unless that publication already has an active (`queued` or `running`) `reconcile` job. Recovery itself makes no network calls; the job does.

## CON-process-model: One model for every process
- source: /Users/apaderin/dev/diffmate/SPEC.md (Concurrency, One model for every process)
- type: protocol
- content:
  - `review <url>` and `tui` never act directly. They enqueue a job and then try to take the lock.
    - If they get it, they become the runner: they perform recovery, run the jobs they are responsible for, and release the lock.
    - If they do not, they wait for their job's result in SQLite and keep retrying the lock while they wait. If the runner exits before their job is done, the next process to get the lock becomes the runner.
  - Which jobs a runner executes:
    - `watch`: every job.
    - `review`: only its own job or the job it attached to, whether or not the process that created that job is still running. Orphaned jobs and the automatic backlog are left to `watch`, so a manual `review` never starts by working through a backlog of 15-minute runs.
    - `tui`: never `review` jobs. While it holds the lock it serves every `publish`, `reconcile`, and `sync` job, including orphaned ones, so a publication left `pending` by a closed `tui` is picked up by the next `tui` even without `watch`. It shows clearly that it is acting as the runner, so a waiting `review` does not look stuck. A re-run requested in `tui` with no `watch` running is enqueued, and `tui` explains that `diffmate review` or `diffmate watch` must run it.
  - So a queued job never waits without an executor while `watch` runs or while a process waiting for it is alive. An orphaned `review` job waits for `watch`, or for a later `review` of the same target that attaches to it. An orphaned `publish`, `reconcile`, or `sync` job runs on `watch` or the next `tui`.
  - `watch` is the runner for as long as it runs. If the lock is held when it starts, it logs that and retries the lock with pauses instead of exiting, so launchd does not restart it in a loop. It exits cleanly on a termination signal while waiting.
  - `review <url>` enqueues its job and either runs its own or attached job or waits, retrying the lock with pauses. In one SQLite transaction it looks for an active `review` job of the same target, whoever created it (for example, a re-run requested in `tui`):
    - a request without `--force` attaches to any active job;
    - a request with `--force` attaches to a queued job and marks it `--force` explicitly, but creates a new job when the active job is already running without `--force`;
    - otherwise it creates a new job.

    It then waits for the job it attached to or created, regardless of who created it. It prints the findings when its job ends. Interrupting the wait does not cancel the job; the job becomes orphaned. On Ctrl-C, `review` prints the actual outcome:
    - if it was only waiting: "Stopped waiting; the job stays queued" and, when `watch` holds the lock, "and `watch` will run it";
    - if it was the runner: "Run cancelled; recorded as interrupted".
  - `tui` publishes through a `publish` job:
    1. `tui` checks what it can before creating anything: no `submitted` publication for the draft, no blocking publication for the target, and no `undecided` finding. A failed check is shown to me and creates no publication or job.
    2. `tui` fetches the PR head itself (read-only) and asks me to confirm if the draft is stale.
    3. On confirmation it creates the publication in `pending` with its snapshot and enqueues the job in the same SQLite transaction, so a publication never exists without its job, with the event and the head SHA I confirmed against. A confirmation applies to that draft and that SHA only.
    4. The runner repeats the checks (see "Publication"). If the head differs from the confirmed SHA, the job ends as `needs_confirmation` with the head SHA the runner saw, and `tui` asks again.
    5. `tui` follows the publication state in SQLite and shows the outcome when it changes.

## CON-storage: Storage layout and cleanup
- source: /Users/apaderin/dev/diffmate/SPEC.md (Storage)
- type: schema
- content: All diffmate data lives in its own directories. My working copies are never used.
  - State: SQLite at `~/Library/Application Support/diffmate/diffmate.db`.
  - Repositories: one bare clone per repo at `~/Library/Application Support/diffmate/repos/<owner>/<repo>.git`, cloned from the PR's base repository. For each run diffmate fetches the PR head into `refs/diffmate/pull/<n>/head` and the base branch into `refs/diffmate/base/<branch>`, from that repository; this covers PRs from forks. The refs keep the objects safe from `git gc` while a target is open. A run works only from the SHAs recorded in its revision, so a later fetch that moves these refs does not change the input of a run in progress.
  - Worktrees: detached at the head SHA, at `~/Library/Caches/diffmate/runs/<run-id>/worktree`.
  - Agent input files: `~/Library/Caches/diffmate/runs/<run-id>/context/`.
  - Cleanup: when a run finishes, the runner runs `git worktree remove --force` and deletes the run directory. During recovery it does the same for its own worktrees whose run is not `running`, then runs `git worktree prune`. The `refs/diffmate/pull/<n>/*` refs of merged and closed targets are deleted.

## CON-git: Git invocation isolation
- source: /Users/apaderin/dev/diffmate/SPEC.md (Git)
- type: protocol
- content: diffmate runs git itself, with fixed arguments and a controlled environment:
  - `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`, so my global and system git config (filters, `core.fsmonitor`, aliases, helpers) never applies.
  - Every other inherited `GIT_*` variable is removed.
  - The local config of each clone is written by diffmate and contains no filters, no fsmonitor, and `core.hooksPath` pointing at an empty directory.
  - Authentication: `-c credential.helper='!gh auth git-credential'`, passed only to git calls that talk to GitHub (clone and fetch).
  - Diffs use `--no-ext-diff --no-textconv`.
  - Files tracked with filters (for example, Git LFS) appear as stored in git, such as LFS pointer files.

## CON-agent-input: Agent input files
- source: /Users/apaderin/dev/diffmate/SPEC.md (Agent input)
- type: api-contract
- content: diffmate collects everything the agent needs, because the agent has no GitHub access. It writes these files into the run's context directory:
  - `pr.md`: title, description, author, base and head branch, head SHA, merge-base SHA.
  - `diff.patch`: `git diff --no-ext-diff --no-textconv <merge-base> <head>`.
  - `discussion.md`: PR comments and review threads, with resolved threads marked.
  - `ci.md`: check runs for the head SHA with their conclusion, plus annotations of failed checks.
  - `previous.json`: from the latest successful earlier run on this target, if any: each original finding with its finding ID, and separately my decision on it and its thread status. Thread statuses of linked findings are read fresh through GraphQL right before the run. Dropped findings count as rebutted.
  - `rules/`: `CLAUDE.md`, `AGENTS.md`, `CONTRIBUTING.md`, and `REVIEW.md` from the repository root, read at the merge-base, so the PR cannot change its own review rules.

  Every file states when its data is unavailable or truncated. Missing CI data is written as "CI status unknown", never as passing.

## CON-agent-contract: Agent invocation and outcome contract
- source: /Users/apaderin/dev/diffmate/SPEC.md (Agent contract)
- type: protocol
- content:
  - Invocation: `claude -p` with the worktree as working directory and `--add-dir` for the context directory. The prompt is passed on stdin, because `--tools` takes a variable number of values and would consume a trailing argument.
  - Isolation flags: `--safe-mode`, `--restricted`, `--strict-mcp-config` without `--mcp-config`, `--tools Read,Grep,Glob`, `--no-session-persistence`. `--allowedTools` is not used to limit tools.
  - Managed (policy) settings still apply under these flags, so the flags alone do not prove full isolation. Whether the flags hold is established by the isolation checks in slice 1, not assumed.
  - CLI support: diffmate requires a minimum Claude CLI version, and before every run checks that each flag it uses is supported. If one is missing, the run fails with `cli_error` and does not start the agent.
  - Environment: built from an explicit allowlist (`PATH`, `HOME`, locale, and the variables Claude needs for authentication). Nothing else is inherited. No GitHub credentials.
  - Instructions: `prompts/review.md` is passed with `--append-system-prompt-file`.
  - Output: `--output-format json` with `--json-schema` set to `schemas/review.schema.json`. The file declares `$schema` draft 2020-12, and diffmate's validator uses that draft explicitly. The CLI rejects the draft 2020-12 meta-schema URI (checked with Claude CLI 2.1.304), so diffmate removes the `$schema` key before passing the schema to `--json-schema`. The run succeeds only when all of these hold, in order:
    1. the process exits with code 0;
    2. the CLI result envelope parses, `is_error` is false, and `subtype` is `success`;
    3. `structured_output` matches the schema;
    4. `status` agrees with whether `findings` is empty.
  - Any failed check makes the run `failed`.
  - Normalization after a successful run, each recorded as a warning on the run:
    - a finding whose `line` is not inside a hunk of `diff.patch` is moved to the review body with `line` and `side` set to `null`;
    - a `previous_finding_id` that does not refer to a finding in `previous.json` is set to `null`, and the finding is treated as new.
  - The run stores `total_cost_usd` and `duration_ms` from the result envelope.
  - Timeout: 15 minutes by default. Hitting it makes the run `failed (timeout)`.
  - Process group: the agent starts in its own process group (on Unix, `Command::process_group(0)`), so a terminal Ctrl-C reaches only diffmate and only diffmate stops the agent. On cancellation or timeout diffmate sends SIGTERM to the group, then SIGKILL after a bounded wait, and waits for its direct child to exit. This covers descendants that stay in the group, such as `rg`; diffmate makes no guarantee for processes that create their own group.
  - Outcome classification: the stop reason is stored before any signal is sent, and it takes precedence over the exit code: timeout → `failed (timeout)`, planned stop → `failed (interrupted: shutdown)`, Ctrl-C on the runner → `failed (interrupted: user_cancel)`. Only without a stored reason do the exit code and result envelope decide.

## CON-publication-procedure: Publication job procedure
- source: /Users/apaderin/dev/diffmate/SPEC.md (Publication)
- type: protocol
- content: The steps below are the `publish` job. The publication, with its ID, was created in state `pending` when I confirmed sending. Steps 1 and 2 repeat the checks `tui` already made, because the state may have changed in between.
  1. The draft has no `submitted` publication, and no other publication of the target is `pending`, `sending`, or `unknown`.
  2. Every finding has a decision other than `undecided`. The TUI offers a shortcut that marks all remaining undecided findings `kept`.
  3. diffmate fetches the PR head and checks it against the SHA I confirmed (property 2).
  4. diffmate lists my reviews on the PR. If I have a pending review, publication stops with a message (property 7).

     Early exits: if step 1, 2, or 4 fails, the publication becomes `abandoned` with the reason and the job `failed`. A transient GitHub error in steps 3 or 4 is retried a limited number of times before it counts as a failure; no POST has happened yet. If step 3 finds a different head, the job ends `needs_confirmation`, and the publication stays `pending` until I confirm again or decline. See "Transitions".
  5. diffmate moves the publication to `sending` and creates the review with `commit_id` set to the run's head SHA:
     - inline comments for anchored `kept` and `edited` findings, each ending with `<!-- diffmate:finding=<finding-id> -->`;
     - a body with unanchored `kept` and `edited` findings, then the limitation if I included it, then future improvements, ending with `<!-- diffmate:publication=<publication-id> -->`.
  6. A success response moves the publication to `submitted` with the GitHub review ID. A definitive client error moves it to `rejected`. A network error, timeout, or server error moves it to `unknown`.
  7. Reconciliation is a `reconcile` job, enqueued by recovery and on demand from the TUI, for every `unknown` publication. diffmate lists my reviews on the PR and looks for the publication marker. If found, the publication becomes `submitted`. If not, it stays `unknown`, and the TUI offers to open the PR and to mark the publication `abandoned` after I have checked.
  8. Comment sync: for every `submitted` publication, diffmate lists the review's comments and links each inline finding to its comment by the finding marker. Sync only establishes links; thread statuses are read separately (see "Thread status"). Unlinked inline findings keep thread status `unknown`, and sync retries on its own schedule without another POST. Body findings are not synced.

## CON-queue-rules: Queue rules
- source: /Users/apaderin/dev/diffmate/SPEC.md (Queue rules)
- type: protocol
- content:
  - Source: PRs where my review is requested directly, found with the GitHub search qualifier `user-review-requested:@me`. Such a PR becomes a tracked target, subject to the discard rule.
  - Only tracked targets are reviewed automatically.
  - Draft PRs are skipped until they are marked ready for review.
  - Automatic runs are deduplicated by head SHA. A base branch change alone never queues a run. A manual re-run recomputes the merge-base.
  - Debounce: a new head SHA is queued after `debounce` (default 60 seconds) without further head changes.
  - A head change during a run does not cancel it. The run finishes, its draft becomes stale, and a run for the latest known head is queued after the debounce.
  - Retries: an automatic run that fails with `timeout`, `interrupted: crash`, or a transient `agent_error` (API overload, rate limit, server error, or network error, as reported in the result envelope) is retried on the same head up to `max_retries` times (default 2), after 1 minute, then 5 minutes, and 5 minutes for every later retry. A run with `interrupted: shutdown` is re-queued without using the budget. `interrupted: user_cancel`, `cli_error`, `invalid_output`, and any other `agent_error` are terminal and not retried.
  - Each new head gets its own retry budget. The remaining budget and the time of the next attempt are stored in SQLite, so restarts do not reset them.
  - When an automatic review gives up on a head, diffmate sends a macOS notification with the failure reason, and the queue view shows the failure.
  - At most one agent run at a time in the MVP.

## CON-commands: CLI commands
- source: /Users/apaderin/dev/diffmate/SPEC.md (Commands)
- type: api-contract
- content:
  - `diffmate review <pr-url> [--force]`: review one PR now. Without `--force` an existing successful run for the same head SHA is reused. Does not change tracking.
  - `diffmate status`: list targets with tracking, latest run, draft freshness with the time of the last head check, and publication state.
  - `diffmate watch`: poll and act as the runner in the background.
  - `diffmate tui`: queue view, draft view, decisions, edit in `$EDITOR`, publish, reconcile, abandon an `unknown` publication, discard a target.
  - `diffmate doctor`: check `gh` auth, Claude CLI version and flag support, storage paths, the runner lock, and the SQLite schema version.

## CON-configuration: Configuration
- source: /Users/apaderin/dev/diffmate/SPEC.md (Configuration)
- type: api-contract
- content: `~/.config/diffmate/config.toml`: poll interval, debounce, run timeout, `max_retries`.

## CON-output-schema: Review output JSON schema
- source: /Users/apaderin/dev/diffmate/schemas/review.schema.json (referenced by SPEC.md, Agent contract; the file is not a classified doc, it is included because the SPEC designates it as the output schema)
- type: schema
- content:
  ```json
  {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "diffmate review output",
    "type": "object",
    "additionalProperties": false,
    "required": ["status", "limitation", "findings", "future_improvements"],
    "properties": {
      "status": { "type": "string", "enum": ["findings", "no_findings"] },
      "limitation": { "type": ["string", "null"], "minLength": 1 },
      "findings": { "type": "array", "items": { "$ref": "#/$defs/finding" } },
      "future_improvements": { "type": "array", "maxItems": 2, "items": { "type": "string", "minLength": 1 } }
    },
    "$defs": {
      "finding": {
        "type": "object",
        "additionalProperties": false,
        "required": ["severity", "path", "line", "side", "title", "body"],
        "properties": {
          "severity": { "type": "string", "enum": ["high", "medium", "low"] },
          "path": { "type": ["string", "null"], "minLength": 1 },
          "line": { "type": ["integer", "null"], "minimum": 1 },
          "side": { "type": ["string", "null"], "enum": ["LEFT", "RIGHT", null] },
          "title": { "type": "string", "minLength": 1 },
          "body": { "type": "string", "minLength": 1 },
          "previous_finding_id": { "type": ["string", "null"], "minLength": 1 }
        },
        "anyOf": [
          { "properties": { "line": { "type": "integer" }, "side": { "type": "string" }, "path": { "type": "string" } } },
          { "properties": { "line": { "type": "null" }, "side": { "type": "null" } } }
        ]
      }
    }
  }
  ```
  (Whitespace compacted from the source file; keys and values unchanged.)
