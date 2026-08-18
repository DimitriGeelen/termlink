---
id: T-2519
name: "termlink_interact panics on UTF-8 output split at snapshot boundary — raw byte
  slice not char-boundary-safe"
description: >
  termlink_interact slices full_output[pre_len..] at a raw byte offset (byte-len of
  an independently utf8-lossy-decoded pre-snapshot) with no char-boundary check; UTF-8
  output split at a PTY read boundary panics 'byte index N is not a char boundary'.
  Fix: extract output_delta() helper using str::get with fallback.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-03T22:10:50Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-03T22:17:02Z
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
  - ts: '2026-08-18T18:56:49Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2519: termlink_interact panics on UTF-8 output split at snapshot boundary — raw byte slice not char-boundary-safe

## Context

`termlink_interact` (crates/termlink-mcp/src/tools.rs) captures a session's
output by diffing two `query.output` snapshots: it records `pre_len =
pre_output.len()` (byte length of the pre-injection snapshot, tools.rs:12022)
then slices the post-injection snapshot `&full_output[pre_len..]` (tools.rs:12065)
guarded only by `full_output.len() > pre_len`. That guard checks *range*, not
*char boundary*. Both snapshots are `String::from_utf8_lossy` of independent raw
scrollback byte reads (handler.rs `query.output`), so if the pre-snapshot ended
mid-multibyte-char the lossy `U+FFFD` (3 bytes) shifts `pre_len` off any real
boundary of `full_output` — and `&full_output[pre_len..]` **panics** ("byte index
N is not a char boundary"). Any UTF-8 terminal output (emoji, box-drawing
progress bars, accented text) split at a 4096-byte PTY read boundary at the
pre-snapshot instant can trigger it, aborting the marquee "run a command in a PTY
and return its output" tool. Found by T-2468 campaign firing #38 (output-capture
lens).

## Acceptance Criteria

### Agent
- [x] The byte-slice diff is replaced with a boundary-safe read: a pure helper
      `output_delta(full: &str, pre_len: usize) -> &str` returns `full.get(pre_len..)`
      with a fallback to `full` when `pre_len` is out of range OR not a char
      boundary (so it can never panic), and the call site (tools.rs) uses it.
- [x] A load-bearing unit test (`output_delta_handles_non_char_boundary`) passes a
      `pre_len` that falls mid-multibyte-char and asserts no panic + fallback to
      `full`; plus valid-boundary, boundary-0, equal-len, past-end, and ASCII cases.
      Proven load-bearing by temp-reverting the helper body to `&full[pre_len..]` →
      test panicked ("byte index 3 is not a char boundary").
- [x] Behaviour preserved for the common case: when `pre_len` is a valid boundary
      ≤ len, `output_delta` returns exactly the suffix (marker detection unchanged).
- [x] `cargo build --release -p termlink-mcp` compiles and `cargo test -p
      termlink-mcp --lib output_delta` passes (1 passed; 0 failed).

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
grep -q "let output = output_delta(&full_output, pre_len);" crates/termlink-mcp/src/tools.rs
cargo test -p termlink-mcp --lib output_delta > /tmp/t2519-verify.txt 2>&1 && grep -q "test result: ok" /tmp/t2519-verify.txt

## RCA

**Symptom:** `termlink_interact` (run a command in a PTY, return its output) can
panic mid-call with "byte index N is not a char boundary" when the session emits
UTF-8 (emoji, box-drawing progress bars, accented text). The tool call aborts.

**Root cause:** the tool diffs two `query.output` snapshots by slicing
`&full_output[pre_len..]` where `pre_len = pre_output.len()` — a raw byte index
guarded only by `full_output.len() > pre_len` (a range check, not a char-boundary
check). Both snapshots are `String::from_utf8_lossy` of independent raw scrollback
byte reads; if the pre-snapshot's tail ended mid-multibyte-char, the lossy
`U+FFFD` (3 bytes) shifts `pre_len` off any real boundary of `full_output`, and
Rust's `str` byte-indexing panics on a non-boundary index.

**Why structurally allowed:** the two snapshots are decoded independently, so
`pre_len` is only coincidentally a valid index into `full_output` — the code
assumed lossy decode is byte-prefix-stable across a partial trailing char, which
it is not. No test drove `termlink_interact` (or the slice) with UTF-8 split at a
read boundary; the happy path (ASCII, or a clean boundary) always lands on a
boundary and hid the latent panic.

**Prevention:** the slice is extracted into a pure `output_delta(full, pre_len)`
helper that uses `str::get(pre_len..)` (returns `None` for out-of-range OR
non-boundary indices) with a fallback to `full` — it can never panic. Unit test
`output_delta_handles_non_char_boundary` drives a mid-char `pre_len` and asserts
no panic + fallback, plus boundary/past-end/ASCII cases — proven load-bearing
(reverting the helper to `&full[pre_len..]` makes it panic "byte index 3 is not a
char boundary"). Extracting the logic also makes this class testable without a
live PTY+hub.

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

### 2026-08-03T22:10:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2519-termlinkinteract-panics-on-utf-8-output-.md
- **Context:** Initial task creation

### 2026-08-03T22:11:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-85ca0083
- **Timestamp:** 2026-08-03T22:17:21Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-03T22:17:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
