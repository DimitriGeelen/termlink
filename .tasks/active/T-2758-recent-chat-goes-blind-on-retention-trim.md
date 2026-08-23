---
id: T-2758
name: "recent-chat goes blind on retention-trimmed topics — count used as offset in
  seek-to-tail"
description: >
  agent-chat-arc-recent.sh derives its seek-to-tail cursor as chat_count - SCAN_LIMIT,
  treating channel info's count as an offset. On a retention-trimmed topic count is
  capped at the retention limit while live offsets keep rising, so the scan lands
  thousands of offsets below the retained range and the time filter discards every
  envelope — /recent-chat and /pulse report 0 posts on a topic that is actively in
  use. Third recurrence of PL-293.

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
created: 2026-08-16T09:29:42Z
last_update: '2026-08-23T19:13:48Z'
date_finished:
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
  - ts: '2026-08-23T19:13:29Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-23T19:13:48Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2758: recent-chat goes blind on retention-trimmed topics — count used as offset in seek-to-tail

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The seek-to-tail cursor is derived from an OFFSET, not from `count`:
      prefer the hub's authoritative `latest_offset` (T-2533+); else fall back
      to the largest `receipts[].up_to` in the same `channel info` payload when
      that exceeds `count - 1` (proving the topic has been trimmed); else keep
      the existing `count`-derived value, which is correct on untrimmed topics
- [x] The derivation is a pure, unit-tested helper — not inline arithmetic —
      covering: authoritative `latest_offset` present, trimmed topic with only
      receipts available, untrimmed topic (behaviour unchanged), and a topic
      with neither signal (no crash, no fabricated offset)
- [x] Regression fixture reproduces the measured defect: a topic whose `count`
      (2003) is far below its true max offset (11973) must yield a cursor
      inside the retained range, and the pre-fix arithmetic must fail it
- [x] `recent-dm.sh` is checked for the same `count`-as-offset arithmetic and
      fixed too if present — the T-2757 lesson (PL-293) is that the count-anchored
      helper always has more than one caller
- [x] The fix cannot silently UNDER-report: when no offset signal is available
      the scan must not claim an empty window it did not actually cover
- [x] Live proof on the hub that produced the numbers: `/recent-chat` surfaces
      the 2026-08-16 post it currently reports as "0 posts"
- [x] `bash tests/chat-arc-recent-fixtures.sh` passes (existing suite, extended
      with the regression fixture), and `bash scripts/run-guard-layer.sh` passes

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

# The four tail-signal branches plus the measured production defect. Asserted
# by NAME, so adding a fixture later does not break verification while a
# deleted or renamed one still does.
out=$(bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "ALL PASS"
out=$(bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "recent posts surface"
out=$(bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "is offset-derived"
out=$(bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "drain continued past a full batch"
out=$(bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "latest_offset wins over count and receipts"
out=$(bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "untrimmed topic keeps the count path"
out=$(bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "no offset signal on a trimmed topic"

# The fixtures must be LOAD-BEARING: against the pre-fix script they must FAIL.
# A regression fixture that passes both ways proves nothing. `! grep` rather
# than a non-zero exit check, because the suite exits 1 for any failure.
# Pinned to the last commit BEFORE the fix (12b1d3a69), not HEAD~N — a relative
# ref drifts with every later commit and would silently start comparing the
# fixtures against a post-fix script, which is the failure mode this check is
# supposed to rule out.
git show 12b1d3a69:scripts/agent-chat-arc-recent.sh > /tmp/.t2758-prefix.sh
out=$(CHAT_ARC_SCRIPT=/tmp/.t2758-prefix.sh bash tests/chat-arc-recent-fixtures.sh 2>&1); echo "$out" | grep -q "FAILURES"

# The cursor must be derived through the helper, not inline arithmetic — the
# T-2699 shape (a helper that exists but nothing calls).
grep -q "derive_tail_offset(" scripts/agent-chat-arc-recent.sh
grep -q 'tail_info="\$(derive_tail_offset' scripts/agent-chat-arc-recent.sh

# The sibling caller must be migrated too (PL-293 / the T-2758 learning).
grep -q "T-2758" scripts/fleet-adoption-snapshot.sh
grep -q "latest_offset" scripts/fleet-adoption-snapshot.sh

# --since must NOT be passed to subscribe: it is a render-side filter and would
# destroy "batch was full" as the end-of-topic signal for the drain.
grep -q 'drain_cursor" --limit' scripts/agent-chat-arc-recent.sh
! grep -q 'drain_cursor" --since' scripts/agent-chat-arc-recent.sh

# An incomplete read must not certify itself complete.
grep -q "tail_unknown_hubs" scripts/agent-chat-arc-recent.sh
grep -q "degraded_hubs | length) == 0" scripts/agent-chat-arc-recent.sh

# Both scripts parse.
bash -n scripts/agent-chat-arc-recent.sh
bash -n scripts/fleet-adoption-snapshot.sh

bash scripts/run-guard-layer.sh

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

### 2026-08-16T09:29:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2758-recent-chat-goes-blind-on-retention-trim.md
- **Context:** Initial task creation

### 2026-08-16T09:32:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
