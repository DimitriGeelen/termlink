---
id: T-2690
name: "TermLink purpose review #4 — the two Constitutional Directives no review has
  examined (Usability, Portability)"
description: >
  Inception: TermLink purpose review #4 — the two Constitutional Directives no review
  has examined (Usability, Portability)

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: [scripts/canary-status.sh, scripts/check-cron-install-drift.sh]
related_tasks: []
created: 2026-08-14T07:25:00Z
last_update: 2026-08-23T21:13:32Z
date_finished: 2026-08-23T21:13:32Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-08-23T19:13:28Z'
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
  - ts: '2026-08-23T19:13:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=7 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2690: TermLink purpose review #4 — the two Constitutional Directives no review has examined (Usability, Portability)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

Research artifact: `docs/reports/T-2690-usability-portability-review.md` (C-001).

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

- **IW-1: Which Constitutional Directives has the review series actually examined?**
  confidence: 3
  disposition: answered
  rationale: Only #1 and #2. T-2468 (breadth vs charter), T-2678 (non-goal guards),
  T-2683 (guard execution) all sit under Antifragility/Reliability. #3 Usability and
  #4 Portability have never been the subject of a pass — a blind spot in the *review
  series*, not just the product.

- **IW-2: Is macOS a supported platform, and is that support verified?**
  confidence: 3
  disposition: answered
  rationale: Supported and *recommended* — README §Platform Support asserts `Yes`
  five times (core binary, PTY ops, Terminal.app spawn, tmux spawn, TCP hub) and names
  Homebrew the recommended macOS install. Verified: **no**. Every `runs-on` is
  `ubuntu-latest` except release.yml's build matrix; no test, install-check, or smoke
  job ever runs on macOS.

- **IW-3: Does anything break on macOS, or is it merely untested?**
  confidence: 3
  disposition: answered
  rationale: It breaks, silently. `read_ppid_from_proc` reads `/proc/<pid>/stat` with
  `.ok()?` on BOTH surfaces (`metadata.rs:782` CLI, `tools.rs:7935` MCP — a documented
  duplicate). On macOS the ancestor chain collapses to `[self]`, so `whoami` always
  reports `ambiguous` with every candidate listed and no indication that
  auto-resolution is structurally impossible there.

- **IW-4: Should macOS CI be added as a blocking gate?**
  confidence: 2
  disposition: answered
  rationale: No — not as the first move. I cannot execute macOS from this host, so
  gating a release on a suite whose macOS result is unknown would violate T-2686's own
  AC ("only add a gate once the suite is actually green"). Non-blocking first makes the
  truth visible without breaking releases on an unverified assumption.

- **IW-5: Is the 214-tool live surface a Directive #3 (Usability) finding?**
  confidence: 2
  disposition: deferred
  rationale: Arguably yes, but it is the same surface T-2548 (`owner: human`) already
  owns as a subtract-or-keep decision. Re-litigating it under a Usability banner would
  be the same question in new clothes, so it is explicitly out of scope here.

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

Fourth critical pass. The series has now examined product-vs-charter (T-2468), charter-vs-guards (T-2678), and guard-execution (T-2683) — all of which live under Directives #1 Antifragility and #2 Reliability. Directives #3 Usability and #4 Portability have never been reviewed at all, which is itself the finding about the review series. Two confirmed live findings on that axis: (F1) macOS is a first-class documented platform — README carries a formal Platform Support table asserting Yes for core binary, PTY operations, Terminal.app spawn, tmux spawn and TCP hub, and names Homebrew the recommended macOS install — yet EVERY CI runner is ubuntu-latest except the build matrix itself, so macOS binaries are cross-built, published to GitHub Releases and Homebrew, and never have a single test executed against them; this is the G-069 shipped-not-verified class on a fresh axis, and my own T-2686 test job inherited the same blind spot. (F2) identity auto-resolution is /proc-dependent on BOTH the CLI (metadata.rs walk_ancestor_pids) and MCP (tools.rs whoami_helpers, a documented duplicate) surfaces; read_ppid_from_proc uses .ok()? so on macOS the ancestor chain silently collapses to just self, whoami always falls through to ambiguous-here-are-all-candidates, and nothing tells the user auto-resolution is structurally impossible on their platform — Directive #4 crossed with #2 no-silent-failures and #3 actionable-errors. GO on the in-authority subset: name the platform limitation at both surfaces, make macOS verification visible in CI without gating a release on an unverified suite, and make the portability convention load-bearing via a check the T-2684 runner already executes.

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

