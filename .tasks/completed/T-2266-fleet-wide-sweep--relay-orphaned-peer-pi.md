---
id: T-2266
name: "Fleet-wide sweep — relay orphaned peer pickups (ring20/025/etc) stranded on dead framework:pickup topic into AEF inbox"
description: >
  Fleet-wide sweep — relay orphaned peer pickups (ring20/025/etc) stranded on dead framework:pickup topic into AEF inbox

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-23T23:43:55Z
last_update: 2026-06-23T23:49:27Z
date_finished: 2026-06-23T23:49:27Z
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

# T-2266: Fleet-wide sweep — relay orphaned peer pickups (ring20/025/etc) stranded on dead framework:pickup topic into AEF inbox

## Context

Extends T-2265 fleet-wide. The `framework:pickup` topic has no inbound consumer
on ANY side (G-063/PL-228) — it is a write-only mirror. So peer filings posted to
it (ring20-management, 025-WorkshopDesigner, etc.) that are addressed to the
framework-agent/AEF were never delivered, same as termlink's were. This task
triages those peer filings on the .107 hub topic and couriers the genuinely
AEF-bound, confirmed-absent ones into AEF's local `inbox/` via termlink_run,
preserving original attribution (source.project stays the peer's) and annotating
that termlink couriered them due to the dead topic. Strictly: do NOT relay pings,
termlink-addressed items, already-present items, or items already subsumed by the
T-2265 relays (e.g. no-federation, already sent as P-048→T-2433).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Triaged all non-termlink filings on framework:pickup (.107) by from/to/type/content; classified each as relay / skip-duplicate / skip-not-AEF-bound / skip-ping (see Updates)
- [x] For each relay candidate, confirmed ABSENT from AEF inbox/processed before couriering (off48 unique tokens G-155/G-156/false-green = 0 files at AEF; off54 G-158/estate-version-currency = 0 files)
- [x] Couriered the confirmed AEF-bound absent filings into AEF inbox via termlink_run with original attribution preserved + courier annotation; AEF auto-created T-2439 (ring20 T-1296) + T-2440 (ring20 T-1300), both "(from ring20-management)", CLAUDECODE unset, no recommendation injected
- [x] Recorded the per-filing disposition (relayed→AEF T-NNNN, or skipped+reason) in this task's Updates

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

### 2026-06-23T23:43:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2266-fleet-wide-sweep--relay-orphaned-peer-pi.md
- **Context:** Initial task creation

### 2026-06-24 — fleet-pickup-sweep disposition (framework:pickup .107, offsets 48–57)

| offset | from | disposition |
|---|---|---|
| 48 | ring20-management → framework-agent (T-1296 systemic) | **RELAYED → AEF T-2439** (R2/R4/R5 net-new; R1/R3 noted covered by T-2433/T-2430) |
| 49,52,55,56,58,59 | termlink (P-047/048/049/050 + nudge) | already handled (T-2265: AEF T-2428/2431/2433/2435/2437) |
| 50 | ring20 | **SKIP** — `read-path-diag warm-ping` (32 bytes, not a filing) |
| 51 | ring20 → framework-agent/termlink-agent (.107 read-path stall) | **SKIP** — termlink hub bug, already tracked as termlink **T-2258** (not AEF-bound) |
| 53 | termlink-agent → ring20 | **SKIP** — ack, not a filing |
| 54 | ring20-manager (arc-004 / G-158, T-1300) | **RELAYED → AEF T-2440** (two framework-tooling proposals; confirmed absent at AEF) |
| 57 | 025-WorkshopDesigner (T-917 federation RCA) | **SKIP** — titled "PICKUP → termlink" (addressed to termlink, not AEF; same no-fed theme) |

Net: 2 genuine AEF-bound stranded peer filings couriered (ring20 only); all others
correctly skipped (pings/acks/termlink-addressed/already-handled). Attribution
preserved (AEF tasks tagged "from ring20-management"). Side-finding: AEF created a
DUPLICATE task pair T-2433+T-2434 for the single P-048 relay — AEF-side dedup
matter, not couriered by termlink; flagged for AEF grooming, not touched.

### 2026-06-23T23:49:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
