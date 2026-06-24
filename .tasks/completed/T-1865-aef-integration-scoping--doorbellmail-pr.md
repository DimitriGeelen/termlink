---
id: T-1865
name: "AEF integration scoping — doorbell+mail propagation vs explicit framework-agent coordination"
description: >
  Inception: AEF integration scoping — doorbell+mail propagation vs explicit framework-agent coordination

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-29T11:12:20Z
last_update: 2026-05-29T12:03:02Z
date_finished: 2026-05-29T12:03:02Z
---

# T-1865: AEF integration scoping — doorbell+mail propagation vs explicit framework-agent coordination

## Problem Statement

This session-resume + the prior one shipped a 9-verb doorbell+mail conversation
arc toolkit at the skill layer:

- PRESENCE: `/be-reachable` (T-1841)
- LIST: `/peers` (T-1859)
- SEND: `/agent-handoff` (T-1431/T-1429)
- RECEIVE: `/check-arc` (T-1810)
- READ broadcasts: `/recent-chat` (T-1851 → T-1849 script)
- READ per-peer DMs: `/recent-dm` (T-1862 → recent-dm.sh script + chat-arc-recent.sh `--topic`)
- BROADCAST: `/broadcast-chat` (T-1857)
- DIGEST: `/pulse` (T-1860)
- THREADS: `/conversations` (T-1864 → agent-conversation-list.sh)

Plus MCP-layer parity for the read trio (T-1839 listeners_fleet, T-1852 chat_arc_recent,
T-1863 recent_dm) and the broadcast (T-1858), plus systemd presence-emitter templates
(T-1832 / T-1840).

**The user's question:** does this propagate AUTOMATICALLY into AEF when a consumer
project invokes `termlink`, or does explicit framework-agent (cohort at /opt/999-AEF)
coordination need to happen to integrate the workflow into the framework?

**For whom:** AEF consumer projects (every project that uses the framework's
governance — currently /opt/termlink, /opt/003-NTB-ATC, and others). Specifically:
agents working under AEF governance who want to talk to each other doorbell+mail
style.

**Why now:** the toolkit is shipped and stable on /opt/termlink. The question of
"how does another project gain this capability?" determines whether we file a vendor
task next, a framework-agent handoff, or do nothing (it just works).

## Assumptions

A1: termlink BINARY distributes automatically via brew/GitHub releases.
A2: termlink SCRIPTS (scripts/recent-dm.sh etc.) live in /opt/termlink and do NOT
    auto-propagate to consumer projects.
