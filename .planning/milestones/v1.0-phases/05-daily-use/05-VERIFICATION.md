---
phase: 05-daily-use
verified: 2026-11-11T17:24:00Z
status: passed
score: 12/13 must-haves verified
covered_files:
  - ".planning/phases/05-daily-use/05-01-PLAN.md"
  - ".planning/phases/05-daily-use/05-01-SUMMARY.md"
  - ".planning/phases/05-daily-use/05-02-PLAN.md"
  - ".planning/phases/05-daily-use/05-02-SUMMARY.md"
  - ".planning/phases/05-daily-use/05-03-PLAN.md"
  - ".planning/phases/05-daily-use/05-03-SUMMARY.md"
  - "docs/install.md"
  - "src/agent/claude.rs"
  - "src/agent/claude/contract.rs"
  - "src/cli.rs"
  - "src/doctor.rs"
  - "src/doctor/claude.rs"
  - "src/doctor/gh_auth.rs"
  - "src/doctor/lock_line.rs"
  - "src/doctor/schema.rs"
  - "src/doctor/scratch.rs"
  - "src/doctor/storage.rs"
  - "src/doctor/which.rs"
  - "src/gh.rs"
  - "src/main.rs"
  - "src/store.rs"
  - "tests/doctor.rs"
  - "tests/doctor/fresh.rs"
  - "tests/doctor/held.rs"
  - "tests/doctor/lock_role.rs"
  - "tests/doctor/pass.rs"
  - "tests/doctor/schema.rs"
  - "tests/doctor/storage.rs"
  - "tests/doctor/tools.rs"
  - "tests/doctor/wal.rs"
covered_digest: "v4:sha256:82d26aba6aee5cb8b6f4a95cb552ffb0f0e5c080be538c7da0bf2af40cb131f0"
behavior_unverified: 1
overrides_applied: 0
behavior_unverified_items:
  - truth: "watch survives a reboot"
    test: "Reboot, log in to the GUI session, then run diffmate doctor."
    expected: "Stdout contains the line runner lock: ok — held by watch (em dash U+2014) and the process exits 0."
    why_human: "D-08 forbids automating a reboot. docs/install.md describing KeepAlive and RunAtLoad is not evidence that the GUI agent came back. The live plist stays in dotfiles, outside this repo."
human_verification:
  - test: "Reboot, log in to the GUI session, then run diffmate doctor."
    expected: "Stdout contains the line runner lock: ok — held by watch (em dash U+2014) and the process exits 0."
    why_human: "D-08. LaunchAgent survival across reboot runs in the GUI session after login. The repository documents the agent and does not reboot."
---

# Phase 5: Daily use Verification Report

