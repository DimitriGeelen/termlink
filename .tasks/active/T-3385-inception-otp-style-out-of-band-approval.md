---
id: T-3385
name: "Inception: OTP-style out-of-band approval confirmation (pluggable provider,
  ring20-manager facility)"
description: >
  Explore and decide: (1) the confirmation-provider contract (ask with digest and
  human summary; verify once against that digest; per-installation configuration as
  a signed policy record, CR-19; changing the provider is high-impact; fail closed;
  prefer remote verification so secrets stay off agent hosts, RR-2); (2) which OTP
  engines to support (TOTP app, push approval, hardware key); (3) the facility ring20-manager
  builds in the Cloudron estate, designed together with ring20-manager. Source: T-3351
  OQ-14 ruling.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-09T23:47:58Z
last_update: 2026-10-10T18:17:33Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-10-10T18:17:33Z'
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
    rubric_sha: e4a00f38e801
---

# T-3385: Inception: OTP-style out-of-band approval confirmation (pluggable provider, ring20-manager facility)

## Problem Statement

High-impact approvals (OQ-14 D‴, R-76) need an active out-of-band confirmation bound to the approval, never
silence. Today only the interim exists (double approval at the terminal, which is BP-13, the weak point). Research
artifact: `docs/reports/T-3385-otp-confirmation.md` (ruled constraints, contract draft, candidate engines,
questions for ring20-manager and the operator, dialogue log).

## Assumptions

1. A remote verifier on the Cloudron estate can keep the confirmation secret off agent hosts (to confirm with
   ring20-manager).
2. A push or WebAuthn engine binds to the approval digest more naturally than plain TOTP (to research).

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

- **IW-1: Which engine(s) bind the confirmation to the approval digest well enough (TOTP, push with summary,
  WebAuthn/FIDO2, an estate IdP step-up)?**
  confidence: 1
  disposition:
  rationale:
- **IW-2: What can ring20-manager build on the Cloudron estate, with remote verification so no secret sits on
  agent hosts, and on what timescale?**
  confidence: 0
  disposition:
  rationale:
- **IW-3: What is the confirmation-provider contract (ask, verify once, status, configuration as a signed policy
  record, audit), and does it stay vendor- and engine-agnostic?**
  confidence: 1
  disposition:
  rationale:
- **IW-4: Which device(s) does the operator want to confirm on, and must the channel be independent of the estate
  login?** (operator question, asked one at a time later)
  confidence: 0
  disposition:
  rationale:

## Exploration Plan

1. Consult ring20-manager (sent 2026-10-10 to proxmox-ring20-management, conversation t3385-otp-confirmation).
2. Survey candidate engines against the contract and the ruled constraints (artifact sections 2-4).
3. Draft the confirmation-provider contract; one spike only if binding cannot be settled on paper.
4. Present go/no-go to the operator with a recommendation.

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
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
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

**Rationale:** Operator ruled OQ-14 D‴ on 2026-10-10 (T-3351 Decisions): high-impact approvals need an active second out-of-band confirmation and never proceed on silence; the operator wants OTP-style confirmation at high priority, with TermLink owning a pluggable confirmation-provider contract and its configuration (different framework users run different OTP engines) and ring20-manager building the facility for the Cloudron estate in close cooperation. Until it exists the interim is double approval at the terminal, which keeps the terminal (BP-13) as the weak point; this inception closes that and feeds GP-11 and CR-14.

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

### 2026-10-10T18:17:33Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
