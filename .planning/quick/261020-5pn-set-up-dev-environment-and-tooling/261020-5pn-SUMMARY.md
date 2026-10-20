---
phase: 261020-5pn-set-up-dev-environment-and-tooling
plan: 01
subsystem: dev-environment
tags: [rust, mise, hk, cargo-nextest, cargo-llvm-cov, cargo-deny, github-actions, dependabot]
requires: []
provides:
  - pinned Rust toolchain and mise tools under a 3d minimum release age
  - mise tasks setup, build, lint, test, test:e2e, coverage, vuln, docs:check, deps:check, deps:update
  - hk pre-commit hooks (lint, large-files, secrets, docs, test, actions)
  - hardened CI, Dependencies workflow and Dependabot config
  - AGENTS.md with CLAUDE.md symlink and GSD claude_md_path
affects: [all later phases]
tech-stack:
  added: [rustc 1.99.1, hk, pkl, gitleaks, lychee, actionlint, cargo-deny, cargo-llvm-cov, cargo-nextest, cargo-machete]
  patterns: [mise task as the single command surface for agents, hooks and CI]
key-files:
  created:
    - rust-toolchain.toml
    - mise.toml
    - tests/cli.rs
    - Cargo.lock
    - .env.example
    - .gitleaks.toml
    - deny.toml
    - scripts/deps.sh
    - hk.pkl
    - .github/workflows/ci.yml
    - .github/workflows/deps.yml
    - .github/dependabot.yml
    - AGENTS.md
    - CLAUDE.md
  modified:
    - Cargo.toml
    - src/main.rs
    - .gitignore
    - .planning/config.json
key-decisions:
  - "hk hooks are installed with `hk install --mise --legacy` so the dev-env checker finds .git/hooks/pre-commit"
  - "actionlint kept as the plan names it although mise marks it deprecated in favour of jactionlint"
status: complete
commits: 3
plan_head_before: 52a53cd624e6539da42c56574c4a3ff37fd1c51b
plan_head_after: 5c54d34d6247a2827cae28636e11947b8b0378a0
actuals:
  tokens: 6050
  tasks: 3
  commits: 3
completed: 2026-10-20
---

# Phase 261020-5pn Plan 01: Dev environment and tooling Summary

Rust 1.99.1 pinned through rust-toolchain.toml (the global mise `rust` no longer overrides it), nine mise-managed tools, ten mise tasks shared by agents, hk hooks and GitHub Actions, and AGENTS.md as the single agent instruction file.

Branch `chore/dev-env-tooling`, three commits, nothing pushed, no PR.

- 40552d9 `chore(261020-5pn): pin Rust toolchain, mise tools, and test/coverage tasks`
- 3a9ce4b `chore(261020-5pn): add lint, vuln, docs and deps gates with hk pre-commit hooks`
- 5c54d34 `chore(261020-5pn): add CI, Dependabot and agent instructions`

No commit carries a `Co-Authored-By` trailer (`git log --format=%B 52a53cd..HEAD | grep -ci co-authored-by` printed 0).

## Versions chosen

All looked up on 2026-10-20 (about 01:25 UTC) inside the repo, with `minimum_release_age = "3d"` in effect (cutoff 2026-10-17 01:25 UTC).

