---
id: T-940
name: "RCA: Unify termlink runtime dir — split-brain blocks cross-host session discovery"
description: "RCA: Unify termlink runtime dir — split-brain blocks cross-host session discovery — see body for context."
status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: [termlink, systemd, runtime-dir, cross-host, rca]
components: []
related_tasks: [T-931, T-933, T-936]
created: 2026-04-12T07:43:43Z
last_update: 2026-04-23T19:30:28Z
date_finished: 2026-04-12T07:49:57Z
---

## Problem Context

## Problem discovered

During deployment of persistent agent sessions (framework-agent, termlink-agent) on .107, a critical split-brain was found:

- **Hub** (systemd, TCP 9100): `TERMLINK_RUNTIME_DIR=/var/lib/termlink` — persistent, survives reboots
- **CLI + ephemeral sessions** (interactive): default `/tmp/termlink-0` — tmpfs, lost on reboot
- **Agent sessions** (systemd): initially registered in wrong dir (invisible to both hub and CLI), fixed by explicitly setting runtime dir to match hub

**Impact**: Remote callers hit the hub on 9100, hub scans `/var/lib/termlink/sessions/` for targets. Sessions registered in `/tmp/termlink-0/sessions/` are invisible to remote callers. The sibling agent on .109 reported "No sessions on 192.168.10.107:9100" even though 5+ sessions existed locally.

**Current workaround**: Persistent agent services explicitly set `TERMLINK_RUNTIME_DIR=/var/lib/termlink` in their unit files. This puts them in the hub's pool. Ephemeral sessions remain in `/tmp/termlink-0` (local-only). Two-pool architecture works but is fragile and confusing.

## Root cause

T-931 (hub systemd deployment) set `TERMLINK_RUNTIME_DIR=/var/lib/termlink` for persistence (anticipating T-933: persist hub secret + TLS cert). But no corresponding migration happened for the CLI, ephemeral sessions, or agent sessions. The hub moved to a new address space; everything else stayed at the old one.

## Requirements to evaluate

1. **Single runtime dir** — all termlink processes (hub, sessions, CLI) should agree on one canonical location
2. **Persistence** — hub secret, TLS cert, and persistent session registrations must survive reboots
3. **Cross-host discovery** — sessions registered locally must be visible to remote callers via hub
4. **Local CLI ergonomics** — `termlink list` should show ALL sessions without env var gymnastics
5. **Backward compat** — existing scripts, cron jobs, and framework agents that don't set the env var must keep working

## Options to evaluate

- **A: Unify on `/var/lib/termlink`** — set `TERMLINK_RUNTIME_DIR=/var/lib/termlink` system-wide (e.g. `/etc/environment` or `/etc/profile.d/termlink.sh`). All processes join one pool. Persistent by default.
- **B: Unify on `/tmp/termlink-0`** — revert hub to tmpfs. Simple. But hub secret regenerates every reboot (breaks remote clients that cache the secret). Loses T-931/T-933 goals.
- **C: Hub bridges both dirs** — hub scans multiple session dirs. Most flexible but requires termlink code change (Rust-side).
- **D: Symlink bridge** — symlink `/tmp/termlink-0` → `/var/lib/termlink` (or vice versa). Zero-code hack. Fragile if anything creates the dir before the link.

## Use cases

1. Remote agent on .109 runs `termlink remote exec .107:9100 framework-agent 'fw pickup status'` — needs framework-agent in hub's session pool
2. Local user runs `termlink list` — expects to see ALL sessions (workers + agents)
3. `fw termlink dispatch` spawns ephemeral worker — worker is local-only (doesn't need hub visibility)
4. Hub restarts after reboot — secret + TLS cert persist, remote clients reconnect without reconfiguration
5. Agent session crashes — systemd restarts it, re-registers in correct dir, hub discovers it

## Value

Without unification, every new persistent session requires explicit `TERMLINK_RUNTIME_DIR` in its unit file, and operators must remember to use the env var for local inspection. This is a recurring source of confusion and will compound as more agent sessions are added.

## Acceptance criteria (human)

- [ ] `termlink list` (no env var) shows ALL sessions (persistent + ephemeral)
- [ ] `termlink remote list .107:9100` shows persistent agent sessions
- [ ] Hub secret and TLS cert survive reboot
- [ ] Existing scripts/cron/framework that don't set the env var keep working
- [ ] Decision documented as ADR or practice note

## Related

- T-931 — hub systemd deployment (introduced the split)
- T-933 — persist hub secret (the motivation for /var/lib/termlink)
- T-936 — cron registry migration (related infra consistency theme)

# T-940: RCA: Unify termlink runtime dir — split-brain blocks cross-host session discovery

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
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-940, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

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

**Decision**: GO

**Rationale**: E1 (unified runtime dir) and E2 (install.sh restart-on-change) implemented and verified. E3 and E4 filed as pickup envelopes, auto-created as T-941 and T-942. Human confirmed Go after reviewing RCA report at docs/reports/T-940-persistent-agent-sessions-rca.md.

**Date**: 2026-04-12T07:49:57Z
## Decision

**Decision**: GO

**Rationale**: E1 (unified runtime dir) and E2 (install.sh restart-on-change) implemented and verified. E3 and E4 filed as pickup envelopes, auto-created as T-941 and T-942. Human confirmed Go after reviewing RCA report at docs/reports/T-940-persistent-agent-sessions-rca.md.

**Date**: 2026-04-12T07:49:57Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T07:49:44Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** E1 (unified runtime dir) and E2 (install.sh restart-on-change) implemented and verified. E3 and E4 filed as pickup envelopes, auto-created as T-941 and T-942. Human confirmed Go after reviewing RCA report at docs/reports/T-940-persistent-agent-sessions-rca.md.

### 2026-04-12T07:49:49Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** E1 (unified runtime dir) and E2 (install.sh restart-on-change) implemented and verified. E3 and E4 filed as pickup envelopes, auto-created as T-941 and T-942. Human confirmed Go after reviewing RCA report at docs/reports/T-940-persistent-agent-sessions-rca.md.

### 2026-04-12T07:49:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
- **Reason:** E1+E2 implementation in progress

### 2026-04-12T07:49:57Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** E1 (unified runtime dir) and E2 (install.sh restart-on-change) implemented and verified. E3 and E4 filed as pickup envelopes, auto-created as T-941 and T-942. Human confirmed Go after reviewing RCA report at docs/reports/T-940-persistent-agent-sessions-rca.md.

### 2026-04-12T07:49:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:39:43Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T22:08:30Z — programmatic-evidence [T-1097]
- **Evidence:** T-1029..T-1032 fix split-brain: resolve_hub_paths searches both /var/lib/termlink and runtime dir; 98 hub tests passing; hub status correctly shows /tmp/termlink-0
- **Verified by:** automated command execution

### 2026-04-22T04:52:52Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
