---
id: T-2694
name: "TermLink purpose review #5 — are the charter's POSITIVE claims true and proven?"
description: >
  Inception: TermLink purpose review #5 — are the charter's POSITIVE claims true and
  proven?

status: work-completed
workflow_type: inception
owner: human
horizon:
tags: []
components: []
related_tasks: []
created: 2026-08-14T07:59:44Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-16T14:11:43Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:56Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-2694: TermLink purpose review #5 — are the charter's POSITIVE claims true and proven?

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

Research artifact: `docs/reports/T-2694-positive-claims-review.md` (C-001).

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

- **IW-1: Has any review examined the charter's POSITIVE claims?**
  confidence: 3
  disposition: answered
  rationale: No. T-2678 built a claim-vs-guard matrix for the five **non-goals**;
  reviews 1–3 measured Directives #1/#2 and review 4 measured #3/#4. The "load-bearing
  nouns" section — the four assertions of what TermLink *is* — has never been the
  subject of a pass.

- **IW-2: Is the "append-log" claim true, given `channel edit` and `channel redact`?**
  confidence: 3
  disposition: dissolved
  rationale: True, and honestly documented. Both post a NEW envelope carrying
  `metadata.replaces` / `metadata.redacts`; the hub keeps the original; the verb is
  named *Retract* not delete; `channel redactions` even previews the retained payload.
  The suspicion dissolves — recorded as an explicit non-finding for calibration.

- **IW-3: Noun #3 claims four PTY capabilities — how many are proven?**
  confidence: 3
  disposition: answered
  rationale: One. `session-selftest.sh` exercises only `termlink exec` (grep: zero
  `inject`, zero `output`). `inject`'s unit tests are named
  `command_inject_resolves_keys_no_pty` — key resolution *without* a PTY. Remaining
  `inject` callers are demo/benchmark scripts. The prover's own header quotes the gap
  and does not close it.

- **IW-4: Are the four charter-verb provers actually executed?**
  confidence: 3
  disposition: answered
  rationale: Two of four are not. `comms-selftest.sh` (verbs 1+2) and
  `substrate-smoke.sh` (verb 3) appear only in comments, docs, handovers and task
  files — no cron, no CI, no check invokes either. Only `session-selftest` is wired,
  via T-2557's canary.

- **IW-5: Can an INJECT prover be built and verified from this host?**
  confidence: 3
  disposition: answered
  rationale: Yes. `tmux` is present and `session-selftest.sh --json` currently returns
  `proven:true` here, so a new stage can be proven end-to-end rather than written
  speculatively — unlike the macOS work in T-2692, which had to ship non-blocking.

- **IW-6: Should the two unexecuted provers be wired to canaries in this pass?**
  confidence: 2
  disposition: deferred
  rationale: `comms-selftest` needs a live *peer* to prove a round-trip and
  `substrate-smoke` needs a reachable hub; neither is guaranteed on an arbitrary host,
  so a cron wiring would fire on absence rather than breakage — the exact
  false-positive class T-2557 avoided by splitting exit 2 from exit 1. Filed rather
  than built.

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

Fifth critical pass. Reviews 1-3 measured Directives #1/#2, review 4 measured #3/#4. What no pass has examined is the charter's POSITIVE claims — its 'load-bearing nouns' section. T-2678 built a claim-vs-guard matrix for the five NON-GOALS; nobody has built one for the four assertions of what TermLink IS. Doing so produced two findings and one explicit non-finding. (F1) Noun #3 asserts 'sessions are real PTYs: peers can stream output, inject keystrokes, exec, and doorbell-wake them, not just exchange text' — four capability claims. scripts/session-selftest.sh, the prover for the FOUNDING verb, exercises only 'termlink exec'; grep confirms zero use of inject or output. inject's unit tests are explicitly named command_inject_resolves_keys_no_pty — they prove key-name resolution WITHOUT a PTY, not that a keystroke reaches a live PTY and takes effect; the remaining inject callers are demo and benchmark scripts. The prover's own header even quotes the acknowledged gap ('agent-conversation-selftest.sh says: What it does NOT validate: PTY inject') and then does not close it. So the founding noun is proven 1 of 4. (F2) Two of the four charter-verb provers are never executed by anything: comms-selftest.sh (verbs 1+2) and substrate-smoke.sh (verb 3) are referenced only in comments, docs, handovers and task files — no cron, no CI, no check invokes either; only session-selftest is wired, via T-2557's canary. (NOT-A-FINDING, recorded for calibration) Noun #1's 'append-log' claim is TRUE and honestly documented: channel edit and redact post NEW envelopes carrying metadata.replaces / metadata.redacts, the hub keeps the original, the verb is named Retract rather than delete, and the audit tool even previews the retained payload. GO on the in-authority subset: close the founding verb's proof gap with real INJECT and OUTPUT stages, which are verifiable on this host (tmux present, session-selftest currently passes here).

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