Fourth critical pass. The series has now examined product-vs-charter (T-2468), charter-vs-guards (T-2678), and guard-execution (T-2683) — all of which live under Directives #1 Antifragility and #2 Reliability. Directives #3 Usability and #4 Portability have never been reviewed at all, which is itself the finding about the review series. Two confirmed live findings on that axis: (F1) macOS is a first-class documented platform — README carries a formal Platform Support table asserting Yes for core binary, PTY operations, Terminal.app spawn, tmux spawn and TCP hub, and names Homebrew the recommended macOS install — yet EVERY CI runner is ubuntu-latest except the build matrix itself, so macOS binaries are cross-built, published to GitHub Releases and Homebrew, and never have a single test executed against them; this is the G-069 shipped-not-verified class on a fresh axis, and my own T-2686 test job inherited the same blind spot. (F2) identity auto-resolution is /proc-dependent on BOTH the CLI (metadata.rs walk_ancestor_pids) and MCP (tools.rs whoami_helpers, a documented duplicate) surfaces; read_ppid_from_proc uses .ok()? so on macOS the ancestor chain silently collapses to just self, whoami always falls through to ambiguous-here-are-all-candidates, and nothing tells the user auto-resolution is structurally impossible on their platform — Directive #4 crossed with #2 no-silent-failures and #3 actionable-errors. GO on the in-authority subset: name the platform limitation at both surfaces, make macOS verification visible in CI without gating a release on an unverified suite, and make the portability convention load-bearing via a check the T-2684 runner already executes.

Evidence:

**Date**: 2026-08-23T21:13:32Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-14T07:25:15Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-23T21:13:32Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

Fourth critical pass. The series has now examined product-vs-charter (T-2468), charter-vs-guards (T-2678), and guard-execution (T-2683) — all of which live under Directives #1 Antifragility and #2 Reliability. Directives #3 Usability and #4 Portability have never been reviewed at all, which is itself the finding about the review series. Two confirmed live findings on that axis: (F1) macOS is a first-class documented platform — README carries a formal Platform Support table asserting Yes for core binary, PTY operations, Terminal.app spawn, tmux spawn and TCP hub, and names Homebrew the recommended macOS install — yet EVERY CI runner is ubuntu-latest except the build matrix itself, so macOS binaries are cross-built, published to GitHub Releases and Homebrew, and never have a single test executed against them; this is the G-069 shipped-not-verified class on a fresh axis, and my own T-2686 test job inherited the same blind spot. (F2) identity auto-resolution is /proc-dependent on BOTH the CLI (metadata.rs walk_ancestor_pids) and MCP (tools.rs whoami_helpers, a documented duplicate) surfaces; read_ppid_from_proc uses .ok()? so on macOS the ancestor chain silently collapses to just self, whoami always falls through to ambiguous-here-are-all-candidates, and nothing tells the user auto-resolution is structurally impossible on their platform — Directive #4 crossed with #2 no-silent-failures and #3 actionable-errors. GO on the in-authority subset: name the platform limitation at both surfaces, make macOS verification visible in CI without gating a release on an unverified suite, and make the portability convention load-bearing via a check the T-2684 runner already executes.

Evidence:

## Reviewer Verdict (v1.5)

- **Scan ID:** R-bbfe9cbd
- **Timestamp:** 2026-08-23T21:13:33Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-2
     - evidence: `IW-2 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

## Recommendation Verdict (v1.0)

- **Scan ID:** RC-88a083fa
- **Timestamp:** 2026-08-23T21:13:33Z
- **Overall:** CONFIRMED
- **Claims:** 5

| Claim | Type | Status |
|-------|------|--------|
| `T-2468` | task | ✓ pass |
| `T-2678` | task | ✓ pass |
| `T-2683` | task | ✓ pass |
| `T-2686` | task | ✓ pass |
| `T-2684` | task | ✓ pass |

### 2026-08-23T21:13:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