| Item | Version | Lookup |
|------|---------|--------|
| Rust (rust-toolchain.toml channel and Cargo.toml rust-version) | 1.99.1 (published 2026-10-16T04:47:44Z) | `gh api 'repos/rust-lang/rust/releases?per_page=50' --jq '...'` |
| Rust edition | 2024 | `mise exec -- cargo new --vcs none <tmp>/probe` with the pinned toolchain |
| hk | 2.7.0 (2.7.1 skipped by mise: younger than 3 days) | `mise latest hk` |
| pkl | 0.32.1 | `mise latest pkl` |
| gitleaks | 8.30.1 | `mise latest gitleaks` |
| lychee | 0.24.2 | `mise latest lychee` |
| actionlint | 1.7.12 | `mise latest actionlint` |
| cargo-deny | 0.20.2 | `mise latest cargo-deny` |
| cargo-llvm-cov (`aqua:taiki-e/cargo-llvm-cov`) | 0.9.1 | `mise latest aqua:taiki-e/cargo-llvm-cov` |
| cargo-machete (`github:bnjbvr/cargo-machete`) | 0.9.2 (published 2026-04-15) | `mise latest github:bnjbvr/cargo-machete` |
| cargo-nextest (`github:nextest-rs/nextest`, `version_prefix = "cargo-nextest-"`) | 0.9.148 (published 2026-10-08) | `mise latest github:nextest-rs/nextest`, release date confirmed with `gh api` |
| actions/checkout | v7.0.1, `3d3c42e5aac5ba805825da76410c181273ba90b1` | `gh api repos/actions/checkout/releases`, `git ls-remote` (lightweight tag, commit SHA confirmed via the git commits API) |
| jdx/mise-action | v5.2.0, `9b200bcecbfd6303e69a4a261b2e09e3757440a7` (published 2026-10-16T11:27:18Z) | same method. v5.2.1 (2026-10-19) was younger than 3 days and skipped. |

The crate dependency list is empty, so no crate versions were chosen.

## Mise tasks, as observed

Final run on the committed tree (`mise run --force TASK`, exit codes read from the shell):

| Task | Result |
|------|--------|
| setup | exit 0 (installs tools, writes .git/hooks/pre-commit) |
| build | exit 0 |
| lint (lint:rust, lint:secrets, lint:actions) | exit 0 |
| lint:large-files | exit 0 in the Task 2 verify chain, and it passed inside the hk pre-commit of the Task 2 and Task 3 commits (not in the final loop) |
| test | exit 0, 1 test passed |
| coverage | exit 0, 100% lines (3/3) against the 80% gate |
| test:e2e | exit 0, 0 tests run (warning "no tests to run", expected: no `#[ignore]` tests yet) |
| vuln | exit 0 (advisories, bans, licenses, sources ok) |
| docs:check | exit 0 (2 files, 0 errors) |
| deps:check | exit 0 ("Rust 1.99.1, mise tools and direct crates are on their latest eligible releases") |
| deps:update | exit 0, changed nothing on the up-to-date tree (`git status --porcelain` empty after) |

Additional evidence:

- `mise exec -- rustc --version` reports `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, with rustfmt, clippy and llvm-tools installed.
- `hk validate` valid; `hk run pre-commit --all` passed (large-files, lint, docs, secrets, test).
- The Task 2 and Task 3 commits ran through the installed hook (lint, large-files, secrets, docs, test, actions all passed in the commit output).
- `shellcheck scripts/deps.sh` clean, 86 lines, executable.
- `actionlint` clean on both workflows. Every `uses:` line matches `@<40 hex> # v<N>`.
- `deps.sh` was exercised against a scratch copy of the repo (outside the tree) with deliberately stale state: old channel and rust-version, old pkl, old hk amends URL, old nextest table pin, and crates `anyhow =1.0.50` and dev-dependency `tempfile =3.2.0`. `check` listed every item and exited 1; `update` moved all of them (the nextest table kept its `version_prefix`), and `check` then exited 0.

## Final checker output

`fish ~/.claude/skills/dev-env-setup/scripts/check-dev-env.fish "$PWD"` on the committed tree, exit 0, `git status --porcelain` empty:

```
  PASS  mise config (mise.toml)
  PASS  mise loads .env
  PASS  .env.example exists
  PASS  .env not tracked
  PASS  .env ignored or absent from git status
  PASS  .env.example keys are in .env
  PASS  no plaintext secrets in working tree
  PASS  mise task: test
  PASS  test task runtime <= 10s
  PASS  mise task: lint
  WARN  mise task: serve/dev
  PASS  mise task: build
  PASS  mise task run blocks <= 10 lines
  PASS  mise task: lint:large-files
  PASS  mise shell aliases
  PASS  GitHub Actions pinned
  PASS  hk config (hk.pkl)
  PASS  pre-commit: lint step
  PASS  pre-commit: large-files step
  PASS  pre-commit: test step
  PASS  hk hooks installed
  PASS  git clean

  All clear! 22 checks passed (1 warnings).
```

