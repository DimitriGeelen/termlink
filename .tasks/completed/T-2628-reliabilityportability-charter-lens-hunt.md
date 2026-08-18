---
id: T-2628
name: "reliability/portability charter-lens hunt round 3 — adversarial defect sweep"
description: >
  reliability/portability charter-lens hunt round 3 — adversarial defect sweep

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T06:16:52Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-12T09:39:23Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:54Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 5
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=5 (body:silent-class-removed); 
      D3=2 (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2628: reliability/portability charter-lens hunt round 3 — adversarial defect sweep

## Context

Round-3 adversarial defect sweep of the T-2468 "subtract-and-deepen" campaign.
Prior rounds cleared the reliability lens (T-2619/T-2621/T-2623/T-2624 —
0/None-laundering silent-failure class) and the usability lens
(T-2625/T-2626/T-2627 — actionable errors + surprising defaults). This task is
the tracking container for a fresh hunt on an un-swept charter lens
(Directive #1 Antifragility / Directive #4 Portability / the discover-peers
verb). A subagent hunter reads the code and returns a ranked findings report;
each verified finding is either BUILT (small/clean/testable) or FILED as its own
one-bug-one-task with RCA + real ACs.

## Acceptance Criteria

### Agent
- [x] A subagent hunter sweeps an un-swept charter lens and returns a ranked findings report
- [x] Each reported finding is VERIFIED in code by the orchestrator (never trust the hunter) — false positives discarded with a one-line reason
- [x] Each verified finding is either BUILT (with a load-bearing test proven via temp-revert) OR FILED as its own task (one-bug-one-task) with full RCA + real ACs + concrete failure scenario
- [x] Every built fix is committed and finalized through the P-011 gate; every filed task is committed
- [x] All work pushed to OneDev

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

### 2026-08-12 — round-3 portability / discover-peers sweep outcome
- **Hunter lens:** Constitutional Directive #4 (Portability) + the discover-peers
  verb. Returned 3 verified findings, all in the HOME-unset silent-relocation class
  that T-2607 unified away for the trust plane but left as CLI-crate outliers.
- **Built + shipped (load-bearing test via temp-revert, P-011-finalized, pushed):**
  - **[1] → T-2629:** `config.rs::termlink_config_dir()` did `HOME.unwrap_or("/tmp")`.
    HOME-unset leaked hub profiles + `bootstrap_from` trust anchors into
    world-writable `/tmp/.termlink` (a cross-user plant vector) AND made every
    fleet discover-peers verb read a nonexistent path → false "no hubs configured".
    Fixed by mirroring `identity_dir.rs`: pure `resolve_config_dir_from` core,
    HOME-set behavior-preserving, HOME-unset → UID-namespaced 0700 private dir +
    loud `tracing::error!`. Highest blast radius of the three.
  - **[3] → T-2630:** `infrastructure.rs` doctor `secret_cache` check did
    `HOME.unwrap_or_default()` → CWD-relative `.termlink/secrets` when HOME unset →
    false-clean "no cached secrets" pass on the auth-drift signal. Fixed by routing
    through the T-2629 hardened resolver via extracted `secret_cache_dir()`.
- **Verified but deliberately NOT built — [2] `push.rs` `const INBOX_DIR =
  "/tmp/termlink-inbox"`:** real hardcode (PL-021 volatile-/tmp class on the REMOTE
  target), but the `remote push` command is DEPRECATED and slated for retirement
  (→ `channel post`, T-1166). Patching a hardcoded path on a command being removed
  is negative value; recorded here for traceability instead of filing a task.
- **Plan impact:** none — the "verify-then-build-small / file-large" flow held. The
  three findings were a tight cluster (one root class), so no per-finding tracker
  churn beyond T-2629/T-2630.
- **Triggered:** T-2629 (built), T-2630 (built). Un-swept lenses remaining for a
  future round: Directive #1 (Antifragility), the claim-work + session-control
  verbs' portability surface, and the CLI crate's remaining `HOME.unwrap_or(...)`
  sites (substrate.rs log path was folded into [1]'s analysis — low blast radius,
  log-only).

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

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-12T06:16:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2628-reliabilityportability-charter-lens-hunt.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-b464ca72
- **Timestamp:** 2026-08-12T09:39:24Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-12T09:39:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
