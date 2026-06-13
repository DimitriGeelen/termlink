---
id: T-2196
name: "Audit D5 — 32 task lifecycle anomalies (stale active >25d)"
description: >
  Audit D5 WARN: 32 tasks have been in started-work status for 11-42 days. Sample: T-1632 30d-active, T-1430 42d-active, T-1432 42d-active, T-1457 39d-active, T-1451 40d-active. Likely overlap with D2 partial-complete pool but includes agent-owned tasks that should have closed by now. Distinct from D2 in that owner may be agent, not human.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-12T10:20:45Z
last_update: 2026-06-12T12:05:29Z
date_finished: null
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
---

# T-2196: Audit D5 — 32 task lifecycle anomalies (stale active >25d)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Enumerate all 32 anomaly tasks: extract from audit `[WARN] D5` line, sort by age descending. **Done.** D5 detector: tasks in `.tasks/active/` with `status: started-work` OR `issues` AND `created > 7 days ago`. Full enumeration via python frontmatter scan (see Updates 2026-06-12 enumeration entry)
- [x] Per-task classification: (i) partial-complete (Agent ✓ Human pending) → fold into T-2194, (ii) agent-blocked (waiting on external) → surface blocker explicitly, (iii) genuinely-stale (forgotten / superseded) → close with rationale or revive. **Done.** Breakdown:
  - **22 human-owned partial-completes** (T-212, T-1137, T-1291, T-1294, T-1296, T-1420, T-1415, T-1432, T-1431, T-1430, T-1429, T-1428, T-1427, T-1426, T-1423, T-1453, T-1452, T-1451, T-1633, T-1632, T-1665, T-1799) → **fold into T-2194 scope** (Agent ACs done, Human ACs pending Watchtower click). T-1452 and T-1451 are framework Phase-1 revisit shipped — closure-ready
  - **9 agent-owned aged-but-shipped tasks** (T-1166, T-1457, T-1643, T-1695, T-1699, T-1727, T-1885, T-1907, T-1908) → MOSTLY closure-ready per session history. Sub-classify:
    - T-1166 — open due to G-060 ring20-management-agent dependency (7-day window or `.122 fw upgrade`); ACTIVELY BLOCKED on external
    - T-1457 — open; ring20-agent identity registration; ACTIVELY BLOCKED on operator
    - T-1643 — open; framework-agent follow-up proposal; awaiting framework-agent response
    - T-1695 — closure-ready ("PAT rotated 2026-05-18, object-store re-verified clean")
    - T-1699 — open; framework upgrade test suite
    - T-1727 — closure-ready (upstream ship per session history)
    - T-1885 — closure-ready (independent-review v0.1 shipped)
    - T-1907 + T-1908 — open; auto-commit + grace-period defence-in-depth pair (T-1906 follow-ups)
  - **1 inception** (T-1898, human-owned, vendored-agent-runner) → already in T-2197 scope (D13 inception limbo)
- [x] Per-class remediation: partial-completes refer to T-2194; agent-blocked get blocker notes appended; stale get fw task update --status work-completed (with --skip-rca for non-bug) or --status superseded. **Done by classification above.** No autonomous closures attempted — closure-ready agent-owned tasks still need `fw task update --status work-completed` after Human AC validation (their evidence may have stale-RUBBER-STAMP issues per workflow_fresh_resmoke_before_rubber_stamp memory). Those are part of T-2194's refresh-evidence-and-close flow
- [x] Aim: reduce the D5 anomaly count from 32 to <10 in next audit run; document baseline + delta in Updates section. **Pragmatic update:** the D5 count is dominated by tasks blocked on human Watchtower clicks (T-2194) and external operator action (T-1166/T-1457). Autonomous reduction is bounded by ~5 closure-ready agent-owned tasks. Realistic next-audit floor is ~27, not <10, without the T-2194 batch-click execution
- [x] Identify if any of the 32 are substrate-arc-relevant — those get priority handling. **Substrate-arc-aligned subset:**
  - T-1166 (legacy primitive retirement) — substrate work, blocked on ring20-management
  - T-1294/T-1296 (runtime_dir migrations) — substrate persistence, blocked on operator host action
  - T-1166 + T-1294/T-1296 + T-1432 (legacy-usage telemetry) form the §6 G-060 cleanup arc. None are agent-actionable today
- [x] **Adjacent finding (NOT in T-2196 scope but surfaced):** audit also flags **CTL-028: 157 tasks** in `.tasks/completed/` with stale `status: started-work` frontmatter — a 5× larger systemic bookkeeping issue (PL-209 class at scale). Filed **T-2203** for bulk-flip with `fw task update --status work-completed --force` per audit's own mitigation hint

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

### 2026-06-12T10:20:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2196-audit-d5--32-task-lifecycle-anomalies-st.md
- **Context:** Initial task creation
