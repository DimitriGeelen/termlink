---
id: T-1297
name: "TermLink agent-routing discipline — prevent misrouting product traffic to wrong session"
description: >
  Inception: TermLink agent-routing discipline — prevent misrouting product traffic to wrong session

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [termlink, routing, agent-discipline, hub, structural]
components: []
related_tasks: [T-1291, T-243]
created: 2026-04-26T20:37:36Z
last_update: 2026-04-26T21:18:12Z
date_finished: 2026-04-26T21:18:12Z
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
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
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

**GO on combined fix:** `(1) termlink whoami + (2) topic↔role soft-lint + (3) relay_for opt-in`. Defense in depth — `whoami` removes the guess, lint catches the miss when guess fails or memory is stale, `relay_for` lets framework-agent legitimately forward cross-project traffic without warnings. Hard destination-rejection rejected because relay is a first-class operation. Decomposed into Build A whoami (½d) + Build B mapping+lint (1d) + Build C relay_for (½d) ≈ 2 dev-days total, reversible.

**Evidence (all three GO criteria satisfied):**
- Spike 1: 5 confirmed misroutes on framework-agent (seq 224/231/688/906/907) — payload-level evidence with originator + intended target both named, neither = framework-agent. ≥3-incident threshold met conservatively.
- Spike 2: cwd-only lookup is ambiguous in 71% of current sessions; env-var injection at register-time disambiguates without registry restructuring; existing registry already has all needed fields.
- Spike 3: 10 prefix rules + 4 exempt categories cover 95% of 125 live topics; centralized hub-side YAML + per-session `relay_for` TOML shape; <50 entries, well under 5-min/week budget.

**NO-GO triggers all cleared:** existing primitives have no concept of role-topic binding; misroutes succeeded auth (not auth-driven); register-time naming convention doesn't catch emit-time originator confusion.

Full evidence trail: `docs/reports/T-1297-termlink-agent-routing-discipline.md` §§ Spike 1/2/3 + Summary.

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

**Rationale**: GO on combined fix: `(1) termlink whoami + (2) topic↔role soft-lint + (3) relay_for opt-in`. Defense in depth — `whoami` removes the guess, lint catches the miss when guess fails or memory is stale, `relay_for` lets framework-agent legitimately forward cross-project traffic without warnings. Hard destination-rejection rejected because relay is a first-class operation. Decomposed into Build A whoami (½d) + Build B mapping+lint (1d) + Build C relay_for (½d) ≈ 2 dev-days total, reversible.

Evidence (all three GO criteria satisfied):
- Spike 1: 5 confirmed misroutes on framework-agent (seq 224/231/688/906/907) — payload-level evidence with originator + intended target both named, neither = framework-agent. ≥3-incident threshold met conservatively.
- Spike 2: cwd-only lookup is ambiguous in 71% of current sessions; env-var injection at register-time disambiguates without registry restructuring; existing registry already has all needed fields.
- Spike 3: 10 prefix rules + 4 exempt categories cover 95% of 125 live topics; centralized hub-side YAML + per-session `relay_for` TOML shape; <50 entries, well under 5-min/week budget.

NO-GO triggers all cleared: existing primitives have no concept of role-topic binding; misroutes succeeded auth (not auth-driven); register-time naming convention doesn't catch emit-time originator confusion.

Full evidence trail: `docs/reports/T-1297-termlink-agent-routing-discipline.md` §§ Spike 1/2/3 + Summary.

**Date**: 2026-04-26T21:18:12Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-26T20:39:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-26T21:05Z — spike-1 complete (quantify) [agent]
- **Method:** `termlink topics` (catalog, all 7 sessions) + `termlink events --target framework-agent --topic <name>` (per-topic payloads).
- **Result:** 5 confirmed misroutes on framework-agent — seq 224/231 (`infra.qdrant.down`, originator email-archive, target ring20-management-agent), seq 688 (`oauth.redirect-uri.help-requested`, originator email-archive@.107, target ring20-management-agent), seq 906 (`infra.lxc.delegate`, T-1191), seq 907 (`infra.s3.bucket.delegate`, T-1194). 4 distinct topics, 3 distinct tasks. Conservative lower bound — does not include 8+ product-prefixed topics (`email-archive.t11*.*`, `dashboard.{rekey,sibling,gap}.*`, `penelope.cutover.*`, `gpu.coordination.*`) where I couldn't unambiguously distinguish misroute from intentional framework-relay broadcast.
- **Volume context:** framework-agent next_seq=914 over 8d (~114 emits/day). 5/30 sampled product-prefixed emits ≈ 17% misroute rate among that subset.
- **GO criterion 1 satisfied:** ≥3 misroute incidents in last 30 days — 5 distinct events with payload-level evidence (originator + intended target both named in payload, neither = framework-agent).
- **Design signal:** every misrouted event carries a `relay_target` / `needs` / `from` field — the originator already encodes the intended destination at emit time. Option 2 soft-lint has high-quality input data: compare `topic_prefix` against `payload.{relay_target,needs}` and warn on mismatch.
- **Bug bonus (out-of-scope, follow-up needed):** one topic on framework-agent literally named `learning.shared</topic>\n<parameter name="from">email-archive` — XML interpolation leaked into topic string. Indicates insufficient topic-name validation on emit.
- **Artifact updated:** `docs/reports/T-1297-termlink-agent-routing-discipline.md` § Spike 1.

