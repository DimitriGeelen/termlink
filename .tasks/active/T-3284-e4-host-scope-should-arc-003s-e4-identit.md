---
id: T-3284
name: "E4 host scope: should arc-003's E4 identity-split claim assert only on hosts
  whose sidecars are declared"
description: >
  Inception: E4 host scope: should arc-003's E4 identity-split claim assert only on
  hosts whose sidecars are declared

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-30T17:52:09Z
last_update: 2026-09-30T17:52:30Z
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
  - ts: '2026-09-30T17:52:31Z'
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

# T-3284: E4 host scope: should arc-003's E4 identity-split claim assert only on hosts whose sidecars are declared

## Problem Statement

arc-003 ("reliable comms, no silent loss") is closed, and its claim is re-checked by a bound prover: `notify-rail-e2e.sh --stages '' --experiment e4`, run by `check-arc-claim-drift` (FAIL tier) in the framework guard layer. E4 resolves this agent's identity (`TERMLINK_AGENT_ID=claude-termlink termlink agent identity --resolve`) and asserts that a sidecar declared in `.context/cron/notify-sidecar-agents.conf` watches that fingerprint. T-3283 measured that on a host with no identity store, `resolve` MINTS a new key and returns ok:true. So on any host other than the one the conf describes, E4 reports IDENTITY SPLIT (exit 1, a failed claim) and writes a key as a side effect. The operator asked for a full analysis before deciding: background, implications, steelman/strawman, directive scoring, exposure. Research artifact: `docs/reports/T-3284-e4-host-scope-analysis.md`.

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

- **IW-1: Exposure. In which environments that actually run the guard layer can E4 reach the mint → IDENTITY SPLIT path today?**
  confidence: 3
  disposition: answered
  rationale: This host if claude-termlink.key is missing (E2: the check silently mints the agent's signing key); any other machine running this guard layer with termlink installed (E3, no known instance); CI only if termlink lands on the PATH (E5). Today's CI gates SKIP (F6/F7). docs/reports/T-3284-e4-host-scope-analysis.md §3
- **IW-2: Is E4's claim a property of the software (portable) or of one deployment (host-specific)?**
  confidence: 3
  disposition: answered
  rationale: Deployment-specific: the conf lists .107 fingerprints with no host column (F3), and resolve depends on the host's key store (F2). The claim is 'no unwatched mailbox for agents that exist HERE'
- **IW-3: Is the key-minting side effect of a check acceptable under Directive 2 (no silent state change)?**
  confidence: 3
  disposition: answered
  rationale: No: a check writing the signing identity it verifies breaks Directive 2 and can CAUSE the silent loss arc-003 forbids (E2); probes already accumulate keys (F5)
- **IW-4: Which option scores best against the four directives: status quo, host-scoped assertion, a read-only identity probe, or moving E4 out of the release gate?**
  confidence: 2
  disposition: answered
  rationale: Directive scores out of 12: A 3, B 4, C 12, D 6. C (read-only --no-create probe) is the only option that removes E2 and needs no host list (§6)

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
- The mint-on-resolve behaviour creates a real exposure in an environment we actually run (not only hypothetical machines)
- A fix exists that removes it without a hard-coded host list, and is additive and testable on disk

**NO-GO if:**
- The only exposure is hypothetical remote hosts (then record it and park)
- The fix needs a breaking CLI change or re-implements the identity precedence outside the resolver

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

GO on option C. Make the identity check read-only: `termlink agent identity --resolve --no-create`, and E4 reports "no identity for this agent here — nothing to verify" (exit 2) instead of minting the key it verifies. It is the only option that removes the real exposure (E2: on THIS host a missing `claude-termlink.key` is silently re-minted by the check, changing the agent's signing identity, which is the silent loss arc-003 forbids), and it needs no hard-coded host list. It scores 12/12 against the four directives (status quo 3, host list 4, move out of gate 6). Two small build tasks; additive CLI flag; fully testable on disk.

**Evidence:**

- `docs/reports/T-3284-e4-host-scope-analysis.md`: F1–F7, exposure table E1–E5, steelman/strawman, directive scoring
- `crates/termlink-cli/src/commands/identity.rs:57`: `--resolve` calls `load_identity_or_create()`
- probe: empty `$HOME` → `{"action":"resolved","fingerprint":"8eba7886a7701de8","ok":true}` rc 0 (a key was created)
- `.context/cron/notify-sidecar-agents.conf`: deployment-specific fps, no host column; label drift (F4)

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

### 2026-09-30 — operator ruling: GO on option C
Operator, in session: "Okay, let's go with C." The formal decision command is Tier 0 (human authority), so it is carried out by the operator running `/opt/termlink/runme.sh` (approved-decision action, T-3285). The build tasks start only after the decision is on record.

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-09-30T17:52:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
