---
id: T-1297
name: "TermLink agent-routing discipline — prevent misrouting product traffic to wrong session"
description: >
  Inception: TermLink agent-routing discipline — prevent misrouting product traffic to wrong session

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [termlink, routing, agent-discipline, hub, structural]
components: []
related_tasks: [T-1291, T-243]
created: 2026-04-26T20:37:36Z
last_update: 2026-04-26T20:39:50Z
date_finished: null
---

# T-1297: TermLink agent-routing discipline — prevent misrouting product traffic to wrong session

## Problem Statement

Cross-host product traffic is being misrouted to `framework-agent` (governance role) instead of the originating product session (e.g. `email-archive`). 2026-04-26 incident: email-archive agent on .107 emitted `infra.lxc.delegate` (T-1191) and `infra.s3.bucket.delegate` (T-1194) targeted at `framework-agent` instead of its own product session. Operator caught it; agent re-emitted on the right bus. No hub-side detection — relied on human catch.

**Root cause:** agents have no API to ask the hub "who am I on this bus?" or "what's the canonical product session for project X?" — they guess, and the most-known-good target on the hub (`framework-agent`) becomes the default wrong answer. Memory `reference_termlink_peer_roles.md` documents the rule but is advice, not enforcement.

**Why now:** the email-archive cutover (T-1191/T-1194) is the second cross-host product-delegation thread we've had in 30 days (first was T-1294/T-1296 ring20 migration coordination). Both required cross-session routing; both saw guess-the-target friction. As more product agents come online (Penelope LXC, more ring20 services), the misroute frequency scales linearly with the number of distinct product sessions sharing the .107 bus.

**For whom:** any agent running in a multi-session-per-hub topology (currently 7 sessions on local-test hub, expected to grow). The cost of misroute today is "no harm, re-emit on the right bus" — but that's a human-cycle cost that scales badly and trains operators to babysit cross-host coordination.

## Assumptions

Register via `fw assumption add "..." --task T-1297`:

- A1: `framework-agent` will continue to be a multi-purpose session (governance + cross-project learning relay), so destination-side hard-rejection cannot be the fix (would break legitimate relay).
- A2: Most product topics have a stable mapping to a product role (e.g. `infra.*` → ring20-management/infra; `s3.*` → storage; `channel:learnings` → framework). The mapping table grows slowly — small enough to maintain.
- A3: Agents that misroute are doing so because they lack a cheap "self-lookup" API, NOT because they intentionally bypass routing rules. So a `whoami`-class lookup will be used when available.
- A4: A warn-not-reject lint at emit time is enough to catch regressions without breaking legit relay. Hard reject is not required for this class.

## Exploration Plan

Three short spikes, each ≤2 hours:

**Spike 1 — Quantify the gap.** Walk through .context/working/.event-history (or equivalent) for the last 30 days, count emits where `destination_role` doesn't match `topic_prefix` per a draft mapping. Goal: confirm misroute rate is non-trivial (>1% of cross-host emits) or disprove and de-prioritize.

**Spike 2 — Prototype `termlink whoami`.** Read-only RPC: returns `{session_id, display_name, roles, project_tag, hub_address}` for the calling session, derived from cwd → project → tag/role lookup. Verify the mapping is unambiguous in the current 7-session bus. Edge case: caller has no session registered yet (return null + actionable hint).

**Spike 3 — Topic↔role mapping shape.** Draft the mapping table format (TOML? hub-side YAML? per-project declaration?). Decide: centralized or distributed. Decide: opt-in `relay_for: ["topic.prefix.*"]` declaration on a session that whitelists otherwise-flagged topics. Sketch the warning event shape (channel, payload, suppression rules so we don't drown the bus).

## Technical Constraints

- **No hub upgrade lockstep.** Older termlink binaries (0.9.844 on ring20-management; 0.9.1262 in /root/.cargo/bin here) must not break when newer hubs add `whoami`/lint events. Forward-compatible by default; new RPCs return `Method not found` cleanly.
- **TOFU + auth unchanged.** Routing discipline lives ABOVE the auth layer. No interaction with the hub.secret / cert rotation work (T-1291 et al).
- **Performance.** `whoami` is on the agent's pre-emit hot path eventually. Must be a cheap lookup (in-memory hub state, no disk hit per call) — the hub already has the session registry, just exposes it.
- **Relay-friendly.** A1 above. The lint must cooperate with declared relays.

## Scope Fence

**IN scope:**
- `termlink whoami` RPC (or equivalent self-lookup) — read-only, returns caller's session identity on the hub.
- Topic↔role mapping table — format, scope (centralized vs distributed), maintenance model.
- Soft lint at emit time — warning-only, not reject; emits a sentinel event when self-emit topic doesn't match self-role.
- `relay_for: ["topic.*"]` opt-in declaration to suppress legit-relay false positives.
- Documenting the discipline in CLAUDE.md (and/or a TermLink doc) so future agents see the rule structurally, not just in memory.

**OUT of scope:**
- Hard rejection of misrouted emits — destination agent decides what to drop; hub doesn't.
- Topic→destination AUTO-routing (auto-rewrite). If we can't detect with confidence, we can't rewrite. Detection first; rewrite is a future inception.
- Cross-hub topic propagation rules. Single-hub discipline first.
- Migration of historical wrong-bus traffic. Sunk cost; flag-then-fix going forward.

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
- Spike 1 finds ≥3 misroute incidents in last 30 days OR misroute rate ≥1% of cross-host emits (i.e. recurrence pattern, not one-off).
- `whoami` prototype works in one session with no edge cases that demand redesign (i.e. the lookup is unambiguous in the current bus topology).
- Topic↔role mapping format converges on something a single human can maintain in <5 min/week.
- Combined fix is decomposable into ≤3 build tasks each ≤1 day.

**NO-GO if:**
- Misroute rate is <1% AND we can't identify a second incident — single anecdote, log a learning and move on.
- `whoami` requires intrusive session-registry restructuring to disambiguate (current sessions overlap roles in ways the lookup can't resolve).
- Topic↔role mapping needs >50 entries to be useful — too tribal to centralize, push to per-project memory.
- A simpler alternative emerges from spike 1 (e.g. naming convention enforced at `register` time) that obviates both whoami and the lint.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

<!-- Filled after spikes 1-3. Pre-spike inclination below; final recommendation lands here after evidence. -->

**Pre-spike inclination (2026-04-26 dialogue, NOT yet validated):**
GO on a combined `whoami` + topic↔role soft-lint fix, decomposed into 2-3 build tasks. Defense in depth — `whoami` removes the *guess*, lint catches the *miss* when guess fails or memory is stale. Hard rejection is dead because relay is a first-class operation; warning-only with `relay_for` opt-in covers the legit relay case.

Locked once spikes complete.

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

### 2026-04-26T20:39:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
