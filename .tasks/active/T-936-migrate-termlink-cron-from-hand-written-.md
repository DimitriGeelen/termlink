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

- Binary path convention: keep `/root/.agentic-framework/bin/fw` (v1.4.553 global) vs switch to `/opt/termlink/.agentic-framework/bin/fw` (vendored v0.9.611). The global binary is a newer lineage (1.x) — is that intentional?
- Should termlink's vendored framework be upgraded from v0.9.611 → v0.9.630 (or v1.4.553) before migration? The version-pin warning (`Pinned: 0.9.585 vs installed: 0.9.630`) suggests pin-management is also drifting.
- Should the pickup inbox processor become part of the default registry for all consumer projects? If yes, feeds into the P-009 upstream bug report (same pickup envelope already filed).

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

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [cron, migration, registry, T-448]
components: []
related_tasks: [T-921]
created: 2026-04-11T22:56:36Z
last_update: 2026-04-12T07:23:20Z
date_finished: null
---

# T-936: Migrate termlink cron from hand-written crontab to registry-based installer

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- [Criterion 1]
- [Criterion 2]

**NO-GO if:**
- [Criterion 1]
- [Criterion 2]

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T07:22:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
