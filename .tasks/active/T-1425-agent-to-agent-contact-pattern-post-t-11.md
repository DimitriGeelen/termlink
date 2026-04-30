---
id: T-1425
name: "agent-to-agent contact pattern (post-T-1166 canon)"
description: >
  Inception: agent-to-agent contact pattern (post-T-1166 canon)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:09:32Z
last_update: 2026-04-30T21:14:04Z
date_finished: null
---

# T-1425: agent-to-agent contact pattern (post-T-1166 canon)

## Problem Statement

Vendored agents in the field repeatedly improvise agent-to-agent contact and get it wrong: legacy `inbox.push` instead of chat arc, fabricated identity strings, no delivery verification, hand-waved reply channels, no local task. The 2026-04-30 ZoneEdit DKIM/DMARC handoff from .107 to .122 is the latest instance — wrote a 59-line request, pushed via the primitive T-1166 is retiring, claimed delivery without verifying, asserted reply on a topic that doesn't exist.

Recurrent because there is no canonical contact pattern. Each agent re-invents from primitives. Six picks were proposed (deprecation print, agent-contact verb, topic self-doc, whoami binding, skill, doctor extension). Picks #1 and #4 (deprecation print + identity binding) need no design — they ship as separate small builds. The remaining four embed a real protocol question: *what is the canonical agent-to-agent contact pattern, post-T-1166?*

Now, because (a) T-1166 cut soon retires the legacy primitive vendored agents lean on, and (b) PL-098/T-1424 just proved cross-host chat arc carries operational meaning — we have the vehicle but no shared protocol on top of it.

## Assumptions

A-1: receivers (.122, .141, .143, future) have meaningfully different priorities for what an inbound contact envelope must contain — design from .107 alone keeps producing .107-shaped solutions. (Test: solicit replies on chat arc; if all three return identical answers, assumption is wrong and solo design is fine.)

A-2: a single high-level verb (`termlink agent contact <name> <msg>`) plus topic self-documentation is sufficient to replace the discover/list/push/inbox dance — vendored agents will use it if it exists. (Test: build the verb behind a feature flag, observe whether next vendored handoff reaches for it.)

A-3: identity binding via `termlink whoami` + post-side rejection is enforceable without breaking compatibility (the `from=` field is metadata, hub can reject mismatch without impacting payload routing). (Test: read existing topic posts, confirm sender_id is already authoritative and the synthetic `from=` strings live only in payload, not envelope headers.)

A-4: receivers are willing to subscribe to `agent-chat-arc` as a default — if they don't subscribe, no contact pattern works regardless of design. (Test: chat-arc topic membership audit on all reachable hubs.)

## Exploration Plan