Fifth critical pass. Reviews 1-3 measured Directives #1/#2, review 4 measured #3/#4. What no pass has examined is the charter's POSITIVE claims — its 'load-bearing nouns' section. T-2678 built a claim-vs-guard matrix for the five NON-GOALS; nobody has built one for the four assertions of what TermLink IS. Doing so produced two findings and one explicit non-finding. (F1) Noun #3 asserts 'sessions are real PTYs: peers can stream output, inject keystrokes, exec, and doorbell-wake them, not just exchange text' — four capability claims. scripts/session-selftest.sh, the prover for the FOUNDING verb, exercises only 'termlink exec'; grep confirms zero use of inject or output. inject's unit tests are explicitly named command_inject_resolves_keys_no_pty — they prove key-name resolution WITHOUT a PTY, not that a keystroke reaches a live PTY and takes effect; the remaining inject callers are demo and benchmark scripts. The prover's own header even quotes the acknowledged gap ('agent-conversation-selftest.sh says: What it does NOT validate: PTY inject') and then does not close it. So the founding noun is proven 1 of 4. (F2) Two of the four charter-verb provers are never executed by anything: comms-selftest.sh (verbs 1+2) and substrate-smoke.sh (verb 3) are referenced only in comments, docs, handovers and task files — no cron, no CI, no check invokes either; only session-selftest is wired, via T-2557's canary. (NOT-A-FINDING, recorded for calibration) Noun #1's 'append-log' claim is TRUE and honestly documented: channel edit and redact post NEW envelopes carrying metadata.replaces / metadata.redacts, the hub keeps the original, the verb is named Retract rather than delete, and the audit tool even previews the retained payload. GO on the in-authority subset: close the founding verb's proof gap with real INJECT and OUTPUT stages, which are verifiable on this host (tmux present, session-selftest currently passes here).

Evidence:

**Date**: 2026-08-16T14:11:43Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-14T07:59:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-16T14:11:43Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

Fifth critical pass. Reviews 1-3 measured Directives #1/#2, review 4 measured #3/#4. What no pass has examined is the charter's POSITIVE claims — its 'load-bearing nouns' section. T-2678 built a claim-vs-guard matrix for the five NON-GOALS; nobody has built one for the four assertions of what TermLink IS. Doing so produced two findings and one explicit non-finding. (F1) Noun #3 asserts 'sessions are real PTYs: peers can stream output, inject keystrokes, exec, and doorbell-wake them, not just exchange text' — four capability claims. scripts/session-selftest.sh, the prover for the FOUNDING verb, exercises only 'termlink exec'; grep confirms zero use of inject or output. inject's unit tests are explicitly named command_inject_resolves_keys_no_pty — they prove key-name resolution WITHOUT a PTY, not that a keystroke reaches a live PTY and takes effect; the remaining inject callers are demo and benchmark scripts. The prover's own header even quotes the acknowledged gap ('agent-conversation-selftest.sh says: What it does NOT validate: PTY inject') and then does not close it. So the founding noun is proven 1 of 4. (F2) Two of the four charter-verb provers are never executed by anything: comms-selftest.sh (verbs 1+2) and substrate-smoke.sh (verb 3) are referenced only in comments, docs, handovers and task files — no cron, no CI, no check invokes either; only session-selftest is wired, via T-2557's canary. (NOT-A-FINDING, recorded for calibration) Noun #1's 'append-log' claim is TRUE and honestly documented: channel edit and redact post NEW envelopes carrying metadata.replaces / metadata.redacts, the hub keeps the original, the verb is named Retract rather than delete, and the audit tool even previews the retained payload. GO on the in-authority subset: close the founding verb's proof gap with real INJECT and OUTPUT stages, which are verifiable on this host (tmux present, session-selftest currently passes here).

Evidence:

### 2026-08-16T14:11:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
