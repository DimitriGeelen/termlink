---
id: T-1155
name: "Channel-based communication bus for TermLink agents — subsume event.broadcast + inbox + pickup + send-file"
description: >
  Inception: Channel-based communication bus for TermLink agents — subsume event.broadcast + inbox + pickup + send-file

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-20T09:46:28Z
last_update: 2026-04-20T09:49:23Z
date_finished: null
---

# T-1155: Channel-based communication bus for TermLink agents — subsume event.broadcast + inbox + pickup + send-file

**Research artifact:** [docs/reports/T-1155-agent-communication-bus.md](../../docs/reports/T-1155-agent-communication-bus.md) — authoritative source for exploration findings, dialogue log, and recommendation. This task file is the governance wrapper; the report is the thinking.

## Problem Statement

Three operational pains: (1) agents or hubs frequently offline (liveness); (2) auth/secrets/certs not propagating cleanly across rotations (T-1051 lineage); (3) agents don't know termlink is available or what's happening fleet-wide (discoverability).

User proposed a shared channel-based communication bus (Signal/WhatsApp-shaped) — channels, 1:1, groups, messages + artifacts — as a unified fix.

**Single go/no-go question:** Can a channel-based bus subsume `event.broadcast` + `inbox` + `pickup envelopes` + `send-file` into one persistent, offline-tolerant abstraction, without adding a new liveness domain?

## Assumptions

Register via `fw assumption add "..." --task T-1155`.

- **A-001** Agents want channels, not just 1:1 sessions
- **A-002** Persistent history + cursor is worth the storage/complexity cost (vs live-only pub/sub)
- **A-003** A single trust anchor is achievable without making hub rotation harder
- **A-004** Offline-tolerant posting is feasible (local queue + replay on reconnect)
- **A-005** Migration from existing primitives is tractable (call-site count bounded)

## Exploration Plan

Time-boxed spikes — hard stop at **3h total**. If not done, descope, don't extend.

- **S-1 (30m)** Call-site census of `event.broadcast`, `inbox`, `send-file`, `pickup`. Classify by pattern.
- **S-2 (45m)** Persistence model spike — log-append vs ring vs TTL. Map to pains + subsumption.
- **S-3 (30m)** Liveness/offline-tolerance spike — viability of local queue + replay.
- **S-4 (30m)** Auth integration sketch — three candidates evaluated against T-1051 lineage.
- **S-5 (15m)** Migration scope estimate from S-1 output.

## Technical Constraints

- Must coexist with existing hub (TCP, TLS, HMAC secret model) during any migration
- Cannot introduce a second auth domain unless it *replaces* per-hub rotation (see A-003)
- Clients span Linux + macOS; storage model must not assume a managed database
- Fleet spans multiple LANs; federation (not hard-coded central server) should be at least sketchable

## Scope Fence

**IN:** subsumption analysis, sketch of persistence/auth/liveness models, migration-scope sizing.

**OUT (separate tasks if GO):** implementation, wire protocol spec, Watchtower UI, channel ACLs beyond sketch, cross-cluster federation details, non-agent uses (human-to-agent pager).

## Acceptance Criteria

### Agent
- [x] Call-site census complete (S-1) — counts and pattern classification recorded in research artifact
- [x] Persistence model spike complete (S-2) — ranked recommendation with disqualifiers
- [x] Offline-tolerance spike complete (S-3) — verdict on A-004 recorded
- [x] Auth integration sketch complete (S-4) — ranked recommendation with disqualifiers
- [x] Migration scope estimate complete (S-5) — concrete count + effort estimate
- [x] All 5 assumptions (A-001..A-005) either validated or explicitly deferred
- [x] Recommendation written in research artifact with evidence from all 5 spikes
- [x] All 5 decision criteria evaluated (subsumption / liveness / auth / migration / storage)

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if ALL of:**
1. **Subsumption clear** — `event.broadcast` + `inbox` replaced by channel abstraction; `pickup` + `send-file` replaced or cleanly reduced to special cases
2. **No new liveness domain** — bus runs inside hub, OR clients degrade gracefully (local queue) when bus unreachable
3. **Auth story plausible** — either reuses hub secrets with federation, or introduces a fleet-wide identity that *replaces* per-hub rotation
4. **Migration path exists** — concrete plan for moving N known call sites off legacy primitives without flag day
5. **Storage model chosen** — log-append vs ring vs TTL, with rationale tied to the 3 user pains

**NO-GO if any one** of the five is unresolved after the 3h exploration budget.

**DEFER if** the subsumption case is strong but auth story is not yet resolvable (e.g., depends on unblocking T-1051 downstream work first).

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation: GO** — build the bus, in-hub, log-append, self-sovereign identity, offline-tolerant client.

**Rationale:** All 5 go/no-go criteria met after 5-spike exploration. The bus is *worth building* because it does more than unify existing primitives — the self-sovereign identity design (S-4) structurally resolves the T-1051 auth-rotation pain by separating identity trust from transport trust, rather than just adding another layer on top.

**Evidence:**
- **S-1 (subsumption):** 30 call sites across 8 files map cleanly to `channel.post` + `channel.subscribe`. `event.broadcast` + `inbox` + `file.send/receive` + `pickup` all collapse to one primitive.
- **S-2 (persistence):** log-append with per-channel retention is the only model that cleanly subsumes pickup's "arrived-while-offline" semantic + inbox's per-recipient cursor + file-send durability.
- **S-3 (liveness):** bus running inside hub + client-side SQLite queue + replay on reconnect = no new liveness domain. Bus outages degrade to bounded-latency posts, not lost messages.
- **S-4 (auth):** ed25519 self-sovereign agent keys separate message authenticity from hub transport trust. Hub rotations (T-1051 lineage) stop invalidating messages. This is the structural fix, not a workaround.
- **S-5 (migration):** ~4000–6000 LOC effort, 3–5 weeks, 4-phase migration with legacy primitives staying live during transition. No flag day.

**What this replaces:**
- `event.broadcast` → `channel.post(topic="broadcast:global")`
- `inbox.{list,status,clear}` → `channel.{subscribe,post}` with recipient channel
- `file.send/receive` → `channel.post {type: artifact}` (artifact is a typed message)
- `pickup` (shell) → kept as-is with a `pickup → channel` bridge at framework boundary

**Out of scope for MVP (defer as separate follow-ups if GO):**
- Cross-hub federation (multi-hub-as-one-bus)
- Channel ACLs beyond "authenticated can post"
- Wire format optimization (JSON for now)

**Research artifact:** [docs/reports/T-1155-agent-communication-bus.md](../../docs/reports/T-1155-agent-communication-bus.md) — full spike details, tradeoff analysis, risks, and proposed follow-up task list.

**Decide via:** `bin/fw inception decide T-1155 go` (or `no-go` / `defer`) from `/opt/termlink`.

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

### 2026-04-20T09:47:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
