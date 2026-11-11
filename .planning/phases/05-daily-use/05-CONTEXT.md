# Phase 5: Daily use - Context

**Gathered:** 2026-11-11
**Status:** Ready for planning

<domain>
## Phase Boundary

`watch` is started by a launchd agent, comes back after a reboot, and comes back if it exits while the owner is logged in. The live plist stays in the owner's dotfiles, outside this repo. This repo adds `docs/install.md` and `diffmate doctor`. SPEC.md slice 5, "Commands", and the `watch` lock and SIGTERM sentences in "One model for every process" fix the scope. `watch` already waits for the runner lock and already exits 0 on SIGTERM while waiting. Phase 2 and Phase 3 stay unaccepted. This phase does not call the Claude API. The SQLite `user_version` stays 5.

</domain>

<decisions>
## Implementation Decisions

SPEC.md already names the `doctor` checks, says the plist lives in dotfiles, and says `watch` waits for the lock and exits cleanly on SIGTERM. The decisions below cover only the report, the install page, the log files, and the path launchd runs.

### Doctor report
- **D-01:** One line per check. A failure names what to do. Exit 0 when every check passes, otherwise 1.
- **D-02:** A held runner lock is ok. The line names the holder (`watch`, `review`, or `tui`). A missing or unreadable lock file is a failure that says what to do. `doctor` does not take the lock.
- **D-03:** `doctor` only reports the schema. It does not write and it does not migrate. `user_version` 5 is ok. An older database fails with the existing sentence that says to run `diffmate review` once. A newer database fails because this binary is older than the database. No database file is ok (`no database yet`) and `doctor` does not create it.
- **D-04:** A missing state directory, cache directory, or `config.toml` is ok. The directories appear on the first `review` or `watch`. A missing config file uses the SPEC.md defaults. `doctor` creates nothing. An existing directory that is not writable or is not mode `0700`, or a broken `config.toml`, fails with the path and the fix.

### Install text
- **D-05:** The instructions live only in `docs/install.md`. This phase does not add a README. The live plist stays in dotfiles.
- **D-06:** `docs/install.md` contains a full example plist in a code block, with path placeholders. No `.plist` file is committed.
- **D-07:** `KeepAlive` is true, `RunAtLoad` is true, and `ThrottleInterval` is 10 seconds. The agent lives in `~/Library/LaunchAgents`. Stop it with `launchctl bootout`. The example names these keys. — **Reversibility:** costly — the owner copies these keys into the live agent in dotfiles, so a later change has to edit that plist and boot the job out.
- **D-08:** The reboot check in the doc is manual. After login, `diffmate doctor` prints `runner lock: ok — held by watch` and exits 0. This phase does not reboot the machine.

### Watch log
- **D-09:** `StandardErrorPath` is `~/Library/Logs/diffmate/watch.log`. `StandardOutPath` is `~/Library/Logs/diffmate/watch.out.log`. launchd creates the files. `diffmate` does not open a log itself. Phase 2 D-03 still holds: `watch` prints one timestamped line per event to stderr. — **Reversibility:** costly — the installed agent and the owner both look at these two paths.
- **D-10:** The example plist sets `Umask` to `0077`. The doc creates the directory with `mkdir -p -m 0700 ~/Library/Logs/diffmate` before `bootstrap`.
- **D-11:** Both files grow. There is no rotator in `diffmate` and no `newsyslog` example. The doc says how to empty them: `bootout`, truncate or delete the files, then `bootstrap`.

### Binary path
- **D-12:** `ProgramArguments` is the absolute path of `~/.local/bin/diffmate`. The doc says `mise run build`, then `cp target/release/diffmate ~/.local/bin/diffmate`. — **Reversibility:** costly — the live agent points at this path.
- **D-13:** The example `PATH` is the directories from `which git`, `which gh`, and `which claude`, then `/usr/bin:/bin:/usr/sbin:/sbin`. On this machine those directories are `~/.local/bin` and `/opt/homebrew/bin`. Do not copy the full shell `PATH`. Do not put tokens in the plist. — **Reversibility:** costly — the value lives in the dotfiles plist, and a wrong `PATH` makes `watch` unable to find `git`, `gh`, or `claude`.
- **D-14:** The label is `dev.diffmate.watch`. The file is `~/Library/LaunchAgents/dev.diffmate.watch.plist`. `bootstrap` and `bootout` use `gui/$UID/dev.diffmate.watch`. — **Reversibility:** costly — changing the label means `bootout` of the old job.
- **D-15:** An updated binary is picked up with `launchctl kickstart -k gui/$UID/dev.diffmate.watch`. The first install is `bootstrap`. Removing the agent is `bootout`. The doc lists these three commands separately.

