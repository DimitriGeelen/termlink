---
id: T-2458
name: "find-idle hint path under-reports live idle agents once cv_index topic saturates its per-topic cap — fall back to authoritative walk when saturated (round-14 F2)"
description: >
  find-idle hint path under-reports live idle agents once cv_index topic saturates its per-topic cap — fall back to authoritative walk when saturated (round-14 F2)

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
created: 2026-07-22T18:27:52Z
last_update: 2026-07-22T18:27:52Z
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

# T-2458: find-idle hint path under-reports live idle agents once cv_index topic saturates its per-topic cap — fall back to authoritative walk when saturated (round-14 F2)

## Context

Round-14 adversarial correctness hunt (F2). `agent.find_idle` (substrate #2)
takes the O(N) cv_index hint path whenever `current_values("agent-presence")`
is non-empty (`channel.rs:2099`), else walks the presence log. But cv_index caps
distinct keys per topic at 1000 (`cv_index.rs:61,207`) and NEVER evicts a key
(`record()` only inserts / monotonic-max-updates; only `remove_topic` removes).
Once the topic saturates, a newly-spawned LIVE idle worker's heartbeat overflows
silently (`cv_index.rs:207-213`) and is ABSENT from the hint's `cv_entries`, so
`find_idle_agents_from_hint` cannot see it — while `find_idle_agents` (the walk,
ground truth) would. Two paths, same state, DIFFERENT idle sets — the exact
disagreement the substrate #4 correctness model forbids. Orchestrator under-counts
idle workers → stalls / over-serializes dispatch (lost coordination, HIGH-when-hit).
The walk is only reached when the index is EMPTY, so this degraded state is not
self-healing. Fix: use the authoritative walk when the topic's cv_index is at/over
cap; keep the fast hint below cap. Touches only the fallback predicate — cv_index's
monotonic-insert-only semantics (which T-2457 depends on) are unchanged.

## Acceptance Criteria

### Agent
- [x] `handle_agent_find_idle` (channel.rs) uses the walk path (`find_idle_agents`) when `current_values("agent-presence").len() >= cv_index::cap_per_topic()`, and the hint path only when the topic is strictly below cap (and non-empty). — via pure predicate `find_idle_hint_is_complete(entry_count, cap)` (channel.rs), unit-tested at the boundary.
- [x] A regression test proves that when the topic's cv_index is saturated (at cap) but a live idle agent overflowed (is absent from cv_entries), find-idle still returns that agent — i.e. the saturated case resolves via the walk, not the lossy hint. — `find_idle_walk_finds_overflow_agent_that_hint_misses` (termlink-bus): walk sees the overflow agent, incomplete hint does not.
- [x] Below-cap behavior is unchanged (hint path still used); empty-index behavior unchanged (walk). Full `cargo test -p termlink-hub --lib` stays green. — 436 hub (+2) / 87 bus (+1) tests green; 17 find_idle tests pass.

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

cargo test -p termlink-hub --lib find_idle 2>&1 | tail -5 | grep -qE 'test result: ok'
cargo test -p termlink-hub --lib 2>&1 | tail -5 | grep -qE 'test result: ok'

## RCA

**Symptom:** `agent.find_idle` returns fewer live idle agents than actually exist
(up to and including reporting zero) once the `agent-presence` cv_index reaches its
per-topic key cap. A freshly-spawned, LIVE, idle worker is invisible to dispatch.

**Root cause:** The hint-vs-walk selector (`channel.rs:2099`) is a binary
`!cv_entries.is_empty()`. It treats a NON-EMPTY cv_index as COMPLETE. But cv_index
is monotonic-insert-only with a hard per-topic cap of 1000 and no eviction
(`cv_index.rs:207-217`); once saturated, further distinct keys overflow silently.
So "non-empty" ≠ "complete": at cap the hint's `cv_entries` is a lossy subset
missing every advertiser that arrived after saturation. The walk path
(`find_idle_agents`) is the ground truth but is only reachable on an EMPTY index.

**Why structurally allowed:** The T-2109 fast-path optimization added the hint
path as a drop-in for the walk assuming cv_index parity, but the per-topic cap
(T-2089) and no-evict policy (a deliberate design choice cv_index shares with the
T-2457 monotonic invariant) mean the hint diverges from the walk exactly at the
boundary the optimization never tested — a saturated-but-non-empty index. No test
exercised find-idle at cap, so the hint/walk disagreement went undetected.

**Prevention:** (1) the regression test added here pins find-idle correctness at
cap (saturated index → walk, not lossy hint); (2) the fix makes saturation, not
just emptiness, trigger the authoritative walk, so the disagreement window is
closed rather than merely documented. Learning candidate: "a fast-path that
mirrors a slow-path is only safe where the two are provably equivalent — a bounded
cache diverges from an unbounded ground-truth at the bound."

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

### 2026-07-22T18:27:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2458-find-idle-hint-path-under-reports-live-i.md
- **Context:** Initial task creation
