---
phase: 01-single-review
plan: 03
subsystem: testing
tags: [rust, black-box-tests, fake-cli, git-shim, fifo, tempfile]

requires:
  - phase: 01-single-review
    provides: "01-RESEARCH.md verified CLI behavior (version line, probe error text, refs/pull/N/head layout)"
provides:
  - "Sandbox: temporary HOME, scrubbed environment, fake claude and gh on PATH, git shim, diffmate Command for CARGO_BIN_EXE_diffmate"
  - "FakeClaude: file-configured version, probe, run; per-call argv/env/cwd/stdin/context log; FIFO hold of the Nth run; background child"
  - "FakeGh: prefix routes with stdout, stderr and exit code, call log, pr_view_json"
  - "Upstream: local bare repository whose PR head exists only under refs/pull/N/head"
affects: [01-04, 01-05, 01-06, 01-07, 01-08, 01-09, 01-10, 01-11, 01-12]

actuals:
  tokens: 5600
  tasks: 2
  commits: 2

plan_head_before: c2eee2c28df6791950588613ac9997c6f270d0a4
plan_head_after: f2ea5942ee994b1eeeaf333ee42c40600b01e3f9
commits: 2

tech-stack:
  added: [tempfile 3.27.0 (dev-dependency)]
  patterns:
    - "Fakes are POSIX sh scripts with a state directory embedded in single quotes, configured by writing files"
    - "Call log numbering by atomic mkdir, no counter file"
    - "FIFO handshake instead of sleeps to hold a fake agent open"
    - "git shim execs the absolute real git with -c url.<upstreams>.insteadOf=https://github.com/"

key-files:
  created:
    - tests/harness.rs
    - tests/support/mod.rs
    - tests/support/envelope.rs
    - tests/support/fake_call.rs
    - tests/support/fake_claude.rs
    - tests/support/fake_claude.sh
    - tests/support/fake_gh.rs
    - tests/support/fake_gh.sh
    - tests/support/upstream.rs
    - tests/support/sandbox.rs
  modified:
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "Support code is split into envelope.rs and fake_call.rs behind pub use re-exports in fake_claude.rs, so the documented paths (support::fake_claude::success_envelope, FakeCall, Hold) hold while every file stays within the 100-line hook limit"
  - "git_in (the controlled git runner for fixture repos) lives in support/mod.rs; real_git stays in upstream.rs as documented"
  - "mod.rs allows dead_code and unused_imports: every test binary compiles the whole module and uses part of it"

patterns-established:
  - "Later plans configure fakes by calling set_run/set_probe/route and assert on calls(); they never edit tests/support"
  - "Sandbox::diffmate never calls env_clear, so LLVM_PROFILE_FILE and other coverage variables reach the instrumented binary; GIT_*, CLAUDE*, ANTHROPIC_*, GH_*, GITHUB_* and XDG_CONFIG_HOME are removed"

requirements-completed: [REV-02]

coverage:
  - id: D1
    description: "Fake claude answers --version, probe and run with configurable output and exit code and logs argv, env, cwd, stdin and a copy of the --add-dir directory"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/harness.rs#fake_claude_answers_version_probe_and_run"
        status: pass
    human_judgment: false
  - id: D2
    description: "Fake claude holds the Nth run on a FIFO handshake and starts a background child whose PID is logged"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/harness.rs#fake_claude_holds_run_and_spawns_child"
        status: pass
    human_judgment: false
  - id: D3
    description: "Fake gh answers by argument prefix from route files, fails with a named error for unknown calls and logs argv"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/harness.rs#fake_gh_routes_by_prefix"
        status: pass
    human_judgment: false
  - id: D4
    description: "Upstream builder publishes a PR head only under refs/pull/N/head and the git shim resolves https://github.com/OWNER/REPO.git to it"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/harness.rs#upstream_pr_head_exists_only_under_pull_ref"
        status: pass
      - kind: integration
        ref: "tests/harness.rs#git_shim_resolves_github_remote"
        status: pass
    human_judgment: false
  - id: D5
    description: "Sandbox state and cache paths follow SPEC under a temporary HOME and the binary starts with a scrubbed environment"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/harness.rs#sandbox_uses_spec_paths"
        status: pass
    human_judgment: false

duration: 9min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 03: Black-box test harness Summary

**Sandbox with temporary HOME, file-configured fake claude and gh, a github.com-to-local git shim and a bare-repository builder that publishes PR heads only under refs/pull/N/head**

## Performance

- **Duration:** 9 min
- **Started:** 2026-10-20T15:50:00Z
- **Completed:** 2026-10-20T15:59:00Z
- **Tasks:** 2
- **Files modified:** 12 (10 created, Cargo.toml and Cargo.lock modified)

## Accomplishments
- Fake `claude` and `gh` are POSIX sh scripts configured per test through files in a state directory. The claude fake logs every call, copies the `--add-dir` context, can hold its Nth run on FIFOs and can start a background `sleep` child, so lock, recovery and signal tests synchronize on events.
- `Upstream` builds a bare repository at `UPSTREAMS/OWNER/REPO.git`; `pr_head` pushes only to `refs/pull/N/head`, and the head is verified not to be an ancestor of `main`. The `bin/git` shim maps `https://github.com/` to that directory with `insteadOf`.
- `Sandbox::diffmate()` runs the binary under test with a temporary HOME, PATH of the sandbox bin followed by `/usr/bin:/bin`, and the inherited `GIT_*`, `CLAUDE*`, `ANTHROPIC_*`, `GH_*`, `GITHUB_*` and `XDG_CONFIG_HOME` removed without `env_clear`.
- Six harness self-tests pass; the full suite (7 tests), `mise run lint`, `mise run deps:check` and `mise run vuln` pass.