### Claude's Discretion
- The English wording of each `doctor` line, as long as D-01 through D-04 hold and a failure names the action.
- The check order follows SPEC.md "Commands": `gh` auth, Claude CLI version and flag support, storage paths, the runner lock, then the schema version.
- A missing `git`, `gh`, or `claude` binary fails the check that needs it, and the line names the `PATH` entry. Reuse the existing Claude flag probe. Do not add a sixth check.
- The example plist writes absolute paths. launchd does not expand `~`.
- Omit `WorkingDirectory`. A GUI agent already has `HOME`. `doctor` and `watch` resolve paths from `HOME`, as `src/app.rs` does.
- `user_version` stays 5. No new crate.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Behavior
- `SPEC.md` — "Commands" (`doctor` checks), "One model for every process" (`watch` waits for the lock and exits on SIGTERM), "Configuration", "Storage", and slice 5 "Daily use".
- `.planning/REQUIREMENTS.md` — OPS-01 to OPS-05.
- `.planning/ROADMAP.md` — Phase 5 goal, success criteria, and the note that the plist lives in dotfiles and the reboot check is manual.
- `.planning/phases/02-reliability/02-CONTEXT.md` — D-03: `watch` prints one timestamped line per event to stderr, and Phase 5 sends that output to a file through launchd.

### Code
- `src/cli.rs` — `Command` has `Review`, `Status`, `Watch`, `Tui`, and a hidden `Debug`. `Doctor` is not there yet.
- `src/watch.rs` — SIGTERM while waiting for the lock returns success. Exit 0 is what launchd needs.
- `src/watch/wait.rs` — retries the lock once a second and prints one waiting line.
- `src/watch/events.rs` — timestamped lines go to stderr.
- `src/paths.rs` — state under `~/Library/Application Support/diffmate`, cache under `~/Library/Caches/diffmate`, `runner.lock`, `diffmate.db`. Directories are created mode `0700`.
- `src/app.rs` — home comes from `HOME`. Config file is `~/.config/diffmate/config.toml`.
- `src/store/migrate.rs` — `SCHEMA_VERSION` is 5.
- `src/store/readonly.rs` — distinguishes the current schema, an older one, and a newer one without being the writer `doctor` should avoid.
- `src/error.rs` — the older-schema sentence tells the owner to run `diffmate review` once.
- `src/agent/probe/run.rs` — the Claude CLI flag probe `doctor` reuses.
- `src/agent/claude/contract.rs` — `AUTH_ENV` is the only environment passed onward (`PATH`, `HOME`, locale, and the Claude auth variables).
- `AGENTS.md` — verification commands, no new crate without a concrete need, and tests via `mise run test`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/agent/probe/run.rs` already checks Claude CLI flag support before a review. `doctor` calls that check and does not start an agent.
- `src/store/readonly.rs` and `src/error.rs` already separate schema 5, an older database, and a newer database. The older-schema words are the ones D-03 reuses.
- `src/lock.rs` can report the holder (`watch`, `review`, `tui`) without taking the lock. `src/watch/wait.rs` already formats that holder.
- `src/config` already fails a broken `config.toml` with the file, the line, and the key (Phase 2 D-09). A missing file uses the defaults.

### Established Patterns
- Commands return `ExitCode`. `watch` prints to stderr through `events::print`. `doctor` stays a read-only command: it does not take `runner.lock`, migrate, or create directories.
- Subprocesses use `tokio::process` and an argument vector. The child environment is the `AUTH_ENV` allowlist, not the full shell environment.
- `user_version` is 5. This phase does not add a migration.

### Integration Points
- Add `Doctor` on `Command` in `src/cli.rs` and dispatch it next to `Watch`.
- `docs/install.md` is the only new document. The plist, the log directory, and `~/.local/bin/diffmate` are created on the machine, not in the repo.

</code_context>

<specifics>
## Specific Ideas

- The reboot line the owner looks for is `runner lock: ok — held by watch`.
- On this machine `git` and `gh` are in `/opt/homebrew/bin` and `claude` is in `~/.local/bin`. The example `PATH` starts with those two directories, written as absolute paths, then `/usr/bin:/bin:/usr/sbin:/sbin`.
- The three launchctl commands in the doc are `bootstrap` (install), `kickstart -k` (new binary), and `bootout` (remove), all with `gui/$UID/dev.diffmate.watch`.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 5-Daily use*
*Context gathered: 2026-11-11*
