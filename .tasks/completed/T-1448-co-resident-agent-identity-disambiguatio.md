---
id: T-1448
name: "co-resident agent identity disambiguation"
description: >
  Inception: co-resident agent identity disambiguation

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-02T17:26:31Z
last_update: 2026-05-20T14:12:51Z
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

**Recommendation:** GO with **Design A (sharpened)** — soft convention + CLI default + T-1288 catalog promotion + warning-on-unresolvable-project at the CLI.

**Conceptual frame (elaboration):** Today's TermLink "identity" is a single ed25519 keypair on disk that is being asked to play two distinct roles: (i) host endpoint pinning + post signing (cryptographic, host-keyed — *correct as designed*) and (ii) chat-arc attribution + `agent contact` routing (operational — *needed an agent axis we never had*). The right move is not to redesign identity (that's C); it's to **add a second axis** at the application layer where it belongs. `from_project` is the natural fit because project directory is stable across `/clear`, restart, and compaction (session-id is not), and is operator-meaningful (UUIDs are not).

**Directive scoring (full table in research artifact):**

| Directive | A | B | C | D |
|---|---|---|---|---|
| Antifragility | ✅ | ✅ | ✅ | ⚠️ |
| Reliability | ✅ | ✅✅ | ✅✅ | ❌ |
| Usability | ✅✅ | ⚠️ | ⚠️⚠️ | ❌ |
| Portability | ✅✅ | ⚠️ | ❌ | ✅ (false win) |

A is the only option that aligns with all four directives without trading off. A is also **additive** to B and C — choosing A now does not preempt B or C later.

**Steelman/strawman summary:**
- **A:** "Codify a learning that emerged" vs. "It's just a string anyone can lie about." → Threat model trusts root; we already accept that anyone with the key can lie about anything. Authenticating `from_project` defends an attack we explicitly do not defend against.
- **B:** "Pay protocol cost once, get cryptographic guarantees forever" vs. "Self-inflicted version-gate after T-1166/T-1418/T-1294/T-1438 just removed that pain." → A is additive to B; choosing A doesn't preempt B.
- **C (per-project identity keys):** "Architecturally cleanest, no metadata convention needed" vs. "Per-project auth bootstrap multiplies the heal protocol." → Defer as future option if threat model ever shifts.
- **D (do nothing):** "Don't codify before second occurrence" vs. "Antifragility anti-pattern — every new agent re-derives the same lesson." → Fails antifragility outright.

**Rationale:** TermLink's identity is host+user-keyed by design (`/root/.termlink/identity.key` is shared by every process under that UID). Co-resident agents on .107 (cohort `002-Claude-Partner-Network` + email-archive `050-email-archive`) already produce identical FP `d1993c2c3ec44c94`, and they have ALREADY coordinated in-band on `from_project` metadata as the disambiguator (chat-arc offset 73, 12h ago). The fix is to codify this convention at the CLI layer and promote `from_project` to the T-1288 well-known-keys catalog. **No protocol change. No version gate. Unchanged threat model.** T-1427's strict-reject stays valid (it disambiguates host identities); T-1429/T-1436/T-1440/T-1441 augment-not-unwind to surface project alongside FP.

**Evidence:**
- **Code (S2):** `crates/termlink-hub/src/channel.rs:436-451` — strict-reject is `sender_id == fingerprint_of(verifying_key)`, NOT cross-checked against any agent metadata. Lines 453-464 — metadata is opaque routing-hint map, "NOT included in canonical signed bytes — trusted-mesh threat model treats it as routing only." Verbatim.
- **Field (S3):** 73 chat-arc entries, 2 unique FPs (1 of which collapses 2 co-resident agents). `_thread`=36% (T-1438 era convention), `_from`=16%, `from_project`=7% (brand new, only on the pen-contract thread). Convention will be **mandated** via cheap CLI default, not promoted from organic majority.
- **Threat model (S4):** TermLink trusts root. Co-resident-forge is out of scope. Design B (signed metadata + sub-keys) over-engineers for an attack we don't defend against.
- **Migration:** 5 affected tasks (T-1427, T-1429, T-1436, T-1440, T-1441) all augment-not-unwind. T-1427 strict-reject still correctly identifies the host; project is a separate axis.

**Cost:** 3 build tasks, ~1 session each:
1. **(a)** termlink-cli: default `from_project` injection from `.context/working/focus.yaml` / `.framework.yaml`; add to T-1288 catalog
2. **(b)** T-1429 extension: `agent contact <name>[:project]` resolution; auto-attach `to_project`
3. **(c)** scripts + skills sync: `field-heartbeat.sh`, `vendored-arc-heartbeat.sh`, `/agent-handoff`, `/check-arc` — emit + read `from_project`

**Order:** a → c → b. (a) unblocks (c); (b) is the operator-visible payoff.

**Reversibility:** High. Each task is independent and reversible; metadata field stays in place even if catalog entry is rolled back.