**Phase Goal:** diffmate runs unattended day to day: `watch` is started by launchd and survives a reboot, and `doctor` tells me exactly what is wrong when something is.
**Verified:** 2026-11-11T17:24:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | `watch` survives a reboot. | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | `docs/install.md` sets `KeepAlive` and `RunAtLoad` and states the manual after-login check. `src/watch/wait.rs` waits on the runner lock instead of exiting. `src/watch.rs` maps shutdown during that wait to `Ok(())`, and `src/watch/run.rs` returns `Ok(())` on SIGTERM, so `watch::run` exits 0. Phase 5 commits do not edit `src/watch`. None of that is a reboot. |
| 2 | `doctor` explains every failed check. | ✓ VERIFIED | `doctor::run` always runs all five checks, prints each line, and returns exit 1 if any failed. Failure text names the action: `gh auth login`, PATH, `chmod 700`, `chmod 600`, run `watch` or `review` once for a missing lock, `SchemaOutdated` (`run diffmate review once`), and `SchemaVersion` (database newer than this binary). Doctor integration tests assert those lines. |
| 3 | `diffmate doctor` prints exactly five stdout lines, in order gh auth, claude, storage, runner lock, schema, and exits 0 only when every line passed. | ✓ VERIFIED | `src/doctor.rs` builds that array and prints after every check. `fresh_home_prints_five_lines_and_creates_nothing` expects exit 1 with those prefixes. `free_lock_and_no_database_exits_zero` and `held_watch_lock_and_schema_five_exit_zero` expect exit 0 and the five lines. |
| 4 | A missing state directory, cache directory, config.toml, or database is ok. Doctor creates none of them. A broken config.toml fails the storage line and the other four lines still print. | ✓ VERIFIED | `storage::directory` treats `NotFound` as ok. `config::load` returns defaults when the file is missing and does not create it. `schema::stat_exists` decides absence before any SQLite open. `main` preloads `config.toml` only for review, watch, and tui. `fresh_home_prints_five_lines_and_creates_nothing` and `broken_config_still_prints_five_lines` both passed. |
| 5 | A readable runner.lock that nobody holds is ok and is not reported as held. Doctor does not take the lock. | ✓ VERIFIED | `lock_line::inspect` uses `File::open` plus `fcntl_getlk`. A free lock returns `runner lock: ok — not held` before the file text is read. `free_lock_and_no_database_exits_zero` writes `watch\n`, drops the flock, expects that line, and `try_lock`s the file afterward. |
| 6 | The schema line for a missing database file is `schema: ok — no database yet`. | ✓ VERIFIED | `schema::check` returns that string when `metadata` is `NotFound`. `free_lock_and_no_database_exits_zero` asserts the exact line and that `diffmate.db` was not created. |
| 7 | A lock held by watch is `runner lock: ok — held by watch`, and the caller's flock is still held after doctor exits. | ✓ VERIFIED | `held_watch_lock_and_schema_five_exit_zero` and `held_review_tui_and_unreadable_role_keep_the_flock` keep the parent flock and assert `TryLockError::WouldBlock` after doctor exits 0. |
| 8 | `user_version` 5 passes. An older database, including `user_version` 0, fails with the existing sentence to run `diffmate review` once. A newer database fails because this binary is older than the database. Doctor does not migrate. | ✓ VERIFIED | `SCHEMA_VERSION` is still 5. `classified` copies `Error::SchemaOutdated` and `Error::SchemaVersion`. Schema tests keep the file bytes and the pragma unchanged for versions 0, 1, and 6. Version 5 exits 0 in the held-lock test. `wal_user_version_five_adds_no_directory_entry` sees no new `-wal` or `-shm`. |
| 9 | A directory that exists and is not mode 0700 fails the storage line with that path and `chmod 700`, and doctor does not chmod it. | ✓ VERIFIED | `storage::mode_owner` fails when mode is not `0o700` or the uid is not the process uid. `loose_state_dir_and_file_cache_share_one_storage_line` leaves the state directory at `0755` and the cache path a file, on one storage line. |
| 10 | A missing gh or claude binary fails only the line that needs it and names PATH. Two planted failures are both visible. | ✓ VERIFIED | `which` gates each tool. `missing_gh_and_claude_name_path_without_a_sixth_line` puts `/usr/bin:/bin` on both of the first two lines and still has five lines. `old_claude_does_not_run_the_flag_probe` returns on the version error before `check_flags`. |
| 11 | The only install document is `docs/install.md`. The repo does not add a README or a plist. | ✓ VERIFIED | `git ls-files` lists `docs/install.md` and no `README*` or `*.plist`. The page says the live plist stays in dotfiles. |
| 12 | The example agent is label `dev.diffmate.watch`, runs the absolute binary plus `watch`, `KeepAlive` true, `RunAtLoad` true, `ThrottleInterval` 10, and `Umask` the string `0077`. | ✓ VERIFIED | The fenced plist in `docs/install.md` has those keys. `Umask` is `<string>0077</string>`. `ProgramArguments` is `/Users/YOU/.local/bin/diffmate` then `watch`. Prose states this machine's PATH as `/opt/homebrew/bin:/Users/apaderin/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin`. No `WorkingDirectory`, no `ProcessType`, and no token. `$UID` is only in the shell commands. |
| 13 | `bootstrap`, `kickstart -k`, and `bootout` are separate commands. The log directory is created mode 0700 before bootstrap, and the logs are emptied without a rotator. | ✓ VERIFIED | `docs/install.md` has separate Install, Update, and Remove sections (`launchctl bootstrap`, `launchctl kickstart -k`, `launchctl bootout`). `mkdir -p -m 0700` is before bootstrap. The log section says both files grow and there is no rotator. |

The plan truth "After login the owner looks for `diffmate doctor` printing `runner lock: ok — held by watch`" restates truth 1 (D-08, OPS-04). It is not scored a second time. `git diff` for the phase commits does not touch `src/watch`.