## Task Commits

1. **Task 1: Fake claude and fake gh driven by files** - `0812155` (test)
2. **Task 2: Upstream builder, git shim and sandbox** - `f2ea594` (test)

**Plan metadata:** committed separately (docs: complete plan)

## Files Created/Modified
- `tests/support/mod.rs` - write_executable, mkfifo, call_numbers, exec, signal, git_in
- `tests/support/fake_claude.rs`, `fake_claude.sh` - FakeClaude, constants, re-exports of FakeCall, Hold and the JSON builders
- `tests/support/fake_call.rs` - FakeCall parsing and Hold (FIFO handshake)
- `tests/support/envelope.rs` - success_envelope, finding_json, output_json
- `tests/support/fake_gh.rs`, `fake_gh.sh` - FakeGh and pr_view_json
- `tests/support/upstream.rs` - real_git and Upstream
- `tests/support/sandbox.rs` - Sandbox
- `tests/harness.rs` - six self-tests
- `Cargo.toml`, `Cargo.lock` - tempfile 3.27.0 dev-dependency (newest release at least 3 days old; confirmed against crates.io)

## Decisions Made
- Split support code into extra files (envelope.rs, fake_call.rs) and moved `git_in` into mod.rs to respect the 100-line-per-file commit hook; public paths from the plan interface are preserved through re-exports.
- Numbered call directories with an atomic `mkdir` loop instead of a counter file, so concurrent invocations cannot share a number.
- `set_probe` appends a newline to the stderr text; `set_run` writes stdout exactly as given.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Files split to satisfy the 100-line commit hook**
- **Found during:** Task 1 and Task 2
- **Issue:** fake_claude.rs with the JSON builders and FakeCall/Hold was about 170 lines after rustfmt; upstream.rs was 119 lines. The large-files hook rejects a file that gains more than 100 lines.
- **Fix:** Added tests/support/envelope.rs and tests/support/fake_call.rs, re-exported through fake_claude.rs; moved `git_in` into tests/support/mod.rs.
- **Files modified:** tests/support/*.rs
- **Verification:** `wc -l tests/support/*.rs` maximum is 90; commit hooks passed.
- **Committed in:** 0812155, f2ea594

**2. [Rule 3 - Blocking] mod.rs declared sandbox and upstream only in Task 2**
- **Found during:** Task 1
- **Issue:** Task 1 declares all four submodules in the plan, but sandbox and upstream do not exist until Task 2, so Task 1 would not compile.
- **Fix:** Task 1 declared the modules it creates; Task 2 added `pub mod sandbox; pub mod upstream;`.
- **Committed in:** f2ea594

**3. [Rule 3 - Blocking] unused_imports allowance and shared helpers in mod.rs**
- **Found during:** Task 1
- **Issue:** The `pub use` re-exports warn as unused imports in a test binary that uses only part of them (warnings are denied). tests/harness.rs would also exceed the hook limit with its helpers inline.
- **Fix:** The inner attribute is `#![allow(dead_code, unused_imports)]`; `exec` and `signal` helpers live in mod.rs.
- **Committed in:** 0812155

---

**Total deviations:** 3 auto-fixed (3 blocking)
**Impact on plan:** Structure only; every interface in the plan is available at its documented path. The plan's mod.rs summary line lists `call_numbers`, `exec`, `signal` and `git_in` as extra helpers.

## Issues Encountered
- tests/harness.rs is 161 lines in total (97 lines added in Task 1, 64 in Task 2). The hook limit is per commit, so both commits pass; the plan's 100-line acceptance check covers tests/support only.
- `file://` upstreams and the shim were exercised on macOS only; the Linux CI leg has not run. The scripts use only POSIX sh, `mkfifo`, `cp -R`, `sleep` and `kill`.
- Plan 01-05's check that the fake's probe text equals the spike's recorded text was not done here (planned for 01-05). `PROBE_OK` is the text from 01-RESEARCH.md.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Threat Flags

None. The fakes and shim are written only inside the sandbox TempDir bin directory (T-01-08); `Sandbox::diffmate` removes the inherited variable families (T-01-07); the spawn_child test kills its `sleep` child (T-01-09); `mise run vuln` passed for the tempfile addition (T-01-SC).

## Next Phase Readiness
- Plans 01-04 onward can write black-box tests against `Sandbox` without editing tests/support.
- Plan 01-01 (checkpoint) and 01-02 (owner fixtures) were not touched.

## Self-Check: PASSED

- tests/harness.rs and tests/support/{mod,envelope,fake_call,fake_claude,fake_gh,upstream,sandbox}.rs and the two .sh files exist.
- Commits 0812155 and f2ea594 are ancestors of HEAD.
- `mise run --force test` (7 passed), `mise run --force lint`, `mise run deps:check` and `mise run vuln` exit 0.

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
