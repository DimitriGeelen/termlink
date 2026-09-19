---
id: T-2753
name: "hubs.toml has three independent parsers with no cross-check"
description: >
  hubs.toml is parsed by three independent implementations with nothing verifying
  they agree, so the CLI and MCP planes can disagree about which hubs exist for the
  same file. Same shape as the three artifact lists T-2751 closed. Options: cross-check,
  consolidate, or accept-and-document. Found while declining herdr rank 21 (T-2752).

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-15T21:38:37Z
last_update: 2026-09-18T19:39:48Z
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
  - ts: '2026-09-08T21:30:30Z'
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
cost_estimate_proposed:
  - ts: '2026-09-08T21:30:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=112,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2753: hubs.toml has three independent parsers with no cross-check

## Problem Statement

`~/.termlink/hubs.toml` — the operator's fleet-membership file — is parsed by three
independent implementations that share no code and no cross-check: (1) the CLI's typed
loader (`crates/termlink-cli/src/config.rs:115,138` — `toml::from_str` into `HubsConfig`,
with a round-trip unit test); (2) a hand-rolled substring parser in
`crates/termlink-mcp/src/tools.rs` ("Simple TOML parser for [hubs.NAME] sections",
tools.rs:7540, with the `[hubs.` prefix-scan repeated at tools.rs:7593 and again at
tools.rs:10962 — no section-parsing unit tests); (3) shell greppers — 32 scripts under
`scripts/` reference hubs.toml, a subset parsing `[hubs.*]` sections with grep/sed.
Path RESOLUTION was unified by T-2632 (CLI and MCP agree where the file lives even with
HOME unset); parsing was not. A TOML construct the typed loader accepts but the substring
parser mis-reads (quoted section names, trailing comments on a section header, CRLF)
would make the CLI and MCP disagree about fleet membership silently — the
plausible-wrong-answer class (Directive #2), on the file every fleet verb depends on.
Re-verified 2026-09-18; the original recommendation's "~11 shell greppers" undercounted
(32 referencing scripts).

## Assumptions

- A1 (validated 2026-09-18): the triplication exists as described — re-verified at
  config.rs:115/138, tools.rs:7540/7593/10962, and 32 hubs.toml-referencing scripts.
- A2 (unvalidated — the DEFER driver): the tools.rs hand-parser exists to avoid a crate
  dependency; nobody has verified that rationale, so consolidating blind could be wrong.

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

- **IW-1: Do the three parsers disagree on any live profile set today?**
  confidence: 2
  disposition: answered
  rationale: No divergence observed or reported anywhere — no CLI-vs-MCP fleet-membership discrepancy in any audit, canary log, or task since the tools.rs parser landed; the risk is structural, not a live defect.

- **IW-2: Is the right remedy a cross-parity check or consolidation onto the CLI's typed loader?**
  confidence: 1
  disposition: deferred
  rationale: Undecidable without either an observed divergence (which names the failing TOML construct) or a verified answer on why tools.rs hand-rolls the parse instead of using the toml crate (dependency-avoidance rationale unverified — A2). Revisit trigger recorded in Recommendation.

## Exploration Plan

Completed as a measurement pass (no spike needed): locate every parse site
(`grep -rn "hubs.toml" crates/ scripts/`), classify by mechanism (typed crate /
substring scan / shell grep), and check for an existing cross-check (none found).

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: establishing whether the triplication is real, measuring its surface, and deciding
whether a remedy is warranted now. OUT: implementing any consolidation or parity check
(a build task after a GO), and the vendored framework's own config reads (G-062).

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

**Recommendation:** DEFER

**Rationale:** The triplication is real and measured (toml-crate loader in termlink-cli/src/config.rs, hand-rolled substring parser in termlink-mcp/src/tools.rs with no section-parsing unit tests, ~11 shell greppers, feeding ~24 Rust call sites and ~15 MCP paths), but no divergence has been OBSERVED — the risk is structural, not a reported defect. DEFER rather than GO because the right remedy is genuinely undecided: a cross-check asserting all three see the same profile set may be cheaper and safer than consolidating MCP onto the CLI loader, and consolidating blind could remove a dependency-avoidance rationale nobody has verified. Revisit when a CLI-vs-MCP fleet-membership disagreement is actually observed, or when hubs.toml gains a new field (the moment three parsers must change together).

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

### 2026-09-18T18:45:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
