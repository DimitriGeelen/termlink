---
id: T-2197
name: "Audit D13 — 4 inception limbo tasks awaiting go/no-go (T-1635, T-1898, T-2025, T-2028)"
description: >
  Audit D13 WARN: 4 inception tasks in limbo. T-1635 A-class (1 human unchecked); T-1898/T-2025/T-2028 B-class (no decision recorded yet). Substrate-arc-aligned: T-2025 + T-2028 are §6 primitives whose GO/NO-GO is blocking new substrate work per T-2144 conclusion. Operator authority required for resolution.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-12T10:20:56Z
last_update: 2026-06-12T12:08:45Z
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

# T-2197: Audit D13 — 4 inception limbo tasks awaiting go/no-go (T-1635, T-1898, T-2025, T-2028)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] T-1635 RCA: read task, identify the 1 unchecked Human AC, refresh evidence if RUBBER-STAMP; if REVIEW, surface for human decision in this task's Human ACs. **Done.** T-1635 Recommendation = GO (with option-i refinement); Decision section already records "**Decision**: GO". Agent ACs all ticked. The single unchecked Human AC is a [REVIEW] of `docs/reports/v2-peer-consult-seam-response.md` against the AEF proposal — pure human-judgment seam-review. Surfaced via `fw task review T-1635`. Already done; just needs human REVIEW click
- [x] T-1898 RCA: identify whether the inception has produced enough information for a go/no-go OR whether more exploration is required. **Recommendation = DEFER (revisit_at: 2026-07-06).** Rationale (per T-1898 file): zero of 8 spikes (S1-S8) executed because operator paused before spike-budget authorization. Decision recorded = DEFER. Two ways back to actionable: explicit spike-budget authorization OR triggering event (e.g. ring20-management >24h silence recurrence within 7d). G-053 daily revisit-due cron will surface on 2026-07-06. Surfaced via `fw task review T-1898`
- [x] T-2025 RCA: same as T-1898. Note T-2025 is substrate primitive #4 (filesystem-write observation) — substrate-arc directly blocks if deferred. **Recommendation = NO-GO; re-scope as documentation-only.** Investigation revealed the captured framing didn't match reality: `agent-presence` IS already durable (retention=forever SQLite-backed), only the derived LIVE/STALE/OFFLINE view is in-memory, reconstructible within one heartbeat (~30s). Building T-2025 as captured would create two-sources-of-truth anti-pattern. Action on NO-GO is doc-only (ADR §6 #7 wording + substrate-claim-primitive post-restart blackout paragraph). Substrate-arc IS NOT blocked — primitive #4 was a phantom requirement. Surfaced via `fw task review T-2025`. **Correction: T-2025 is substrate primitive #7 (persistent presence + circuit-breaker), not #4 (filesystem-write observation, which is T-2022). T-2197's original description had this miscategorized**
- [x] T-2028 RCA: same as T-1898. Note T-2028 is substrate primitive #10 throughput/connection budget per T-2144 (already shipped) — likely closable as superseded. **Recommendation = PARTIAL GO with 3 tracks (A: retention audit + Retention::Latest, B: connection cap + per-sender rate limit, C: budget observability).** ALL THREE TRACKS SHIPPED per CLAUDE.md catalog: Track A (T-2142 Retention::Latest), Track B + C (T-2048..T-2070 hub.governor_status RPC + CLI + fleet wrapping + watch + notify + log + history MCP). Closeable as "GO retroactively — all three tracks shipped." Already surfaced for human close via `fw task review T-2028` during T-2201 work, see commit `dc6df225`
- [x] Per-task next-step proposed in Updates section of each (decide GO / decide NO-GO with evidence / DEFER with revisit_at). **Done by Recommendation section in each task file** (all 4 have populated Recommendation blocks). Per-task /inception/<id> Watchtower URL surfaced via the 4 `fw task review` calls. Decision routes:
  - **T-1635** → GO (Watchtower or `fw inception decide T-1635 go --rationale "..."`)
  - **T-1898** → DEFER (already recorded; needs human confirmation to close as DEFER, or operator wants to bump revisit_at)
  - **T-2025** → NO-GO (closure-ready; doc-only follow-ups per Recommendation)
  - **T-2028** → GO retroactively (all three tracks shipped; closure-as-superseded equivalent)

### Human
- [ ] [REVIEW] Make GO/NO-GO/DEFER decision on each of the 4 inceptions per agent recommendations. **Steps:** read each task's Recommendation; run `fw inception decide T-XXX go|no-go|defer --rationale "..."`. **Expected:** all 4 transition out of limbo. **If not:** defer with explicit revisit_at date so G-053 daily scan picks them up later

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

### 2026-06-12T10:20:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2197-audit-d13--4-inception-limbo-tasks-await.md
- **Context:** Initial task creation

### 2026-06-12T12:06:59Z — status-update [task-update-agent]
- **Change:** status: started-work → started-work
