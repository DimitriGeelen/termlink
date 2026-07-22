---
id: T-2446
name: "WS client read timeout — dead-hub push wedge must degrade to poll (WS#4, T-2445)"
description: >
  WS client read timeout — dead-hub push wedge must degrade to poll (WS#4, T-2445)

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
created: 2026-07-21T22:08:32Z
last_update: 2026-07-21T22:13:06Z
date_finished: 2026-07-21T22:13:06Z
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

# T-2446: WS client read timeout — dead-hub push wedge must degrade to poll (WS#4, T-2445)

## Context

Round-9 continuation of the round-8 review backlog (T-2445 WS#4, flagged the
strongest remaining candidate). The client push consumer `run_ws_session`
(`ws_consumer.rs`) streams via `source.next().await` with no read timeout and
sends no pings. On a silently-dead / half-open hub link the consumer blocks
forever: the CLI reconnect loop (`run_ws_reconnect_loop`) never fires because
`run_ws_push` never returns, so push silently wedges with no fallback to poll —
the client-side mirror of the hub-side leak fixed in T-2442. The reconnect loop
ALREADY degrades to poll on any `Err` from the session; the only gap is that a
dead hub produces no error. Fix: bound the client read with a timeout so a
silent hub yields `Err(ReadTimeout)`, letting the existing reconnect+catch-up
path degrade to poll. A live hub (post-T-2442) pings every 30s, and each ping
frame counts as inbound activity that resets the read window, so healthy quiet
push sessions are unaffected.

## Acceptance Criteria

### Agent
- [x] `ws_consumer` bounds every stream read (handshake acks AND the push loop) with `TERMLINK_WS_CLIENT_READ_TIMEOUT_MS` (default 90000, clamped, > hub ping interval); a hub that goes silent yields `WsConsumerError::ReadTimeout` instead of hanging forever.
- [x] The timeout maps to an `Err` that propagates through `run_ws_push` to `run_ws_reconnect_loop`'s existing `Err` arm (degrade-to-poll) — no new plumbing needed; confirmed by reading the caller.
- [x] Unit test: a source that never yields returns `Err(ReadTimeout)` within the window.
- [x] Unit test: a source with a frame ready returns the frame (no false timeout).
- [x] `cargo test -p termlink-session --lib ws_consumer` green.

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

cargo test -p termlink-session --lib ws_consumer

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

**Symptom:** A CLI `channel subscribe --push` (or any `stream_ws_events`
consumer) against a hub whose TCP link half-opens (no FIN/RST — NAT drop, cable
pull, hung peer) freezes: no more events render, and — unlike a clean
disconnect — it never falls back to poll. The session appears alive but is
permanently deaf.

**Root cause:** `run_ws_session`'s stream loop and `next_text_frame` await
`source.next()` with no timeout. A half-open socket never yields another frame
and never errors, so the future is pending forever; `run_ws_push` never returns,
so `run_ws_reconnect_loop`'s `Err`/`Ended` arms (which poll + reconnect) never
run.

**Why structurally allowed:** The reconnect-and-degrade-to-poll machinery was
built to handle session *errors* (T-2442-era), but "silent hang" is the absence
of an error — the one failure mode the loop cannot observe. No test exercised a
source that neither yields nor errors. It is the exact client-side twin of the
hub-side leak T-2442 fixed (hub held a slot on a silent client; here the client
hangs on a silent hub).

**Prevention:** A unit test drives a never-yielding stream through the bounded
read and asserts `Err(ReadTimeout)` within the window; a sibling asserts a ready
frame is returned unchanged (no false timeout). Any future refactor that drops
the timeout fails the first test.

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

### 2026-07-21T22:08:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2446-ws-client-read-timeout--dead-hub-push-we.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-977b3044
- **Timestamp:** 2026-07-21T22:13:08Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-21T22:13:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
