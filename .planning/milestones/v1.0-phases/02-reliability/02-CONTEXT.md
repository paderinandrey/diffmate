# Phase 2: Reliability - Context

**Gathered:** 2026-10-27
**Status:** Ready for planning

<domain>
## Phase Boundary

A review survives being killed, interrupted with Ctrl-C or SIGTERM, repeated, or overtaken by a push, with no duplicate runs, orphaned worktrees, lost stop cause or unbounded retries, and the draft says whether it still matches the PR head. The phase adds stop-cause bookkeeping, recovery that keeps the stored cause, the automatic/manual job distinction, the retry budget and next-attempt time in SQLite, job attachment and `--force`, the run timeout, the first reads of `config.toml`, and draft freshness in `review` and `status`. SPEC.md slice 2 and ROADMAP.md Phase 2 (REL-01 to REL-13) fix the scope. Polling, tracking, debounce and notifications stay in Phase 3; the TUI stays in Phase 4.

</domain>

<decisions>
## Implementation Decisions

SPEC.md already fixes the behavior of this phase: interrupted causes and signals by role, recovery steps, the retry rules and delays, attachment and `--force`, the 15-minute timeout and SIGTERM then SIGKILL for the agent group, and the Ctrl-C messages. The decisions below cover only what SPEC.md leaves open.

### Stand-ins for `watch` and `tui`
- **D-01:** `diffmate watch` ships in this phase as a real, visible command that acts only as the runner: it waits for the lock with pauses instead of exiting, runs recovery, executes every job in the queue, and treats SIGTERM and SIGINT as `shutdown`. It does not poll GitHub; Phase 3 adds polling to the same loop. The REL-08 hand check runs on the release binary.
- **D-02:** A hidden command, `diffmate debug enqueue <url>`, enqueues an automatic `review` job or a `tui`-style re-run through the same enqueue and attach code the real sources will use, then exits without taking the lock. It is in the release binary and hidden from `--help`. It never touches GitHub beyond resolving the target and never publishes. Phases 3 and 4 replace its uses with real sources and may remove it.
- **D-03:** `watch` prints one timestamped line per event to stderr, for example `2026-10-27T12:00:00Z job 7 review owner/repo#1 started`, `... succeeded (findings: 2)`, `... failed (interrupted: shutdown), requeued`. It does not print findings; `review` and `status` show those. Phase 5 sends this output to a file through launchd.

### Head check and freshness
- **D-04:** The runner reads the PR head at job start (as SPEC.md requires for deduplication) and again after the agent finishes, and stores the last known head with the time it was checked. A push during a run therefore makes the draft `stale` as soon as the run ends (REL-02). A base branch move does not (REL-03).
- **D-05:** `status` makes one live read-only head request per target and shows freshness from it. It stays read-only: it opens SQLite with `Store::open_read_only` and does not store what it fetched. Only the runner writes the last known head. When the live request fails, `status` shows the stored head and its check time and says that the live check failed.
- **D-06:** Freshness is shown with relative time: `stale: head 9f3c2a1, checked 3m ago`, or `checked just now` after a successful live check.
- **D-07:** When the draft is already stale at the end of a `review` (a push landed during the run), `review` prints the findings as usual with a visible warning on the first line, for example `... at 5c59247: succeeded (findings: 1), stale: PR head is now 9f3c2a1`, and exits 0. The exit codes from Phase 1 D-04 (0, 1, 2) do not change.

### `config.toml`
- **D-08:** Durations are integers with the unit in the key name: `run_timeout_minutes = 15`, `debounce_seconds = 60`, `poll_interval_seconds = 60`, `max_retries = 2`. Parsing uses the `toml` crate with serde; no duration parser or extra crate. The new crate is explained in the PR. — **Reversibility:** costly — the key names become a file format the owner writes; renaming them later breaks existing config files.
- **D-09:** An unknown key, an invalid value (a duration of 0 or less, a negative `max_retries`) or broken TOML is a hard error naming the file, the line and the key. The command exits before enqueuing anything. A missing file or a missing key uses the SPEC.md default.
- **D-10:** `review` and `watch` read `config.toml` once at process start; changing it takes effect on the next start. `status` does not read it.
- **D-11:** All four SPEC.md settings are accepted and validated in this phase. Phase 2 uses `run_timeout_minutes` and `max_retries`; `poll_interval_seconds` and `debounce_seconds` are parsed and not yet read by any code.
- **D-12:** The default poll interval is 60 seconds. SPEC.md "Configuration" did not state it; the owner decided it on 2026-10-27 and SPEC.md now records it.

### Proof of "kill at any step"
- **D-13:** REL-01 is proven by named crash points at every runner step (enqueue, lock, recovery, revision, fetch, worktree, context files, agent start, agent running, store result, cleanup). A test starts the real binary, waits until it reports reaching the named point, sends SIGKILL, starts it again, and checks SQLite and the filesystem: no duplicate run, no orphaned worktree or run directory, no stale `git worktree` metadata, and `failed (interrupted: crash)` where a run had started. The hand check adds one or two `kill -9` during a live run.
- **D-14:** The crash points exist only in a test build behind a cargo feature. The release binary contains no crash-point code. The test build selects the point through an environment variable that exists only in that build; this is the one exception to the Phase 1 rule of no new environment variables.
- **D-15:** The crash-point tests use fake `claude` and `gh` and real `git` on a local repository, need no network or authentication, and run in the normal `mise run test` suite, the hk pre-commit hook and CI on ubuntu and macos.
- **D-16:** The REL-06 hand check uses the release binary: enqueue an automatic job with `debug enqueue`, start `watch`, `kill -9` it while the agent runs, and start it again, three times. `status` shows the remaining budget and the next attempt time between attempts, and after the last attempt the job is `failed`. The crash-point tests cover the same behavior systematically.