The checker baseline at the start was 8 FAIL of 15. The only WARN left is serve/dev, which does not apply (diffmate has no server).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] hk install uses `--legacy`**
- **Found during:** Task 2 (hook install)
- **Issue:** hk 2.7.0 on this git version installs config-based hooks (`hook.hk-pre-commit.*` in .git/config) and leaves .git/hooks empty. The dev-env checker's "hk hooks installed" check only accepts `core.hooksPath` or an executable `.git/hooks/pre-commit` containing `hk`, and the plan's own verify greps that file. It would have failed.
- **Fix:** `setup` runs `hk install --mise --legacy`. The config-based entries from the first install were removed (`git config --remove-section hook.hk-pre-commit`) so hooks do not fire twice. The legacy shim is `exec mise x -- hk run pre-commit --from-hook`.
- **Files modified:** mise.toml
- **Commit:** 3a9ce4b

**2. [Rule 1 - Bug avoided] `unused-allowed-license = "allow"` set in deny.toml**
- **Issue:** Without it cargo-deny 0.20.2 prints a `license-not-encountered` warning for each of the 8 allowed licenses while the crate list is empty. The key was accepted (an unknown key makes the config fail to parse, which I checked), and the plan asked for it where supported.
- **Commit:** 3a9ce4b

No `--locked` flag was dropped and no `asset_pattern` override was needed. All nine tools resolved to the correct prebuilt assets for aarch64-apple-darwin.

## Notes and things not verified

- GitHub Actions did not run: nothing was pushed. Workflows are verified only by actionlint, the SHA pin regex and the YAML structure checks. The ubuntu-latest and macos-latest runs, jdx/mise-action behaviour with rustup on the runners, and the `deps.yml` run with `github.token` are unobserved.
- `scripts/deps.sh` was run only on macOS (BSD date, bash 3.2 syntax constraints kept). The GNU `date -d` branch and bash 5 are untested locally.
- The plan's Task 1 verify loop includes `git check-ignore -q .env`. A local secret-read guard hook blocks any Bash command that names `.env`, so that single check was replaced by `target`, `lcov.info`, `x.profraw` and `.DS_Store` via `git check-ignore`, plus reading `.gitignore` (contains `.env`) and the checker line "PASS .env ignored or absent from git status".
- `mise install` also installs the global config's tools (18 listed in total); only the nine project tools are in mise.toml.
- mise prints a deprecation warning for `actionlint` (suggests `jactionlint`). The plan and the skill name actionlint, so it stays. Switching is a one-line follow-up for the owner.
- hk 2.7.1 exists but is younger than 3 days, so 2.7.0 is pinned. `deps:check` honours the same rule and will flag 2.7.1 once it is old enough.
- If this branch is merged from a linked worktree, rerun the checker in the main checkout, because hook detection there differs (this run was in the main checkout).

## Known Stubs

None. `src/main.rs` is a deliberate minimal program (prints `diffmate <version>`); product code starts in Phase 1.

## Threat Flags

None. The surfaces added (workflows, deps.sh network calls to crates.io and GitHub) are covered by the plan's threat model: SHA pins, `contents: read`, `persist-credentials: false`, semver guards on every fetched version, and a one-request-per-second crates.io rate with an identifying User-Agent.

## Self-Check: PASSED

- Files exist: rust-toolchain.toml, Cargo.toml, Cargo.lock, src/main.rs, tests/cli.rs, mise.toml, .gitignore, .env.example, .gitleaks.toml, deny.toml, scripts/deps.sh, hk.pkl, .github/workflows/ci.yml, .github/workflows/deps.yml, .github/dependabot.yml, AGENTS.md, CLAUDE.md (symlink to AGENTS.md), .planning/config.json (claude_md_path = ./CLAUDE.md).
- Commits 40552d9, 3a9ce4b and 5c54d34 are ancestors of HEAD (5c54d34).