A3: termlink SLASH SKILLS (.claude/commands/*.md) live in /opt/termlink and do NOT
    auto-propagate to consumer projects.
A4: AEF (/opt/999-AEF) and its vendored copies in consumer projects don't currently
    ship doorbell+mail toolkit assets.
A5: framework-agent (the cohort agent maintaining /opt/999-AEF) is the right
    coordinator for any framework-level vendoring.
A6: systemd presence templates need operator action per host (not framework-level).
A7: hubs.toml + secret deployment is operator action (not framework-level).

(A1, A6, A7 likely already true and uncontroversial. A2–A5 are the real
questions this inception answers.)

## Exploration Plan

Three time-boxed spikes (~15 min each, ~45 min total):

**Spike 1 — AEF surface scan (15 min).** Read /opt/999-AEF top level + any
.claude/commands/ + scripts/ + docs/operations/ to find existing doorbell+mail
references. Memory `workflow_channel1_upstream_mirror` says T-559 blocks Bash on
/opt/999-AEF — must use `termlink_run` MCP for reads.

Validates A4 (does AEF already ship anything?) and identifies any prior
discussion / partial vendor we'd need to update rather than create.

**Spike 2 — vendored-AEF relationship in /opt/termlink (15 min).** Read
/opt/termlink/.agentic-framework/ to understand the upstream→downstream sync
mechanism. Does upstream AEF ship `.claude/commands/` or does each consumer
project own its own? Does `fw upgrade` propagate skills, or only `agents/` and
`bin/`?

Validates A2/A3 — if `fw upgrade` already syncs `.claude/commands/`, the answer
might be "vendor the skills upstream once, every consumer gets them automatically."

**Spike 3 — framework-agent coordination channel (15 min).** Check
.context/project/ + recent DMs to known cohort agents on shared host
(d1993c2c) to see whether framework-agent is the right party, what they own,
and whether prior toolkit-vendoring (T-1656/T-1657 — "12 genuinely standalone
fabric cards") has a pattern we'd reuse.

Produces the actual handoff plan: cohort DM, OR upstream PR, OR docs only.

## Technical Constraints

- **T-559 sandbox** — Bash on /opt/999-AEF blocked. Use `termlink_run` MCP for any
  read/write to upstream. Memory `workflow_channel1_upstream_mirror` documents
  the pattern.
- **No source edits during inception** — scoping only. Build artifacts produced
  AFTER `fw inception decide T-1865 go` and on separate build task(s).
- **fw upgrade semantics unknown** — Spike 2 must answer whether
  `.claude/commands/` syncs upstream→downstream or is consumer-owned. The answer
  changes the recommendation significantly.
- **No new structural primitives** — this inception scopes DISTRIBUTION of an
  existing toolkit, not new doorbell+mail features.

## Scope Fence

**IN scope:**
- Identifying which parts of the doorbell+mail toolkit auto-propagate vs need
  coordination.
- Drafting the coordination handoff (cohort DM, AEF vendor PR, docs-only, or hybrid).
- Producing one decision: GO / NO-GO / DEFER with concrete next-task list.

**OUT of scope (post-decision build tasks):**
- Actually vendoring scripts/skills upstream — separate task per decided path.
- Deploying hubs / systemd units on AEF consumer hosts — operator work.
- Adopting per-agent keys (T-1693 / G-056 / G-057) — independent inception arc.
- Documenting the toolkit for end-users — separate docs task post-vendor.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if** (explicit coordination is required):
- Spike 2 shows `fw upgrade` does NOT sync `.claude/commands/` automatically (consumer-owned) — i.e. skills don't propagate, framework-agent coordination needed to vendor them upstream OR each consumer must copy them.
- OR Spike 1 shows AEF has no doorbell+mail awareness at all, meaning we need a deliberate vendor PR.
- AND we can name concrete follow-up tasks: vendor manifest, cohort DM, distribution docs, etc.

**NO-GO if** (it just propagates):
- Spike 2 shows `fw upgrade` already syncs `.claude/commands/` — vendor the skills once into upstream AEF and every consumer gets them on next upgrade. No further coordination needed beyond the one PR.
- AND Spike 1 confirms no conflicting AEF-side doorbell+mail integration exists.

**DEFER if:**
- Spikes reveal partial automation that bifurcates the answer (e.g. scripts propagate via `fw upgrade` but skills don't, or vice versa) — need follow-up inception per layer.
- OR the framework-agent coordination channel is itself broken / stale (e.g. last cohort DM was T-1166-era and the rail has rotted) — fix the channel first.

## Dialogue Log

Inception driven by direct human question (S-2026-0529-1159+ resume turn):
"please incept do we need to coordinate with framework agent for integrating
teh doorbell-mail workflow into aef , or will this go automatically when
termlink is called ?"

Initial agent reading: the question conflates two layers — the binary-level
rail (auto-propagates via release pipeline) and the operator-level toolkit
(scripts + skills + systemd templates, currently project-local). The
inception will disambiguate and produce a concrete answer per layer.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:**

Three spikes complete (full details in `docs/reports/T-1865-aef-integration-scoping.md`). The binary-level rail (termlink CLI + hubs + MCP server) propagates automatically via brew/GitHub releases — that part of the user's question is "yes, it just works." BUT the operator toolkit (9 doorbell+mail slash skills + 7 supporting scripts + systemd presence-emitter template) is 100% termlink-project-local AND `fw upgrade`'s `do_vendor` includes list (`bin lib agents web docs .tasks/templates FRAMEWORK.md metrics.sh`, fw line 254-264) does NOT cover `.claude/commands/` or `scripts/`. Upstream AEF has the PATTERN for shipping framework skills (10 framework-default skills in `/opt/999-Agentic-Engineering-Framework/.claude/commands/`) but contains ZERO doorbell+mail awareness. So AEF consumers do NOT get the toolkit automatically — explicit vendor work is required. Recommended path: Phase 1 ships skill+script bundle upstream (direct commit via termlink_run, no behavioral change), Phase 2 extends `do_vendor` includes so the toolkit propagates to consumer projects on next `fw upgrade`. Coordination channel (cohort DM rail) is dormant — framework-agent has no live heartbeat, 1 chat post / 7 days — so coordination cannot be DM-driven; goes via upstream commit, with retroactive notification when framework-agent next surfaces.

**Evidence:**

- Upstream skill bundle present (10 skills, none doorbell+mail): Spike 1
  `ls /opt/999-Agentic-Engineering-Framework/.claude/commands/`
- Upstream scripts absent (only `spikes/` subdir): Spike 1 `ls .../scripts/`
- `do_vendor` includes list omits both `.claude/commands/` and `scripts/`:
  `.agentic-framework/bin/fw:254-264`
- `fw upgrade` only sync's `.claude/commands/resume.md` via a separate
  special-case path: `.agentic-framework/lib/upgrade.sh:172` + 1020-1051
- Fleet has 1 listener (this host) as of 2026-05-29: Spike 3
  `bash scripts/agent-listeners-fleet.sh --include-offline --json`
- Cohort DM channel: 1 chat post in 168h, sender=this host: Spike 3
  `bash scripts/recent-dm.sh d1993c2c --since 168 --filter-msg-type chat`

**Concrete follow-up tasks (NOT created until GO decision):**

- T-1866 (build) — vendor doorbell+mail skill + script bundle into upstream
  `/opt/999-Agentic-Engineering-Framework` via termlink_run direct commit.
- T-1867 (build) — extend `do_vendor` `includes` to add `.claude/commands/`
  + `scripts/` so the toolkit propagates to consumers on next `fw upgrade`.
  Affects EVERY consumer project — careful review required.
- T-1868 (docs, optional) — operator runbook for hub deployment + presence
  opt-in + identity setup.

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

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale:

Three spikes complete (full details in `docs/reports/T-1865-aef-integration-scoping.md`). The binary-level rail (termlink CLI + hubs + MCP server) propagates automatically via brew/GitHub releases — that part of the user's question is "yes, it just works." BUT the operator toolkit (9 doorbell+mail slash skills + 7 supporting scripts + systemd presence-emitter template) is 100% termlink-project-local AND `fw upgrade`'s `do_vendor` includes list (`bin lib agents web docs .tasks/templates FRAMEWORK.md metrics.sh`, fw line 254-264) does NOT cover `.claude/commands/` or `scripts/`. Upstream AEF has the PATTERN for shipping framework skills (10 framework-default skills in `/opt/999-Agentic-Engineering-Framework/.claude/commands/`) but contains ZERO doorbell+mail awareness. So AEF consumers do NOT get the toolkit automatically — explicit vendor work is required. Recommended path: Phase 1 ships skill+script bundle upstream (direct commit via termlink_run, no behavioral change), Phase 2 extends `do_vendor` includes so the toolkit propagates to consumer projects on next `fw upgrade`. Coordination channel (cohort DM rail) is dormant — framework-agent has no live heartbeat, 1 chat post / 7 days — so coordination cannot be DM-driven; goes via upstream commit, with retroactive notification when framework-agent next surfaces.

Evidence:

- Upstream skill bundle present (10 skills, none doorbell+mail): Spike 1
  `ls /opt/999-Agentic-Engineering-Framework/.claude/commands/`
- Upstream scripts absent (only `spikes/` subdir): Spike 1 `ls .../scripts/`
- `do_vendor` includes list omits both `.claude/commands/` and `scripts/`:
  `.agentic-framework/bin/fw:254-264`
- `fw upgrade` only sync's `.claude/commands/resume.md` via a separate
  special-case path: `.agentic-framework/lib/upgrade.sh:172` + 1020-1051
- Fleet has 1 listener (this host) as of 2026-05-29: Spike 3
  `bash scripts/agent-listeners-fleet.sh --include-offline --json`
- Cohort DM channel: 1 chat post in 168h, sender=this host: Spike 3
  `bash scripts/recent-dm.sh d1993c2c --since 168 --filter-msg-type chat`

Concrete follow-up tasks (NOT created until GO decision):

- T-1866 (build) — vendor doorbell+mail skill + script bundle into upstream
  `/opt/999-Agentic-Engineering-Framework` via termlink_run direct commit.
- T-1867 (build) — extend `do_vendor` `includes` to add `.claude/commands/`
  + `scripts/` so the toolkit propagates to consumers on next `fw upgrade`.
  Affects EVERY consumer project — careful review required.
- T-1868 (docs, optional) — operator runbook for hub deployment + presence
  opt-in + identity setup.

**Date**: 2026-05-29T12:03:02Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-29T11:39:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-29T12:03:02Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

Three spikes complete (full details in `docs/reports/T-1865-aef-integration-scoping.md`). The binary-level rail (termlink CLI + hubs + MCP server) propagates automatically via brew/GitHub releases — that part of the user's question is "yes, it just works." BUT the operator toolkit (9 doorbell+mail slash skills + 7 supporting scripts + systemd presence-emitter template) is 100% termlink-project-local AND `fw upgrade`'s `do_vendor` includes list (`bin lib agents web docs .tasks/templates FRAMEWORK.md metrics.sh`, fw line 254-264) does NOT cover `.claude/commands/` or `scripts/`. Upstream AEF has the PATTERN for shipping framework skills (10 framework-default skills in `/opt/999-Agentic-Engineering-Framework/.claude/commands/`) but contains ZERO doorbell+mail awareness. So AEF consumers do NOT get the toolkit automatically — explicit vendor work is required. Recommended path: Phase 1 ships skill+script bundle upstream (direct commit via termlink_run, no behavioral change), Phase 2 extends `do_vendor` includes so the toolkit propagates to consumer projects on next `fw upgrade`. Coordination channel (cohort DM rail) is dormant — framework-agent has no live heartbeat, 1 chat post / 7 days — so coordination cannot be DM-driven; goes via upstream commit, with retroactive notification when framework-agent next surfaces.

Evidence:

- Upstream skill bundle present (10 skills, none doorbell+mail): Spike 1
  `ls /opt/999-Agentic-Engineering-Framework/.claude/commands/`
- Upstream scripts absent (only `spikes/` subdir): Spike 1 `ls .../scripts/`
- `do_vendor` includes list omits both `.claude/commands/` and `scripts/`:
  `.agentic-framework/bin/fw:254-264`
- `fw upgrade` only sync's `.claude/commands/resume.md` via a separate
  special-case path: `.agentic-framework/lib/upgrade.sh:172` + 1020-1051
- Fleet has 1 listener (this host) as of 2026-05-29: Spike 3
  `bash scripts/agent-listeners-fleet.sh --include-offline --json`
- Cohort DM channel: 1 chat post in 168h, sender=this host: Spike 3
  `bash scripts/recent-dm.sh d1993c2c --since 168 --filter-msg-type chat`

Concrete follow-up tasks (NOT created until GO decision):

- T-1866 (build) — vendor doorbell+mail skill + script bundle into upstream
  `/opt/999-Agentic-Engineering-Framework` via termlink_run direct commit.
- T-1867 (build) — extend `do_vendor` `includes` to add `.claude/commands/`
  + `scripts/` so the toolkit propagates to consumers on next `fw upgrade`.
  Affects EVERY consumer project — careful review required.
- T-1868 (docs, optional) — operator runbook for hub deployment + presence
  opt-in + identity setup.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-a9850124
- **Timestamp:** 2026-05-29T12:03:03Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-29T12:03:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
