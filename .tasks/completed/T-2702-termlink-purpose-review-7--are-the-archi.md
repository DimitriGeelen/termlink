---
id: T-2702
name: "TermLink purpose review #7 — are the architecture doc's declared invariants enforced?"
description: >
  Inception: TermLink purpose review #7 — are the architecture doc's declared invariants enforced?

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-08-14T11:26:58Z
last_update: 2026-08-16T14:12:19Z
date_finished: 2026-08-16T14:12:19Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2702: TermLink purpose review #7 — are the architecture doc's declared invariants enforced?

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

Research artifact: `docs/reports/T-2702-architecture-invariants-review.md` (C-001).

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Has the document the charter delegates authority to ever been audited?**
  confidence: 3
  disposition: answered
  rationale: No. The charter names `docs/architecture/parallel-execution-substrate.md`
  "the authoritative statement of the substrate design and its invariants"; six reviews
  audited the charter, the guards, the platforms, the capability claims and the error
  taxonomy, and none opened this file's §10.

- **IW-2: Does T-2569's tripwire guard the "spokes never connect to one another" invariant?**
  confidence: 3
  disposition: answered
  rationale: No — it guards a DIFFERENT EDGE. `no_federation_tripwire.rs` scans only
  `CARGO_MANIFEST_DIR/src` (the hub crate) and forbids the hub building a hub-speaking
  client: that is hub↔hub federation, charter non-goal #1. The invariant is spoke↔spoke
  mesh. Nothing guards it.

- **IW-3: Is a spoke→spoke channel actually constructible today?**
  confidence: 3
  disposition: answered
  rationale: Yes. `termlink-session/src/client.rs` is a generic RPC client that connects
  to a unix path or a TCP host:port; nothing constrains the target. A direct
  agent-to-agent channel could be added in `termlink-session` or `termlink-cli` and no
  test would fail — the same capability shape T-2569 was built to deny the hub.

- **IW-4: Is "producer ≠ judge at the seam" enforced?**
  confidence: 3
  disposition: answered
  rationale: It is **unfalsifiable as written**. §9 declares neither side self-certifies
  and the AEF layer signs off each hard dependency — while the same section rejects a
  sign-off ceremony as "enterprise scaffolding for a workshop". Zero sign-off records
  exist; the only occurrence of the phrase in the doc is the sentence rejecting it. The
  cross-repo seam channel was 36-sent / 0-received (G-063).

- **IW-5: Is append-log ordering/durability unguarded?**
  confidence: 3
  disposition: dissolved
  rationale: Not supported — 88 tests in `termlink-bus/src/lib.rs`, monotonic cursor
  semantics documented and asserted across `meta.rs`. Investigated and cleared rather
  than counted.

- **IW-6: Is the CLI connecting to a local session socket a mesh violation?**
  confidence: 3
  disposition: dissolved
  rationale: No. §3 forbids *spokes connecting to one another* — an agent-to-agent mesh.
  An operator tool reaching a local session over a unix socket is not that. Considered
  and rejected rather than inflated into a finding.

- **IW-7: Should F2 be fixed in this pass?**
  confidence: 3
  disposition: deferred
  rationale: No. It is a change to an authoritative design document, and the underlying
  question (what evidence the cross-repo seam should produce) is cross-repo governance —
  G-062 territory, human-sovereign. Recorded, not edited.

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

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

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

Seventh pass. Prior axes: breadth (T-2468), non-goal guards (T-2678), guard execution (T-2683), Usability+Portability (T-2690), positive claims vs provers (T-2694), refusal taxonomy (T-2698). This pass audits the one document the charter explicitly DELEGATES AUTHORITY TO — docs/architecture/parallel-execution-substrate.md, which the charter calls 'the authoritative statement of the substrate design and its invariants'. Its section 10 declares five invariants 'must not be violated'. Two findings. (F1) The decisive one — 'Strict star; spokes never connect to one another' — is guarded by NOTHING. T-2569's tripwire looks like it covers this but guards a DIFFERENT EDGE: it scans only crates/termlink-hub/src and forbids the HUB from building a hub-speaking client, i.e. hub-to-hub federation, which is charter non-goal #1. The invariant here is spoke-to-spoke mesh, and termlink-session ships a generic client (client.rs) that connects to either a unix path or a TCP host:port with nothing constraining the target, so a direct spoke-to-spoke channel could be introduced in termlink-session or termlink-cli and no test would fail. Section 3 spends forty lines rejecting exactly that mesh and calls the fragility argument decisive. (F2) 'Producer != judge at the seam' is UNFALSIFIABLE AS WRITTEN. Section 9 declares that neither side self-certifies the boundary and the AEF layer signs off each hard-dependency primitive — and the same section explicitly rejects a sign-off ceremony as 'enterprise scaffolding for a workshop'. Grep finds zero sign-off records anywhere; the only occurrence of the phrase in the doc is the sentence rejecting the ceremony. CLAUDE.md separately records G-063: the cross-repo seam channel framework:pickup sat at 36-sent / 0-received, a write-only sink. An invariant whose only possible evidence was deliberately removed is not an invariant, it is a value. Also checked and NOT findings, recorded for calibration: append-log ordering/durability is well covered (88 tests in termlink-bus lib.rs, monotonic cursor semantics documented across meta.rs), and the CLI connecting directly to a LOCAL session socket is an operator tool reaching a local session, not the agent-to-agent mesh the invariant forbids — both were investigated and cleared rather than counted. GO on F1 which is buildable and testable here; F2 is a doc/governance change to an authoritative artifact and is human-sovereign.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

