---
id: T-2513
name: "Fix split-read alt-screen detection in PtySession::scan_alternate_screen"
description: >
  Fix split-read alt-screen detection in PtySession::scan_alternate_screen

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-session/src/pty.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-03T16:53:34Z
last_update: 2026-08-03T17:07:32Z
date_finished: 2026-08-03T17:07:32Z
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

# T-2513: Fix split-read alt-screen detection in PtySession::scan_alternate_screen

## Context

`PtySession::scan_alternate_screen` (crates/termlink-session/src/pty.rs) scans
each PTY read in isolation with `chunk.windows(8)`. The alt-screen enter/leave
escape sequences (`\x1b[?1049h` / `\x1b[?1049l`) are 8 bytes, so a sequence
delivered in a sub-8-byte read (common — interactive programs often flush the
mode-switch in a tiny write) OR split across a 4096-byte read boundary is
**never detected** → `alternate_screen` state (and thus `terminal_mode()` /
`poll_mode_change()`) reports the wrong value. That boolean is LOAD-BEARING: it
reaches the `termlink_pty_mode` MCP tool, the `pty.mode` / status RPCs, the
`pty.mode-change` event bus, and local+remote CLI status output. Charter verb:
control terminal sessions. Fix: a rolling per-session carry of the trailing
`seq_len-1 = 7` bytes; scan `[carry || chunk]` each call.

## Acceptance Criteria

### Agent
- [x] `scan_alternate_screen` scans across read boundaries via a rolling carry buffer (`scan_carry` field), not `chunk.windows(8)` in isolation
- [x] A regression test drives detection with the sequence SPLIT across two calls (`\x1b[?104` then `9h`) and asserts the state flips — and FAILS against the old per-chunk logic (proven load-bearing by temp-revert)
- [x] A regression test feeds the sequence one byte at a time and asserts the state flips
- [x] The existing `alternate_screen_detection` (whole-sequence-in-one-call) test still passes (no double-count / no regression)
- [x] `cargo test -p termlink-session --lib pty` passes; `cargo build --release -p termlink` succeeds

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
cargo test -p termlink-session --lib pty 2>&1 | tee /tmp/t2513-test.log; grep -q "test result: ok" /tmp/t2513-test.log
cargo build --release -p termlink 2>&1 | tail -3

## RCA

**Symptom:** `terminal_mode().alternate_screen` (surfaced via `termlink_pty_mode`
MCP, `pty.mode`/status RPC, `pty.mode-change` event, CLI status) can report the
wrong alt-screen state for a session — e.g. a TUI (vim/less/htop) is in the
alternate screen but the mode reads `alternate_screen: false`.

**Root cause:** `scan_alternate_screen` was stateless — it scanned each PTY read
with `chunk.windows(8)` and kept no bytes between calls. An 8-byte enter/leave
sequence that arrived in a read of fewer than 8 bytes (`windows(8)` on a <8-byte
slice yields an EMPTY iterator) or straddled a 4096-byte read boundary matched no
window and was silently dropped. Since nothing re-frames the PTY byte stream, the
miss was unrecoverable.

**Why structurally allowed:** the only test (`alternate_screen_detection`) always
fed the complete 8-byte sequence in a single `scan_alternate_screen` call, so it
passed regardless of the boundary bug. No test exercised a split/streamed feed —
the exact shape a real PTY read loop produces.

**Prevention:** two regression tests that feed the sequence split across calls and
one-byte-at-a-time; both FAIL against the old per-chunk `windows(8)` logic
(proven by temp-revert). Captured as PL-298 (scan-across-a-streamed-byte-boundary:
any detector run per-read needs a carry of `seq_len-1` bytes, else sub-window and
boundary-split inputs are silently missed).

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

### 2026-08-03T16:53:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2513-fix-split-read-alt-screen-detection-in-p.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-cda3bcee
- **Timestamp:** 2026-08-03T17:17:57Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-03T17:07:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