**Out of scope:** sub-key cryptography (Design B); hub-side schema enforcement; renaming `_from`/`_thread`; cross-host project-namespace conflicts (e.g. two `050-email-archive` directories on different hosts) — flagged for follow-up.

Full report: `docs/reports/T-1448-co-resident-agent-identity-inception.md`

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

**Rationale**: Recommendation: GO with Design A (sharpened) — soft convention + CLI default + T-1288 catalog promotion + warning-on-unresolvable-project at the CLI.

Conceptual frame (elaboration): Today's TermLink "identity" is a single ed25519 keypair on disk that is being asked to play two distinct roles: (i) host endpoint pinning + post signing (cryptographic, host-keyed — correct as designed) and (ii) chat-arc attribution + `agent contact` routing (operational — needed an agent axis we never had). The right move is not to redesign identity (that's C); it's to add a second axis at the application layer where it belongs. `from_project` is the natural fit because project directory is stable across `/clear`, restart, and compaction (session-id is not), and is operator-meaningful (UUIDs are not).

Directive scoring (full table in research artifact):

| Directive | A | B | C | D |
|---|---|---|---|---|
| Antifragility | ✅ | ✅ | ✅ | ⚠️ |
| Reliability | ✅ | ✅✅ | ✅✅ | ❌ |
| Usability | ✅✅ | ⚠️ | ⚠️⚠️ | ❌ |
| Portability | ✅✅ | ⚠️ | ❌ | ✅ (false win) |

A is the only option that aligns with all four directives without trading off. A is also additive to B and C — choosing A now does not preempt B or C later.

Steelman/strawman summary:
- A: "Codify a learning that emerged" vs. "It's just a string anyone can lie about." → Threat model trusts root; we already accept that anyone with the key can lie about anything. Authenticating `from_project` defends an attack we explicitly do not defend against.
- B: "Pay protocol cost once, get cryptographic guarantees forever" vs. "Self-inflicted version-gate after T-1166/T-1418/T-1294/T-1438 just removed that pain." → A is additive to B; choosing A doesn't preempt B.
- C (per-project identity keys): "Architecturally cleanest, no metadata convention needed" vs. "Per-project auth bootstrap multiplies the heal protocol." → Defer as future option if threat model ever shifts.
- D (do nothing): "Don't codify before second occurrence" vs. "Antifragility anti-pattern — every new agent re-derives the same lesson." → Fails antifragility outright.

Rationale: TermLink's identity is host+user-keyed by design (`/root/.termlink/identity.key` is shared by every process under that UID). Co-resident agents on .107 (cohort `002-Claude-Partner-Network` + email-archive `050-email-archive`) already produce identical FP `d1993c2c3ec44c94`, and they have ALREADY coordinated in-band on `from_project` metadata as the disambiguator (chat-arc offset 73, 12h ago). The fix is to codify this convention at the CLI layer and promote `from_project` to the T-1288 well-known-keys catalog. No protocol change. No version gate. Unchanged threat model. T-1427's strict-reject stays valid (it disambiguates host identities); T-1429/T-1436/T-1440/T-1441 augment-not-unwind to surface project alongside FP.

Evidence:
- Code (S2): `crates/termlink-hub/src/channel.rs:436-451` — strict-reject is `sender_id == fingerprint_of(verifying_key)`, NOT cross-checked against any agent metadata. Lines 453-464 — metadata is opaque routing-hint map, "NOT included in canonical signed bytes — trusted-mesh threat model treats it as routing only." Verbatim.
- Field (S3): 73 chat-arc entries, 2 unique FPs (1 of which collapses 2 co-resident agents). `_thread`=36% (T-1438 era convention), `_from`=16%, `from_project`=7% (brand new, only on the pen-contract thread). Convention will be mandated via cheap CLI default, not promoted from organic majority.
- Threat model (S4): TermLink trusts root. Co-resident-forge is out of scope. Design B (signed metadata + sub-keys) over-engineers for an attack we don't defend against.
- Migration: 5 affected tasks (T-1427, T-1429, T-1436, T-1440, T-1441) all augment-not-unwind. T-1427 strict-reject still correctly identifies the host; project is a separate axis.

Cost: 3 build tasks, ~1 session each:
1. (a) termlink-cli: default `from_project` injection from `.context/working/focus.yaml` / `.framework.yaml`; add to T-1288 catalog
2. (b) T-1429 extension: `agent contact <name>[:project]` resolution; auto-attach `to_project`
3. (c) scripts + skills sync: `field-heartbeat.sh`, `vendored-arc-heartbeat.sh`, `/agent-handoff`, `/check-arc` — emit + read `from_project`

Order: a → c → b. (a) unblocks (c); (b) is the operator-visible payoff.

Reversibility: High. Each task is independent and reversible; metadata field stays in place even if catalog entry is rolled back.

Out of scope: sub-key cryptography (Design B); hub-side schema enforcement; renaming `_from`/`_thread`; cross-host project-namespace conflicts (e.g. two `050-email-archive` directories on different hosts) — flagged for follow-up.

