---
id: T-2971
name: "Project value review: delete / refactor / add across TermLink"
description: >
  Inception: Project value review: delete / refactor / add across TermLink

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-16T21:06:13Z
last_update: 2026-09-16T21:07:24Z
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
  - ts: '2026-09-16T21:07:24Z'
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

# T-2971: Project value review: delete / refactor / add across TermLink

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

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

The first four are the review prompt's own unfilled placeholders. They are recorded as
questions rather than guessed, because each one changes what evidence is even collected —
a wrong scope silently produces a confident review of the wrong thing.

- **IW-1: What is the review SCOPE — whole repo, or a named subsystem?**
  confidence: 1
  disposition:
  rationale:

- **IW-2: What is the authoritative PURPOSE SOURCE to judge value against?**
  confidence: 2
  disposition:
  rationale:
  note: `docs/CHARTER.md` is the strong candidate — T-2470 shipped it as "the single owned
  statement of what TermLink is", and T-2484 guards its canonical sentence against the
  README/ARCHITECTURE copies. `policy/value-drivers.yaml` carries the weighted drivers and
  is §ACD-sovereign. Proposing both; needs confirmation, since a yardstick nobody confirmed
  produces verdicts that are taste wearing evidence's clothes.

- **IW-3: What EXTERNAL DATA may this review use?**
  confidence: 0
  disposition:
  rationale:
  note: matters disproportionately here. Several axes (real use per verb, consumer breakage
  on DELETE) are only answerable from outside this repo — peer projects on the fleet, hub
  topic state on other hosts. Without it, DELETE confidence is capped and must be reported
  as capped rather than quietly downgraded to "no evidence of use".

- **IW-4: What is the session BUDGET, and is a partial review in slices acceptable?**
  confidence: 1
  disposition:
  rationale:

- **IW-5: Can GATHERER and JUDGE genuinely be separated on this host?**
  confidence: 2
  disposition:
  rationale:
  note: the prompt requires producer-not-judge and drops every confidence one level if the
  roles collapse. Sub-agents are available, so JUDGE can run as a fresh agent whose only
  input is the evidence file and the confirmed yardstick. A different MODEL FAMILY is not
  available here, so the separation is contextual, not architectural — recorded as a
  limitation rather than claimed as full independence.

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

GO on running the review, not on any outcome it proposes. The repo carries strong prior evidence that a value review has purchase: the T-2468 purpose review already found TermLink over-built in breadth and pruned 52 charter-untraceable tools, and 28 live tools remain acknowledged-but-unjustified in .context/checks/charter-drift-allowlist pending T-2548 — an open question about ~30 tools that has sat unresolved while a daily canary reports the surface clean. The guard layer has grown to 18 cron canaries plus 11+ static checks, several of which were found shipped-but-dark (T-2683, T-2696, T-2939), which is exactly the cost-without-value shape this review is meant to detect. Against that, the review is read-only through Phase 5 and creates no authorization to change anything, so the downside is bounded to review effort. The honest risk is the opposite one: that a DELETE axis run against a project whose charter is already narrow produces pressure to cut guards that are load-bearing but quiet. The prompt's own DELETE CHECKS and the 'non-use is a symptom, not a verdict' rule are the mitigation, and they are strict enough to rely on.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-09-16T21:07:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
