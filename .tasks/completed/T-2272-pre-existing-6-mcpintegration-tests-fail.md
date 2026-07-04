---
id: T-2272
name: "Pre-existing: 6 mcp_integration tests fail on main (map vs sequence) — list_sessions/discover/topics"
description: >
  Discovered during T-2268. On clean main (verified by stashing unrelated edits), 6 tests in crates/termlink-mcp/tests/mcp_integration.rs fail with 'invalid type: map, expected a sequence' at line 97: test_list_sessions_empty/_with_session/_filtered_by_role, test_discover_by_role_and_name, test_topics_specific_session/_with_events. Likely the tool output shape changed array->object OR they need a live hub fixture absent in sandbox. Investigate env-dependence vs genuine breakage; the suite is red either way. Not caused by T-2268 (error-rendering only).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-24T08:32:00Z
last_update: 2026-07-04T23:23:34Z
date_finished: 2026-07-04T23:23:34Z
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

# T-2272: Pre-existing: 6 mcp_integration tests fail on main (map vs sequence) — list_sessions/discover/topics

## Context

Investigated 2026-07-05: NOT env-dependence — all 6 failures are stale test-side response
shapes against DELIBERATE tool-side envelope changes:
- `termlink_list_sessions` (tools.rs ~11060) + `termlink_discover` (~11495): bare array →
  `{ok, sessions: [...]}` envelope, T-1918/T-1919 (CLI `--json` parity; comments in-source).
- `termlink_topics` (~13230): `sessions` map keyed by name → `sessions` ARRAY of
  `{session, topics}` objects (total_topics/total_sessions retained; the topics tests'
  `total_topics >= 1` assert passes — only the `.as_object()` on the array panics).
Fix is test-side only; tool behavior is correct and intentional.

## Acceptance Criteria

### Agent
- [x] The 6 failing tests (`test_list_sessions_empty` / `_with_session` / `_filtered_by_role`,
      `test_discover_by_role_and_name`, `test_topics_specific_session` / `_with_events`)
      updated to the T-1918/T-1919 envelope shapes via a shared `envelope_sessions()` helper
      (list/discover) + array-membership asserts (topics). NO tool-code changes.
- [x] `cargo test -p termlink-mcp --test mcp_integration` fully green (99 passed, 0 failed, 15.07s).
- [x] RCA filled: why the suite sat red ~2 weeks (deliberate shape change shipped without
      updating the integration suite; nothing gates red tests in this crate).

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

out=$(cargo test -p termlink-mcp --test mcp_integration 2>&1); echo "$out" | grep -q "test result: ok"

## RCA

**Symptom:** 6 tests in `crates/termlink-mcp/tests/mcp_integration.rs` red on clean main
(`invalid type: map, expected a sequence` for list_sessions/discover; `Option::unwrap()`
on None for the topics tests' `sessions.as_object()`), sitting red ~2 weeks until T-2268
stumbled on them 2026-06-24.

**Root cause:** T-1918/T-1919 deliberately changed three MCP tool response shapes for CLI
`--json` parity — `termlink_list_sessions` and `termlink_discover` from bare array to
`{ok, sessions: [...]}` envelope, `termlink_topics` `sessions` from a name-keyed map to an
array of `{session, topics}` — but the integration suite's consumers of those shapes were
not updated in the same change. Test-side staleness, not tool breakage.

**Why structurally allowed:** no gate runs the termlink-mcp integration suite — the
pre-push audit is structure-only, task Verification blocks run only the crates each task
touched (T-1918/T-1919 were BVP/arc-tooling tasks whose verification never exercised this
crate), and there is no CI test job (release workflow only builds). A red suite in an
untouched crate is invisible until someone happens to run it.

**Prevention:** (1) the shared `envelope_sessions()` helper concentrates the envelope
assumption in one place — the next deliberate shape change breaks one helper with a clear
panic message instead of scattering 8 stale parse sites; (2) learning registered (this
session): a deliberate tool-response shape change must grep the integration tests for
consumers of the old shape in the same commit.

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

### 2026-06-24T08:32:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2272-pre-existing-6-mcpintegration-tests-fail.md
- **Context:** Initial task creation

### 2026-07-04T23:20:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e06ed85a
- **Timestamp:** 2026-07-04T23:24:10Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-04T23:23:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
