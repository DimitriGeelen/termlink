---
id: T-2473
name: "Fix OBS-108 bug #1 file receive --replay hardwired to oldest transfer — serve newest + nudge channel subscribe"
description: >
  OBS-108 bug #1 (separate from T-2472 bug #2): file receive --replay subscribes at cursor 0 then process_artifact_batch returns artifacts.first() = oldest, so replay is permanently stuck on the earliest transfer and later ones are unreachable. Minimal fix (retiring verb per T-1166): serve NEWEST artifact on replay instead of oldest + deprecation nudge to channel subscribe for full addressable history. Not building a rich transfer-id model into a dying primitive.

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
created: 2026-07-31T11:52:07Z
last_update: 2026-07-31T11:52:07Z
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

# T-2473: Fix OBS-108 bug #1 file receive --replay hardwired to oldest transfer — serve newest + nudge channel subscribe

## Context

OBS-108 bug #1 (bug #2 fixed in T-2472). `file receive --replay` subscribes at
cursor 0 (file.rs:432) then `process_artifact_batch` returns `artifacts.first()`
(file.rs:484) = lowest offset = oldest artifact, with no cursor/selector — so
replay is permanently pinned to the earliest transfer and newer ones are
unreachable. **Minimal fix** on a retirement-track verb (T-1166): serve the
NEWEST artifact on replay (`.last()` not `.first()`) and nudge to
`channel subscribe` for full addressable history. NOT building a transfer-id
addressing model into a dying primitive.

## Acceptance Criteria

### Agent
- [x] `process_artifact_batch` serves the NEWEST artifact in the batch (highest
      `channel_offset`), not the oldest — `--replay` no longer pins to the earliest
      transfer.
- [x] A unit test proves newest-wins selection over a multi-artifact batch.
- [x] `cargo check -p termlink` clean; existing file tests still pass.

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
cargo check -p termlink 2>&1 | tail -1
cargo test -p termlink --bin termlink select_newest 2>&1 | grep -q '2 passed'
grep -q 'select_newest_artifact' crates/termlink-cli/src/commands/file.rs

## RCA

**Symptom:** `termlink file receive --replay` returned the same months-old transfer
(`escalation-patterns.yaml`) on all four invocations; newer transfers were
permanently unreachable through the verb.

**Root cause:** the replay path subscribes at channel cursor 0 (oldest-first;
file.rs:432) and `process_artifact_batch` returned `artifacts.first()` (file.rs:484)
= the lowest offset = the oldest artifact. With no persisted cursor and no
selector, replay was hardwired to the earliest envelope forever.

**Why structurally allowed:** the verb had exactly one selection policy
("first in the batch") that happened to coincide with "oldest" once the
subscribe-at-0 replay path was added; nothing expressed "which transfer did the
caller actually want?", and OBS-108 bug #2's false-green (fixed in T-2472) meant
the wrong selection produced a clean success, hiding it.

**Prevention:** select the NEWEST artifact (`select_newest_artifact`, max
`channel_offset`) so replay serves the most-recent transfer; a unit test locks
newest-wins with an explicit regression guard against the old `.first()` behaviour.
Full addressable history remains available via `channel subscribe` — the
retirement-track successor (T-1166); no transfer-id model is added to the dying verb.

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

### 2026-07-31T11:52:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2473-fix-obs-108-bug-1-file-receive---replay-.md
- **Context:** Initial task creation