### Claude's Discretion
- The SQLite schema moves from version 1 to version 2 through an incremental migration that keeps existing data; the owner's database already holds runs from Phase 1 UAT. — **Reversibility:** one-way — the migration rewrites the owner's live `diffmate.db`; a mistake there loses Phase 1 runs, so the migration needs a test that starts from a version 1 database.
- How `status` words the retry budget and next attempt time for an automatic job.
- The bounded wait between SIGTERM and SIGKILL for the agent group, and the pause between lock retries in `watch`.
- How `watch` waits for new jobs when the queue is empty (polling SQLite on an interval), as long as it uses tokio time so tests run with paused time.
- The exact list and names of crash points, as long as every step in D-13 has one.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Behavior
- `SPEC.md` — source of truth: "Run" (failure kinds, interrupted causes, signals by role), "Draft freshness", "Job" (recovery of `running` jobs, attachment, deduplication), "Concurrency" and "Recovery", "One model for every process", "Agent contract" (timeout, process group, outcome classification), "Queue rules" (retries and budget), "Commands", "Configuration", and slice 2 "Done when".
- `.planning/REQUIREMENTS.md` — REL-01 to REL-13 and the required-property map.
- `.planning/ROADMAP.md` — Phase 2 success criteria and planning notes (the `watch` and `tui` stand-ins, the automatic/manual distinction, the retry budget in SQLite).

### Conventions and prior decisions
- `AGENTS.md` — project conventions, testing rules (paused tokio time, typed fakes, proptest, `#[ignore]` live checks) and verification commands (`CLAUDE.md` is a symlink to it).
- `.planning/phases/01-single-review/01-CONTEXT.md` — Phase 1 decisions, especially D-03 (minimal signal handler, agent PID/PGID on the run row) and D-04 (output and exit codes).
- `.planning/phases/01-single-review/01-RESEARCH.md` — verified facts on `flock`, process groups, the Claude CLI and git.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/store/schema.sql` and `src/store.rs` `migrate`: schema version 1 through `user_version`; `runs` already has `interrupted_cause` (`shutdown`, `user_cancel`, `crash`), `agent_pid` and `agent_pgid`. `jobs` has `enqueued_by_pid` and `enqueued_by_started_at` but no automatic/manual flag, no `--force`, no retry budget and no next attempt time. `targets` has no last known head.
- `src/recovery.rs` and `src/store/recovery.rs`: recovery already marks `running` runs `interrupted: crash`, fails their jobs and removes run directories; Phase 2 makes it keep a stored cause and apply the per-cause job rules.
- `src/lock.rs`: `try_acquire` and `acquire` with `LOCK_POLL` (500 ms) and a waiting callback; `review` prints the `waiting:` line through it.
- `src/review/signals.rs`: `until_done` turns SIGINT and SIGTERM into one stop flag; Phase 2 needs the cause per signal and per role.
- `src/agent/stop.rs`, `src/cancel.rs`, `src/runner/cleanup.rs`: process-group stop, cancellation racing and worktree cleanup from Phase 1.
- `src/store/readonly.rs` and `src/status.rs`: the read-only store and the `status` rows that gain freshness.
- `src/cli.rs`: clap derive with `Review { url }` and `Status`; `watch`, `debug enqueue` and `review --force` are added here.

### Established Patterns
- External processes (`git`, `gh`, `claude`) sit behind traits with hand-written fakes (`src/recovery/fake.rs`, the agent fakes); new seams follow the same pattern.
- Blocking SQLite work goes through `spawn_blocking`; timers and retries use paused tokio time in tests.
- Files are kept under the 100-line limit of the commit hook by splitting into submodules.

### Integration Points
- `src/runner.rs` `execute_review_job` is the place for the start-of-job head check, deduplication against an existing run, the after-agent head check and the timeout.
- `src/store/jobs.rs` holds enqueue; attachment and `--force` happen there in one transaction.
- `src/main.rs` and `src/app.rs` dispatch commands; `config.toml` is read here once per process.

</code_context>

<specifics>
## Specific Ideas

- Example lines the owner accepted: `stale: head 9f3c2a1, checked 3m ago` in `status`, a first `review` line ending `succeeded (findings: 1), stale: PR head is now 9f3c2a1`, and `watch` lines such as `2026-10-27T12:00:00Z job 7 review owner/repo#1 started`.
- The hand checks run on the release binary from `mise run build`, using `debug enqueue` and `watch`; only the crash-point tests use the feature build.

</specifics>

<deferred>
## Deferred Ideas

None. The discussion stayed within the phase scope.

</deferred>

---

*Phase: 02-reliability*
*Context gathered: 2026-10-27*
