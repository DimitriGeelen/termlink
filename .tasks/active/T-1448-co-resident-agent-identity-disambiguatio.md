---
id: T-1448
name: "co-resident agent identity disambiguation"
description: >
  Inception: co-resident agent identity disambiguation

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-02T17:26:31Z
last_update: 2026-05-02T17:28:21Z
date_finished: null
---

# T-1448: co-resident agent identity disambiguation

## Problem Statement

**The cryptographic identity model in TermLink is host+user-keyed, not agent-keyed.**

Two Claude sessions running as root on .107 share `/root/.termlink/identity.key` and therefore produce the **same** `identity_fingerprint` (`d1993c2c3ec44c94`). Discovered 2026-05-02 via cross-agent dialogue with peer agent ("Penelope") who noted the FP collision and proposed `from_project` metadata as the disambiguator.

This breaks several assumptions that the recent foundation work was built on:

| Recent work | What it assumed | What's actually true |
|---|---|---|
| **T-1427** (whoami + identity binding on channel post) | FP uniquely identifies the posting agent | FP identifies a (host, user) pair — multiple agents per pair are possible |
| **T-1436** (identity_fingerprint registered at session start) | Each session registration produces a distinct FP | Co-resident sessions register identical FPs |
| **T-1429** (`agent contact <name>`) | Name → FP resolution lands on a unique agent | Name → FP can land on a shared identity, then it's a coin flip which co-resident agent the message reaches |
| **T-1440 / T-1441** (whoami surfacing, FP column in `remote list`) | The visible FP tells the operator which agent they're talking to | The visible FP tells the operator which (host, user) — opaque w.r.t. agent count |
| **chat-arc attribution** (T-1438) | sender_id distinguishes participants | Co-resident participants are indistinguishable at the cryptographic layer |

Why now: the field rollout (T-1438) just made multi-agent coordination first-class. Until today every host had at most one Claude session, so FP-as-agent was an accidentally-correct shorthand. Penelope's arrival on .107 is the first real co-residency. As the chat-arc pattern propagates, FP collisions will become normal.

**The exploration question:** *What is the right model for agent identity in TermLink, given that the cryptographic root of trust is necessarily host+user-keyed?*

Sub-questions:
1. Is application-layer metadata (`_from`, `from_project`) the answer, or do we need a per-agent identity key?
2. If metadata: which field is canonical, and is that a convention or an enforced schema?
3. If per-agent key: how is it bound to the host's identity (sub-key, attestation)? How does it survive the agent's process lifecycle (re-fork, resume)?
4. How does the strict-reject path (T-1427 `-32014 CHANNEL_IDENTITY_MISMATCH`) interact — should it reject when sender_id and agent metadata disagree?
5. What does this mean for `agent contact <name>` — is "agent" a routable address or a label on top of (host, project)?

## Assumptions

To register with `fw assumption add` after template review:

- **A1** — Agents need addressability beyond (host, user). If a chat-arc has 2 co-resident peers, an operator must be able to direct a question at one and not the other.
- **A2** — Per-host per-user single-key identity is **not** a constraint we want to relax in the cryptographic layer. The host owns its private key; agents on it are not separately attested by an external CA. Adding a CA is out of scope.
- **A3** — At least one of {`_from`, `from_project`, `_thread`} is already present on most chat-arc posts in practice. Actual coverage needs measurement.
- **A4** — The strict-reject path (T-1427) currently checks `sender_id` against the connection's identity FP only, not against any agent-layer claim. Confirm by reading code.
- **A5** — A small set of metadata conventions + light schema enforcement is sufficient; we do not need to invent a new protocol-layer field.

## Exploration Plan

Time-boxed spikes — total ≤ 4h:

1. **Spike S1 (30 min) — Penelope locator card retrieval.** Read the cut-off message from the chat-arc / DM topic. Capture the proposed `from_project` convention, any code references she's already pointed at. Output: locator card transcript appended to research artifact.
2. **Spike S2 (45 min) — Code archaeology.** Read T-1427 strict-reject in `crates/termlink-hub`. Read T-1436 registration path. Read `channel post` metadata serialization. Establish: where does `sender_id` come from on the wire, what metadata fields are first-class vs. opaque, what (if any) per-agent state exists today.
3. **Spike S3 (60 min) — Field measurement.** Sample existing `agent-chat-arc` traffic (last 7d on .107 + .122 + .141). Tabulate: how often is `_from` present? `from_project`? `_thread`? Is there a de-facto convention already?
4. **Spike S4 (30 min) — Adversarial think.** What attacks does host+user FP allow today that per-agent identity would prevent? (Co-resident agent impersonation; misattribution of decisions in the audit trail.) Are these in our threat model?
5. **Spike S5 (30 min) — Sketch two designs.** (a) Pure metadata convention + schema validator. (b) Sub-key per agent derived from host key + agent UUID. Estimate cost and reversibility for each.
6. **Synthesize (30 min) — Recommendation + go/no-go.** Apply criteria below.

## Technical Constraints

- Identity key files (`/root/.termlink/identity.key`) are host-scoped, root-readable. Agent processes don't have isolated home dirs by default.
- Hub strict-reject path (T-1427) is already deployed on a subset of fleet (0.9.1693+); changing the rejection rule requires careful version-gating.
- `channel post` metadata is currently a free-form `--metadata key=value` flag — there's no schema enforcement.
- Multiple Claude sessions on the same UID is the dominant deployment pattern (Penelope is not exotic; this will recur).
- Agent UUIDs would need to be stable across the agent's process lifecycle (compaction, restart) — Claude Code provides session IDs that are NOT stable across `/clear` or restart.

## Scope Fence

**IN scope:**
- Identity model for agents addressed via TermLink (chat-arc, DMs, `agent contact`, channel posts, emit_to)
- How `sender_id` and agent-layer metadata relate, including strict-reject behavior
- A migration path from "FP-as-agent" assumptions in T-1427/T-1429/T-1436/T-1440/T-1441 to whatever model we land on

**OUT of scope:**
- External CA / attestation / hardware tokens
- Per-process Linux user isolation
- Threat model expansion to defend against root-level co-resident attackers (root is trusted; this is about disambiguation, not authorization)
- Renegotiating the host identity key model itself

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
- A model is identified that disambiguates co-resident agents using primitives already in the protocol (or a small, additive change)
- The model is implementable in ≤3 build tasks, each scoped to one session
- The model has a clear migration story for T-1427/T-1429/T-1436/T-1440/T-1441 — those tasks don't need to be unwound
- Field measurement (S3) shows the proposed metadata convention is already ≥70% present in real traffic, OR the convention is small enough that we can mandate it via T-1431 skill + heartbeat scripts

**NO-GO if:**
- Problem requires per-agent cryptographic keys (CA, attestation, sub-key derivation infrastructure) — punt to a separate larger initiative
- Penelope's `from_project` proposal turns out to be inconsistent with how the hub actually stamps `sender_id` (S2 disproves A4) — needs deeper protocol redesign
- Field measurement (S3) shows there's no existing convention AND no peer agent to coordinate with on adopting one — we'd be inventing in isolation

**DEFER if:**
- The co-residency case is verified to be Penelope-only and unlikely to recur — record the learning, defer the structural fix until a second incident

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

### 2026-05-02T17:28:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