**Score:** 12/13 truths verified (1 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `src/doctor.rs` | Doctor entry that always prints five lines | ✓ VERIFIED | `run` awaits gh, claude, storage, lock, schema, then prints. Wired from `Command::Doctor` in `src/main.rs`. |
| `src/cli.rs` | Doctor subcommand | ✓ VERIFIED | `Command::Doctor` exists and is dispatched. |
| `src/doctor/lock_line.rs` | Read-only flock query | ✓ VERIFIED | `fcntl_getlk` on a read-only `File`. No `try_acquire` and no `lock::holder`. |
| `src/doctor/schema.rs` | Schema report without migrating | ✓ VERIFIED | Substantive. Reports `PRAGMA user_version` through `file:…?mode=ro&immutable=1`. `verify.artifacts` reported `Missing pattern: open_read_only`. That pattern is the stale plan 01 string. D-03 forbids `Store::open_read_only` here because a read-only WAL open creates sidecars. Absence of that call is required. |
| `tests/doctor/fresh.rs` | Fresh home and broken config still print five lines | ✓ VERIFIED | Both tests passed in the doctor binary run. |
| `tests/doctor/held.rs` | Held watch lock and schema version 5 | ✓ VERIFIED | `held_watch_lock_and_schema_five_exit_zero` passed. |
| `tests/doctor/schema.rs` | Older, newer, and zero schema versions stay unchanged | ✓ VERIFIED | All three schema tests passed. |
| `docs/install.md` | Launchd install, update, and remove text | ✓ VERIFIED | Contains `dev.diffmate.watch` and the three launchctl commands. |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `src/main.rs` | `src/doctor.rs` | `Command::Doctor` calls `doctor::run` and is outside the config preload match | WIRED | Preload match is only `Review`, `Watch`, and `Tui`. Doctor still reaches `doctor::run`. |
| `src/doctor/schema.rs` | immutable SQLite URI | Schema age is read with `mode=ro&immutable=1`, not `Store::open_read_only` | WIRED | D-03. `verify.key-links` matched `open_read_only` only because that symbol still exists in `src/store/readonly.rs`. `schema.rs` does not call it. `wal_user_version_five_adds_no_directory_entry` passed. |
| `src/doctor/claude.rs` | `src/agent/probe/run.rs` | `check_version`, then `check_flags` only if version succeeded | WIRED | Version `Err` returns before `flags`. `old_claude_does_not_run_the_flag_probe` passed. Neither function constructs `ClaudeCli`. |
| `src/doctor/gh_auth.rs` | `src/gh.rs` | `gh auth status` through `GhCli::command` and `run_tool` | WIRED | Argument vector, `env_clear`, no shell. |
| `tests/doctor/held.rs` | `src/doctor/lock_line.rs` | Binary prints held-by-watch while the parent flock still blocks `try_lock` | WIRED | Test passed. |
| `src/doctor/schema.rs` | `src/error.rs` | `SchemaOutdated` and `SchemaVersion` `Display` are the schema failure text | WIRED | Older text includes ``run `diffmate review` once``. Newer text is `database schema version {n} is newer than this diffmate supports`. |
| `docs/install.md` | `src/watch.rs` | Doc relies on the existing wait and SIGTERM exit 0 and does not edit watch | WIRED | Wait loop and shutdown `Ok(())` are in the tree. Phase 5 did not modify `src/watch`. |
| `docs/install.md` | `~/Library/LaunchAgents/dev.diffmate.watch.plist` | Example is copied by the owner; the plist is not a repo file | WIRED | Path is in the doc. `git ls-files` has no plist. Absence from the repo is expected. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `src/doctor/gh_auth.rs` | gh auth line | `gh auth status` stdout or the spawn error | Yes | ✓ FLOWING |
| `src/doctor/claude.rs` | claude line | `claude --version`, then the flag probe | Yes | ✓ FLOWING |
| `src/doctor/storage.rs` | storage line | Directory metadata and `config::load` | Yes | ✓ FLOWING |
| `src/doctor/lock_line.rs` | runner lock line | `fcntl_getlk` and, only if held, the lock file text | Yes | ✓ FLOWING |
| `src/doctor/schema.rs` | schema line | `metadata` for absence, else `PRAGMA user_version` | Yes | ✓ FLOWING |
| `docs/install.md` | install text | Static document | n/a | n/a |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Doctor binary covers the five-line report, failure lines, held lock, schema age, and WAL sidecars | `mise run --force test -- -E 'binary(doctor)' --no-tests=fail` | 14 tests run: 14 passed, 0 skipped (nextest run `48789b2e-3989-490e-a540-9371414dd66e`) | ✓ PASS |
| `watch` is alive after a reboot | not run | D-08 forbids a reboot from this verification | ? SKIP |

### Probe Execution

No phase plan or success criterion declares `scripts/*/tests/probe-*.sh`. No probe was required.

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| none | n/a | n/a | n/a |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| OPS-01 | 05-03 | A launchd agent runs `watch`. Its plist lives in dotfiles, outside this repo; this repo documents what it needs and how to install it. | ✓ SATISFIED | `docs/install.md` documents the GUI agent. No plist is committed. The live plist is outside the repo by decision, not a gap. |
| OPS-02 | 05-03 | Install instructions exist and are enough to set diffmate up on a fresh macOS machine. | ✓ SATISFIED | Build, copy to `~/.local/bin/diffmate`, `mkdir -p -m 0700`, example plist, and separate bootstrap, kickstart, and bootout sections. |
| OPS-03 | 05-01, 05-02 | `diffmate doctor` checks `gh` auth, Claude CLI version and flag support, storage paths, the runner lock, and the SQLite schema version. | ✓ SATISFIED | Five checks in that order in `src/doctor.rs`, exercised by the doctor binary tests. |
| OPS-04 | 05-03 | `watch` survives a reboot. | ? NEEDS HUMAN | Documented `KeepAlive` / `RunAtLoad` plus existing wait and SIGTERM exit 0. Survival after reboot is truth 1 and is not proven here. |
| OPS-05 | 05-01, 05-02 | `doctor` explains every failed check. | ✓ SATISFIED | Failed checks still print five lines, and each failure line names what to do. Covered by fresh, schema, storage, lock, and tools tests. |

No OPS id mapped to Phase 5 in `REQUIREMENTS.md` is missing from plan frontmatter. OPS-01 and OPS-02 appear only on plan 03. OPS-04 appears only on plan 03. OPS-03 and OPS-05 appear on plans 01 and 02.

### Prohibitions

Plan frontmatter marks these `verification: judgment`. They are not a silent pass. Static search of `src/doctor` and `src/doctor.rs` found no `lock::holder`, `try_acquire`, `Store::open`, `migrate::run`, `spawn::invoke`, `ClaudeCli::new`, or `open_read_only`. `Cargo.toml` and `Cargo.lock` are unchanged in the phase commits. `SCHEMA_VERSION` remains 5.

| Statement | Evidence | Disposition |
| --- | --- | --- |
| MUST NOT take the runner lock. | Read-only `fcntl_getlk`. Held-lock tests still observe `WouldBlock` after doctor exits. Free-lock test can `try_lock` afterward. | Holds. Tests enforce it. |
| MUST NOT create or migrate the database. | Absence is a `metadata` check. Existing files use an immutable read-only URI. Schema tests compare bytes and `user_version` before and after. Fresh-home test asserts no database file. | Holds. Tests enforce it. |
| MUST NOT commit a plist, a token, or a README. | `git ls-files` has neither a plist nor a README. `docs/install.md` says not to put a token in the plist and contains no credential. | Holds. |

`unverified-prohibition — human review recommended` does not apply: each judgment item has enforcing tests or a repository listing, not an empty check. They are not extra human steps.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `src/doctor/lock_line.rs` | 30 | `panic::set_hook` and `catch_unwind` around `fcntl_getlk` | ℹ️ Info | Comment names the rustix debug assert when macOS reports a BSD flock pid of -1. Debug doctor tests still distinguish a free lock from a held lock, so this did not collapse those results. |
| none | n/a | `TBD`, `FIXME`, or `XXX` in phase files | none | No unreferenced debt marker. |

### Human Verification Required

### 1. Reboot survival

**Test:** Reboot, log in to the GUI session, then run `diffmate doctor`.
**Expected:** Stdout contains `runner lock: ok — held by watch` (em dash, U+2014) and the process exits 0.
**Why human:** D-08. A LaunchAgent in `~/Library/LaunchAgents` runs after GUI login, not at the login window. The install page describing that check is not evidence the agent survived. Do not automate a reboot.

### Gaps Summary

No product gap. `doctor` is wired, read-only, and covered by 14 passing integration tests. The launchd agent is documented in `docs/install.md` and is not committed, which is the locked shape for OPS-01. The phase goal is not fully achieved until a person sees `watch` holding the runner lock after a reboot. That item is human verification, not a missing implementation.

---

_Verified: 2026-11-11T17:24:00Z_
_Verifier: Claude (gsd-verifier)_