Seventh pass. Prior axes: breadth (T-2468), non-goal guards (T-2678), guard execution (T-2683), Usability+Portability (T-2690), positive claims vs provers (T-2694), refusal taxonomy (T-2698). This pass audits the one document the charter explicitly DELEGATES AUTHORITY TO — docs/architecture/parallel-execution-substrate.md, which the charter calls 'the authoritative statement of the substrate design and its invariants'. Its section 10 declares five invariants 'must not be violated'. Two findings. (F1) The decisive one — 'Strict star; spokes never connect to one another' — is guarded by NOTHING. T-2569's tripwire looks like it covers this but guards a DIFFERENT EDGE: it scans only crates/termlink-hub/src and forbids the HUB from building a hub-speaking client, i.e. hub-to-hub federation, which is charter non-goal #1. The invariant here is spoke-to-spoke mesh, and termlink-session ships a generic client (client.rs) that connects to either a unix path or a TCP host:port with nothing constraining the target, so a direct spoke-to-spoke channel could be introduced in termlink-session or termlink-cli and no test would fail. Section 3 spends forty lines rejecting exactly that mesh and calls the fragility argument decisive. (F2) 'Producer != judge at the seam' is UNFALSIFIABLE AS WRITTEN. Section 9 declares that neither side self-certifies the boundary and the AEF layer signs off each hard-dependency primitive — and the same section explicitly rejects a sign-off ceremony as 'enterprise scaffolding for a workshop'. Grep finds zero sign-off records anywhere; the only occurrence of the phrase in the doc is the sentence rejecting the ceremony. CLAUDE.md separately records G-063: the cross-repo seam channel framework:pickup sat at 36-sent / 0-received, a write-only sink. An invariant whose only possible evidence was deliberately removed is not an invariant, it is a value. Also checked and NOT findings, recorded for calibration: append-log ordering/durability is well covered (88 tests in termlink-bus lib.rs, monotonic cursor semantics documented across meta.rs), and the CLI connecting directly to a LOCAL session socket is an operator tool reaching a local session, not the agent-to-agent mesh the invariant forbids — both were investigated and cleared rather than counted. GO on F1 which is buildable and testable here; F2 is a doc/governance change to an authoritative artifact and is human-sovereign.

Evidence:

**Date**: 2026-08-16T14:12:18Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-14T11:27:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-16T14:12:18Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

Seventh pass. Prior axes: breadth (T-2468), non-goal guards (T-2678), guard execution (T-2683), Usability+Portability (T-2690), positive claims vs provers (T-2694), refusal taxonomy (T-2698). This pass audits the one document the charter explicitly DELEGATES AUTHORITY TO — docs/architecture/parallel-execution-substrate.md, which the charter calls 'the authoritative statement of the substrate design and its invariants'. Its section 10 declares five invariants 'must not be violated'. Two findings. (F1) The decisive one — 'Strict star; spokes never connect to one another' — is guarded by NOTHING. T-2569's tripwire looks like it covers this but guards a DIFFERENT EDGE: it scans only crates/termlink-hub/src and forbids the HUB from building a hub-speaking client, i.e. hub-to-hub federation, which is charter non-goal #1. The invariant here is spoke-to-spoke mesh, and termlink-session ships a generic client (client.rs) that connects to either a unix path or a TCP host:port with nothing constraining the target, so a direct spoke-to-spoke channel could be introduced in termlink-session or termlink-cli and no test would fail. Section 3 spends forty lines rejecting exactly that mesh and calls the fragility argument decisive. (F2) 'Producer != judge at the seam' is UNFALSIFIABLE AS WRITTEN. Section 9 declares that neither side self-certifies the boundary and the AEF layer signs off each hard-dependency primitive — and the same section explicitly rejects a sign-off ceremony as 'enterprise scaffolding for a workshop'. Grep finds zero sign-off records anywhere; the only occurrence of the phrase in the doc is the sentence rejecting the ceremony. CLAUDE.md separately records G-063: the cross-repo seam channel framework:pickup sat at 36-sent / 0-received, a write-only sink. An invariant whose only possible evidence was deliberately removed is not an invariant, it is a value. Also checked and NOT findings, recorded for calibration: append-log ordering/durability is well covered (88 tests in termlink-bus lib.rs, monotonic cursor semantics documented across meta.rs), and the CLI connecting directly to a LOCAL session socket is an operator tool reaching a local session, not the agent-to-agent mesh the invariant forbids — both were investigated and cleared rather than counted. GO on F1 which is buildable and testable here; F2 is a doc/governance change to an authoritative artifact and is human-sovereign.

Evidence:

### 2026-08-16T14:12:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
