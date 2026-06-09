---
id: T-2110
name: "hub.governor_status: expose cv_index telemetry (substrate primitives 9 + 10 cross-reference)"
description: >
  hub.governor_status: expose cv_index telemetry (substrate primitives 9 + 10 cross-reference)

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
created: 2026-06-09T22:36:49Z
last_update: 2026-06-09T22:36:49Z
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

# T-2110: hub.governor_status: expose cv_index telemetry (substrate primitives 9 + 10 cross-reference)

## Context

Cross-reference observability gap between substrate primitive #10 (BACKPRESSURE
— `hub.governor_status`, T-2048) and substrate primitive #9 (BROADCAST-WITH-REPLAY
— cv_index, T-2089/T-2103..T-2109).

**Today.** `hub.governor_status` exposes connection/rate/dedupe counters
but says nothing about cv_index health. Operators cannot answer "is the
cv_index growing? Has any topic saturated the per-topic cap?" without
shelling out to a hub-internal counter print. `docs/operations/substrate-broadcast-with-replay.md`
explicitly calls this gap out under "Related primitives → #10": *"cv-related
counters could be added in a future expansion."*

**Why it matters.** cv_index is the substrate's late-joiner-discovery store
(T-2103) and the source for the new T-2109 find_idle fast path. If a
producer mis-emits cv_key (e.g. uses a timestamp instead of a stable id),
the per-topic cap (default 1000) silently throttles new entries. The
existing `overflow_total` AtomicU64 counts every refusal, but nothing
surfaces it. A fleet-wide watch loop (`fleet governor-status --watch`)
should fire on cv_index saturation the same way it already fires on
capacity hits + rate hits.

**Approach.** Pure additive expansion of `hub.governor_status` response:
add `cv_index_entries_active`, `cv_index_topics_active`,
`cv_index_overflow_total`, `cv_index_cap_per_topic`. CLI and MCP render
them alongside existing counters. Fleet aggregator (`fleet governor-status`)
includes them in per-hub blocks but does NOT add them to the
"pressured" predicate yet — that needs the operator pressure-threshold
question answered first (deferred).

**Out of scope.** cv_index persistence across restarts (deliberate
deferral). Per-topic cv_index counters in the response (the API returns
hub-wide totals; per-topic inspection lives in `channel cv-keys`).
Modifying the `--only-pressured` predicate to fire on
`cv_index_overflow_total > 0` (needs operator-tuned pressure semantics).

## Acceptance Criteria

### Agent
- [x] Module-level wrappers added to `crates/termlink-hub/src/cv_index.rs` for `topics_active()` and `cap_per_topic()` (mirror the existing `entries_active()` / `overflow_total()` wrappers) so `router.rs` can read them without owning a `CvIndex` reference.
- [x] `handle_hub_governor_status` (crates/termlink-hub/src/router.rs:838) response includes four new fields: `cv_index_entries_active: u64`, `cv_index_topics_active: u64`, `cv_index_overflow_total: u64`, `cv_index_cap_per_topic: u64`. Pure additive — existing fields unchanged.
- [x] `hub.governor_status` doc-comment updated to list the four new fields.
- [x] CLI rendering of `termlink hub status --governor` shows the cv_index counters inline alongside dedupe counters (consistent T-2060 pattern). Pure text render — no new flags.
- [x] CLI rendering of `termlink fleet governor-status` (per-hub block + fleet rollup `total_cv_index_entries_active` / `total_cv_index_overflow_total`) includes the cv_index counters.
- [x] MCP shape: `termlink_hub_governor_status` returns the four new fields too (pure passthrough — no shape divergence vs the RPC). `termlink_fleet_governor_status` extends the `summary` envelope with `total_cv_index_entries_active` + `total_cv_index_overflow_total` for parity with the CLI fleet rollup.
- [x] Hub unit test: `handle_hub_governor_status` returns all 13 expected fields (9 prior + 4 new). One assertion per new field name.
- [x] Hub regression: full `cargo test -p termlink-hub` passes (352 tests = 351 prior + 1 new, no regression).
- [x] Live sidecar smoke: start a sidecar hub, post a few cv-tagged messages, call `hub.governor_status` and verify `cv_index_entries_active > 0`. Then exceed the cap (override `TERMLINK_CV_INDEX_CAP_PER_TOPIC=2` and post 3 distinct cv_keys), verify `cv_index_overflow_total > 0`. PASS via /tmp/T-2110-smoke.sh + /tmp/T-2110-json-smoke.sh — both text + JSON envelopes show {entries=2, topics=1, overflow=1, cap=2} after over-cap post.
- [x] `docs/operations/substrate-broadcast-with-replay.md` "Related primitives → #10" updated — note that cv-related counters are now surfaced.
- [x] CLAUDE.md Quick Reference row for `hub.governor_status` updated with one-line callout of the four new cv_index fields.

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

out=$(cargo build -p termlink-hub -p termlink -p termlink-mcp 2>&1); echo "$out" | grep -q "Finished\|warning"
out=$(cargo test -p termlink-hub governor_status_exposes_cv_index_counters 2>&1); echo "$out" | grep -q "1 passed"
out=$(cargo test -p termlink --bin termlink render_governor_section 2>&1); echo "$out" | grep -q "2 passed"
test -f docs/operations/substrate-broadcast-with-replay.md
grep -q "cv_index" CLAUDE.md

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

### 2026-06-09T22:36:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2110-hubgovernorstatus-expose-cvindex-telemet.md
- **Context:** Initial task creation