Full report: `docs/reports/T-1448-co-resident-agent-identity-inception.md`

**Date**: 2026-05-02T20:09:45Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-02T17:28:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-02T20:09:45Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO with Design A (sharpened) — soft convention + CLI default + T-1288 catalog promotion + warning-on-unresolvable-project at the CLI.

Conceptual frame (elaboration): Today's TermLink "identity" is a single ed25519 keypair on disk that is being asked to play two distinct roles: (i) host endpoint pinning + post signing (cryptographic, host-keyed — correct as designed) and (ii) chat-arc attribution + `agent contact` routing (operational — needed an agent axis we never had). The right move is not to redesign identity (that's C); it's to add a second axis at the application layer where it belongs. `from_project` is the natural fit because project directory is stable across `/clear`, restart, and compaction (session-id is not), and is operator-meaningful (UUIDs are not).

Directive scoring (full table in research artifact):

| Directive | A | B | C | D |
|---|---|---|---|---|
| Antifragility | ✅ | ✅ | ✅ | ⚠️ |
| Reliability | ✅ | ✅✅ | ✅✅ | ❌ |
| Usability | ✅✅ | ⚠️ | ⚠️⚠️ | ❌ |
| Portability | ✅✅ | ⚠️ | ❌ | ✅ (false win) |

A is the only option that aligns with all four directives without trading off. A is also additive to B and C — choosing A now does not preempt B or C later.

Steelman/strawman summary:
- A: "Codify a learning that emerged" vs. "It's just a string anyone can lie about." → Threat model trusts root; we already accept that anyone with the key can lie about anything. Authenticating `from_project` defends an attack we explicitly do not defend against.
- B: "Pay protocol cost once, get cryptographic guarantees forever" vs. "Self-inflicted version-gate after T-1166/T-1418/T-1294/T-1438 just removed that pain." → A is additive to B; choosing A doesn't preempt B.
- C (per-project identity keys): "Architecturally cleanest, no metadata convention needed" vs. "Per-project auth bootstrap multiplies the heal protocol." → Defer as future option if threat model ever shifts.
- D (do nothing): "Don't codify before second occurrence" vs. "Antifragility anti-pattern — every new agent re-derives the same lesson." → Fails antifragility outright.

Rationale: TermLink's identity is host+user-keyed by design (`/root/.termlink/identity.key` is shared by every process under that UID). Co-resident agents on .107 (cohort `002-Claude-Partner-Network` + email-archive `050-email-archive`) already produce identical FP `d1993c2c3ec44c94`, and they have ALREADY coordinated in-band on `from_project` metadata as the disambiguator (chat-arc offset 73, 12h ago). The fix is to codify this convention at the CLI layer and promote `from_project` to the T-1288 well-known-keys catalog. No protocol change. No version gate. Unchanged threat model. T-1427's strict-reject stays valid (it disambiguates host identities); T-1429/T-1436/T-1440/T-1441 augment-not-unwind to surface project alongside FP.

Evidence:
- Code (S2): `crates/termlink-hub/src/channel.rs:436-451` — strict-reject is `sender_id == fingerprint_of(verifying_key)`, NOT cross-checked against any agent metadata. Lines 453-464 — metadata is opaque routing-hint map, "NOT included in canonical signed bytes — trusted-mesh threat model treats it as routing only." Verbatim.
- Field (S3): 73 chat-arc entries, 2 unique FPs (1 of which collapses 2 co-resident agents). `_thread`=36% (T-1438 era convention), `_from`=16%, `from_project`=7% (brand new, only on the pen-contract thread). Convention will be mandated via cheap CLI default, not promoted from organic majority.
- Threat model (S4): TermLink trusts root. Co-resident-forge is out of scope. Design B (signed metadata + sub-keys) over-engineers for an attack we don't defend against.
- Migration: 5 affected tasks (T-1427, T-1429, T-1436, T-1440, T-1441) all augment-not-unwind. T-1427 strict-reject still correctly identifies the host; project is a separate axis.

Cost: 3 build tasks, ~1 session each:
1. (a) termlink-cli: default `from_project` injection from `.context/working/focus.yaml` / `.framework.yaml`; add to T-1288 catalog
2. (b) T-1429 extension: `agent contact <name>[:project]` resolution; auto-attach `to_project`
3. (c) scripts + skills sync: `field-heartbeat.sh`, `vendored-arc-heartbeat.sh`, `/agent-handoff`, `/check-arc` — emit + read `from_project`

Order: a → c → b. (a) unblocks (c); (b) is the operator-visible payoff.

Reversibility: High. Each task is independent and reversible; metadata field stays in place even if catalog entry is rolled back.

Out of scope: sub-key cryptography (Design B); hub-side schema enforcement; renaming `_from`/`_thread`; cross-host project-namespace conflicts (e.g. two `050-email-archive` directories on different hosts) — flagged for follow-up.

Full report: `docs/reports/T-1448-co-resident-agent-identity-inception.md`

### 2026-05-02T20:09:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
