---
id: T-936
name: "Migrate termlink cron from hand-written crontab to registry-based installer"
description: >
  ## Problem

Termlink's cron has three inconsistent layers discovered during P-009 pickup-processor rollout on 2026-04-12:

1. **`/etc/cron.d/agentic-audit-termlink`** (live) — 10 pre-migration jobs: structural audits ×3, OE fast/hourly/daily/weekly, full audit, docs regen, retention + manually-appended pickup processor (2026-04-12 interim fix). Binary path: `/root/.agentic-framework/bin/fw` (v1.4.553 — a separate lineage from termlink's vendored `.agentic-framework` at v0.9.611).

2. **`/opt/termlink/.context/cron/agentic-audit.crontab`** (git-tracked "source of truth") — **6-line empty stub** (header + SHELL + PATH, zero jobs). Claims to be source-of-truth but is out of sync with the live `/etc/cron.d/` content. Migration was started (file wiped) but never completed (no reinstall).

3. **`/opt/termlink/.context/cron-registry.yaml`** — exists but near-empty. The newer `fw cron install` (v0.9.630+) reads this as source-of-truth. Dry-run on 2026-04-12 confirmed: running `fw cron install` against current state would **overwrite `/etc/cron.d/` and silently wipe all 10 existing jobs.** Latent footgun.

## Scope of this inception

Decide the target end-state and produce a migration plan. Options to evaluate:

- **Option A — Full registry migration.** Populate `cron-registry.yaml` with all 10 existing jobs + pickup processor, then run `fw cron install`. Matches the pattern in `/opt/999-Agentic-Engineering-Framework` (reference implementation). Requires cataloguing current jobs and reverse-engineering the registry schema.
- **Option B — Rollback to hand-written.** Restore the git-tracked `agentic-audit.crontab` to mirror the live `/etc/cron.d/` content. Keeps the old flow, reinstates source-of-truth correctness, but rejects the framework's migration direction.
- **Option C — Hybrid.** Registry for new jobs, hand-written for legacy, with clear partition rules.

## Decisions needed

**Decision**: GO

**Rationale**: Registry populated with all 11 jobs, dry-run diff clean, binary path corrected to vendored

**Date**: 2026-04-12T13:13:47Z

## Acceptance criteria (human)

- [ ] `/etc/cron.d/agentic-audit-termlink`, the git-tracked source file, and `cron-registry.yaml` are all in sync (or clearly documented why not).
- [ ] `fw cron install --dry-run` produces zero diff against the installed file.
- [ ] Pickup processor job runs and the interim manual append can be removed.
- [ ] All 10 current scheduled jobs continue to run uninterrupted through the migration (no gap).
- [ ] A runbook / note captures which binary path convention is canonical for termlink going forward.

## Related

- T-921 — the original inception where this surfaced (P-009 pickup envelope filed upstream).
- P-009 — upstream bug report on hardcoded `bin/fw` path in error hints.
- T-448 — cron registry feature (the framework-side change that created this migration gap).
- `/opt/999-Agentic-Engineering-Framework/.context/cron-registry.yaml` — reference implementation to mirror.

## Source

Session on dimitri-mint-dev, 2026-04-12, installing pickup-processor cron on termlink. Discovery made during dry-run review before applying `fw cron install` — which would have been destructive.

status: work-completed
workflow_type: inception
owner: human
horizon: later
tags: [cron, migration, registry, T-448]
components: []
related_tasks: [T-921]
created: 2026-04-11T22:56:36Z
last_update: 2026-04-16T05:40:16Z
date_finished: 2026-04-12T13:13:47Z
---

# T-936: Migrate termlink cron from hand-written crontab to registry-based installer

## Problem Statement

Three cron layers (live `/etc/cron.d/`, git-tracked crontab stub, `cron-registry.yaml`) were out of sync. The registry has since been populated (11 jobs), the generated crontab uses the vendored binary path (`/opt/termlink/.agentic-framework/bin/fw`), and the only remaining step is to install.

## Assumptions

1. The vendored binary path (`/opt/termlink/.agentic-framework/bin/fw`) is correct for termlink — validated: this is what the project uses.
2. The pickup processor at `*/15` (every 15 min) is sufficient — replaces the interim every-30s hack.
3. No jobs will be lost — `cron-registry.yaml` contains all 11 jobs from the live crontab.

## Exploration Plan

1. **Catalogued live cron** — 11 jobs in `/etc/cron.d/agentic-audit-termlink` (done by prior session)
2. **Populated registry** — `cron-registry.yaml` now has all 11 entries (done by prior session)
3. **Dry-run validated** — `fw cron install --dry-run` shows clean diff: binary path fix + pickup schedule normalization
4. **Remaining: install** — `fw cron install` to sync live cron with registry (requires human approval, Tier 0)

## Technical Constraints

- Requires root to write `/etc/cron.d/` (Tier 0 action)
- Must not disrupt running cron jobs during switchover (atomic file replace)

## Scope Fence

**IN:** Migrate termlink cron to registry-based. Fix binary paths. Normalize pickup schedule.
**OUT:** Upgrading vendored framework version. Changing audit frequencies. Adding new jobs.

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-936, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- Registry contains all live jobs (11/11 — confirmed)
- Dry-run diff is clean and understood (confirmed: binary path fix + pickup normalization)
- No jobs will be lost during migration (confirmed: atomic file replace)

**NO-GO if:**
- Registry missing jobs vs live cron (not the case — 11/11)
- Dry-run shows unexpected deletions (not the case)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO (Option A — Full registry migration)

**Rationale:** The registry is already populated with all 11 jobs. The dry-run shows exactly two intentional changes: (1) binary path switches from global `/root/.agentic-framework/bin/fw` to vendored `/opt/termlink/.agentic-framework/bin/fw` (correct — aligns with T-909 vendoring), and (2) pickup processor schedule normalizes from every-30s hack to every-15m registry entry. No jobs are lost.

**Evidence:**
- `cron-registry.yaml` has 11 entries matching all live cron jobs
- `fw cron install --dry-run` diff is clean and understood
- Binary path fix is correct (vendored framework is source of truth for termlink)
- Pickup processor at 15m interval is sufficient (inbox processing is idempotent)

**Next step:** Run `cd /opt/termlink && bin/fw cron install` to apply (Tier 0 — requires human approval)

## Decisions

**Decision**: GO

**Rationale**: Registry populated with all 11 jobs, dry-run diff clean, binary path corrected to vendored

**Date**: 2026-04-12T13:13:47Z
## Decision

**Decision**: GO

**Rationale**: Registry populated with all 11 jobs, dry-run diff clean, binary path corrected to vendored

**Date**: 2026-04-12T13:13:47Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T07:22:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-12T13:13:47Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Registry populated with all 11 jobs, dry-run diff clean, binary path corrected to vendored

### 2026-04-12T13:13:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:40:16Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T22:08:30Z — programmatic-evidence [T-1097]
- **Evidence:** 4 crons installed via /etc/cron.d/ (agentic-audit, agentic-learnings-exchange, agentic-pickup, termlink-watchdog); registry-based pattern confirmed
- **Verified by:** automated command execution