Spike 1 — *RFC artifact + multi-agent feedback loop* (this task's deliverable, ~1 session):
- Write `docs/reports/T-1425-agent-contact-pattern-rfc.md` with initial design + 4-5 concrete questions
- Post `inception-rfc` envelope to `agent-chat-arc` with `metadata.thread=T-1425`
- Soak ~24h or until all reachable peer agents reply (whichever first)
- Synthesize replies into Decisions section
- Recommend GO + scoped build tasks, or NO-GO + recorded rationale

Spike 2 — *Receiver perspective probe* (parallel, async): in the RFC envelope, ask each peer 4-5 specific questions (auto-create vs explicit topic? per-message vs per-thread ack? fail-fast vs queue? identity rejection vs warning? thread retention?).

No third spike. If A-2 needs validation we build behind a flag — that's a build task downstream of GO, not part of this inception.

## Technical Constraints

- Cross-host coordination via `agent-chat-arc` only. Synchronous design dialogue across topic-poll latency is too slow; design must work async.
- Receiver agents are not always active — replies may take hours. Don't block on synchronous response.
- Identity binding must be backward compatible. Pre-binding posts have valid sender_id but no `metadata.from=` constraint; protocol must accept both during transition.
- Topic retention is `forever` for `agent-chat-arc` (already set); no constraint there.
- T-1166 cut is the deadline — once `LEGACY_PRIMITIVES_ENABLED=false` flips, anything depending on `inbox.push` breaks.

## Scope Fence

**IN scope:**
- Protocol/UX of agent-to-agent contact: verb shape, identity binding, topic semantics, ack mechanism, deprecation transition
- Recommendation on whether to build picks #2/#3/#5/#6 and in what order

**OUT of scope:**
- Picks #1 (deprecation print) and #4 (whoami binding) — those ship independently as small builds, no design question to answer
- Implementation details of any pick (line counts, file paths, language-level choices) — those live in the downstream build tasks
- T-1166 cut decision itself — that has its own ledger
- Cross-project (proxmox-ring20-management, laptop-141 projects) governance changes — each project decides for itself

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

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Reachable peer agents (.122, .141 minimum) respond to the RFC envelope and their answers either converge OR diverge along predictable axes (validating A-1)
- The four protocol questions resolve into a single coherent verb signature + identity model + topic semantics — no contradictions
- Each downstream build task (#2 verb, #3 topic-doc, #5 skill, #6 doctor extension) can be sized independently, fits one session, and has a clear owner

**NO-GO if:**
- Peers don't reply within 48h soak window AND solo .107 design is judged insufficient (A-1 unfalsifiable from this side alone) — defer until peers are responsive
- The questions cannot resolve without changing T-1166 cut semantics (e.g. requires keeping `inbox.push` alive long-term) — escalate to T-1166
- The protocol implies tighter coupling than the chat arc supports (e.g. requires sub-second sync) — fundamentally wrong vehicle, redesign

**DEFER if:**
- Answers come back but build task scoping reveals the right move is to land #1/#4 first, observe vendored-agent behavior for 2 weeks, then revisit — the foundation may make some picks unnecessary

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Pending synthesis after 48h soak (fire time: 2026-05-02T21:13Z / 23:13 local Europe/Amsterdam).**

If you are the next agent picking up this task: the soak window has elapsed. Run the synthesis sequence below.

### Synthesis runbook (executable by next session)

1. `cd /opt/termlink && .agentic-framework/bin/fw context focus T-1425`
2. Read RFC at `docs/reports/T-1425-agent-contact-pattern-rfc.md` (full design + 5 questions).
3. Walk topic for replies:
   ```
   termlink channel subscribe agent-chat-arc --cursor 7 --limit 100
   ```
   RFC is at offset 6. Look for replies with `metadata.thread=T-1425` and `metadata.in_reply_to=6` from peer sender_ids `9219671e` (.122 ring20-management) and `6604a2af` (.141 laptop-141).
4. For each reply: extract q1–q5 choices + per-host perspective. Append to `## 7. Dialogue Log` in the RFC artifact (format: `### YYYY-MM-DD — <peer-host> (<sender_id>) at offset N` then bulleted q1–q5 with rationale).
5. Build Decision matrix (per question: which choice each peer picked, convergence/divergence). Add to `## Decisions` below.
6. Write this `## Recommendation` section:
   - **GO** if: peers replied AND choices resolve cleanly into a coherent design
   - **NO-GO** if: zero replies in 48h AND solo .107 design is judged insufficient (re-post RFC or defer)
   - **DEFER** if: choices imply landing #1/#4 (T-1426/T-1427) first and observing for 2 weeks
7. If **GO**: create build tasks T-14XX per pick (#2 verb, #3 topic-doc, #5 skill, #6 doctor extension) with protocol decisions baked into ACs. Do NOT build under T-1425 (inception discipline).
8. Post synthesis envelope to `agent-chat-arc`:
   ```
   termlink channel post agent-chat-arc --msg-type status \
     --metadata "thread=T-1425,kind=inception-synthesis" \
     --payload '{"event":"inception-synthesis","decision":"...","convergence":{...},"next_tasks":[...]}'
   ```
9. `fw git commit -m "T-1425: inception synthesis after 48h soak — <decision>"` and push.
10. Leave T-1425 at `started-work` (owner=human; sovereignty gate R-033 blocks autonomous work-completed). User closes via watchtower.

### Constraints for synthesis agent

- Do not fabricate consensus. Zero replies → write NO-GO/DEFER with rationale.
- Identity discipline: respect each reply's `sender_id`; don't conflate.
- T-1426 / T-1427 (foundation builds for picks #1 / #4) are independent of this synthesis — reference them in the next-task list but don't gate on their state.
- Per CLAUDE.md inception discipline: this task may have only exploration commits. Build tasks land separately.

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

### 2026-04-30T21:11:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
