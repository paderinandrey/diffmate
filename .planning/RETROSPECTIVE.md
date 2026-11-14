# Project Retrospective

*Updated after each milestone. Lessons feed the next planning cycle.*

## Milestone: v1.0 — SPEC

**Shipped:** 2026-11-14
**Phases:** 5 | **Plans:** 43 | **Tasks:** 74

### What Was Built

- Single review: `diffmate review <url>` and `diffmate status`, isolated worktree, validated output, runner lock
- Reliability: idempotent runs, crash recovery, draft freshness, timeouts, `--force`, signal handling, job attachment
- Queue: `diffmate watch`, tracking, debounce, retries, macOS notifications, job handoff
- Review and publish: `diffmate tui`, decisions, editing, one publication, reconciliation, comment sync
- Daily use: `diffmate doctor` and launchd install instructions
- The review agent that ships is the grok CLI. Phase 2 was accepted on 2026-11-14 after five named Grok live checks passed (nextest c3965e45, 204.489s)

### What Worked

- SPEC.md stayed the source of truth over `.planning/`
- All 43 plans are on main at `42ad952`
- The Grok live gate was five named ignored tests. Phase 2 was accepted only after they passed

### What Was Inefficient

- Phase 2 stayed open while phases 3–5 shipped, so acceptance happened last
- LIVE-01 and LIVE-02 kept Claude wording after the agent moved to the grok CLI. They were retargeted on 2026-11-14
- The accomplishment lines in `.planning/MILESTONES.md` are the plan summaries as written. Two still name Claude and an Anthropic block

### Patterns Established

- `gh`, `git`, and `grok` run through `tokio::process` with explicit argument vectors
- The ignored Claude isolation checks (`rev06`, `rev07`, `rev08`) stay the slice 1 record. They are not the Grok contract
- `.planning/config.json`, `.planning/milestone.lock`, and `.planning/state.json` stay untracked
- The crate version in `Cargo.toml` stays `0.1.0` at this planning tag

### Key Lessons

1. A production-shaped `grok -p` that returns `{"ok":true}` is not a pass of LIVE-01 or LIVE-02
2. `phase complete` needs verification status `passed` and a covered digest over files that exist on disk
3. A bare continue does not name the next milestone

### Cost Observations

- Model mix: not measured
- Sessions: not measured
- Notable: the five Grok checks on 2026-11-14 took 204.489s (nextest c3965e45). Token cost was not recorded

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 5 | 43 | First ship. Review agent is the grok CLI. Phase 2 accepted after the named live checks |

### Cumulative Quality

| Milestone | Tests | Coverage | Notes |
|-----------|-------|----------|-------|
| v1.0 | not remeasured at close | 80% line gate | `mise run coverage` is the gate. This close did not rerun it |

### Top Lessons (Verified Across Milestones)

1. SPEC.md wins over `.planning/`
2. Nothing reaches GitHub until the owner acts