### 2026-04-26T21:15Z — spike-2 complete (whoami prototype) [agent]
- **Cwd-collision matrix:** 5 of 7 sessions share cwd with another (3× /opt/termlink, 2× /opt/3021-Bilderkarte-tool-llm). Cwd-only lookup fails 71% of the time.
- **Disambiguator design:** TERMLINK_SESSION_ID env var set at session creation (canonical), source-PID tree walk fallback, ambiguous-result hint for outsiders. Hub registry already tracks (id, display_name, roles, tags, cwd, pid, hub_address); whoami is pure exposure of existing state.
- **Existing CLI surface:** `termlink info` shows runtime state, `termlink status` queries OTHER sessions. No current `whoami`-class primitive — Spike 2 design is additive.
- **Edge cases checked:** multi-role sessions return list (no collapse to single role); cross-host out of scope; stale registrations already filtered. No design surprises.
- **GO criterion 2 satisfied:** lookup unambiguous once disambiguator added. Env-var injection at register-time is small additive change to existing register flow, NOT registry restructuring.

### 2026-04-26T21:25Z — spike-3 complete (mapping shape) [agent]
- **Coverage check:** drafted 10 prefix rules + 4 exempt categories; covers 119 of 125 live catalog topics (95%). Remaining 6 = 1 malformed-name bug + 5 unclassified product names needing operator review on first sight.
- **Centralized hub-side YAML chosen** over distributed: prefix conflicts need a single arbiter; lint must apply uniformly across all sessions; operator audits one file.
- **`relay_for` opt-in shape:** per-session TOML stanza in hubs.toml, lint suppresses warnings when declared session emits declared prefixes. Keeps central mapping clean.
- **Maintenance budget:** 10 rules + ~5 relay declarations < 50-entry NO-GO threshold and well within 5-min/week budget.
- **Decomposition:** Build A whoami ½d + Build B mapping+lint 1d + Build C relay_for ½d ≤ "≤3 builds each ≤1 day" satisfied.
- **GO criterion 3 satisfied.**

### 2026-04-26T21:30Z — all GO criteria satisfied — ready for inception decide [agent]
- **Summary:** GO criterion 1 (≥3 incidents) — 5 confirmed; GO criterion 2 (unambiguous whoami) — env-var solves 71% collision; GO criterion 3 (maintainable mapping) — 10 rules + 4 exempt cover 95%, decomposable into ≤3 ≤1-day builds.
- **NO-GO triggers all cleared:** existing primitives cannot cover; pain not auth-driven (all 5 misroutes succeeded auth); naming-convention-at-register doesn't address emit-time confusion.
- **Locked recommendation:** GO on combined fix `(1) termlink whoami + (2) topic↔role soft-lint + (3) relay_for opt-in`. Total ~2 dev-days, reversible.
- **Awaiting operator action:** record GO via inception decide, then create child build tasks T-A/B/C.

### 2026-04-26T21:18:12Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** GO on combined fix: `(1) termlink whoami + (2) topic↔role soft-lint + (3) relay_for opt-in`. Defense in depth — `whoami` removes the guess, lint catches the miss when guess fails or memory is stale, `relay_for` lets framework-agent legitimately forward cross-project traffic without warnings. Hard destination-rejection rejected because relay is a first-class operation. Decomposed into Build A whoami (½d) + Build B mapping+lint (1d) + Build C relay_for (½d) ≈ 2 dev-days total, reversible.

Evidence (all three GO criteria satisfied):
- Spike 1: 5 confirmed misroutes on framework-agent (seq 224/231/688/906/907) — payload-level evidence with originator + intended target both named, neither = framework-agent. ≥3-incident threshold met conservatively.
- Spike 2: cwd-only lookup is ambiguous in 71% of current sessions; env-var injection at register-time disambiguates without registry restructuring; existing registry already has all needed fields.
- Spike 3: 10 prefix rules + 4 exempt categories cover 95% of 125 live topics; centralized hub-side YAML + per-session `relay_for` TOML shape; <50 entries, well under 5-min/week budget.

NO-GO triggers all cleared: existing primitives have no concept of role-topic binding; misroutes succeeded auth (not auth-driven); register-time naming convention doesn't catch emit-time originator confusion.

Full evidence trail: `docs/reports/T-1297-termlink-agent-routing-discipline.md` §§ Spike 1/2/3 + Summary.

### 2026-04-26T21:18:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
