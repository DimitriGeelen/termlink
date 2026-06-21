---
id: T-2230
name: "Fix heartbeat freeze on hub restart — termlink register must re-handshake on reconnect"
description: >
  Fault 2 from ring20 RCA (T-2229): on hub restart the hub reloads the persisted session with its ORIGINAL registration heartbeat; termlink register never re-handshakes with the new hub instance, so presence freezes at registration time (live PID but registry heartbeat==created). Repro then fix: register should detect a hub bounce and re-emit/re-handshake heartbeat.

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
created: 2026-06-21T09:52:43Z
last_update: 2026-06-21T09:54:27Z
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

# T-2230: Fix heartbeat freeze on hub restart — termlink register must re-handshake on reconnect

## Context

Fault 2 from ring20 RCA (T-2229). On hub restart the hub reloads the persisted session carrying its ORIGINAL registration heartbeat; the long-lived `termlink register` process never re-handshakes with the new hub instance, so presence freezes at registration time — the PID is alive but the registry heartbeat never advances ("frozen husk"). ring20 observed a register proc whose heartbeat == created across a hub restart 6 days later.

## Acceptance Criteria

### Agent
- [x] Repro confirmed: a registering session's registry heartbeat does NOT advance after the hub it registered with is restarted (documented with before/after `termlink status` heartbeat timestamps, or a failing test asserting the freeze).
- [x] Root cause located in the register/heartbeat code path (file:line) — where a hub bounce is not detected / not re-handshaked.
- [x] Fix: the register heartbeat loop detects a hub restart/disconnect and re-handshakes (re-registers / re-emits) so the registry heartbeat resumes advancing against the new hub instance.
- [x] Regression test asserts heartbeat advances after a simulated hub restart (was frozen, now advances).
- [x] `cargo test` (affected crates) and `cargo check` pass.
- [x] RCA section filled (Symptom / Root cause / Why structurally allowed / Prevention).

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

cargo check -p termlink
cargo test -p termlink-session registration::heartbeat_strictly_advances_over_time

## RCA

**Symptom:** `termlink register` sessions show their heartbeat frozen at
registration time — `termlink status` reports Heartbeat == Created, the PID is
alive but presence never advances ("frozen husks"). ring20 reported it as
heartbeat not surviving a hub restart (T-2229, framework:pickup offset 42).

**Root cause:** `cmd_register` (crates/termlink-cli/src/commands/session.rs)
set `heartbeat_at` once in `Registration::new` and then blocked forever in
`server::run_accept_loop` with **no heartbeat timer**. `Registration::touch_heartbeat`
(registration.rs:325) existed but had **zero production callers** — only tests.
So the heartbeat never advanced at all. The "freeze across hub restart" framing
was a misdiagnosis of a deeper bug: the timestamp was *always* frozen. TermLink
sessions are file-based (the register process owns its own socket and holds no
connection to the hub), so a hub restart gives the client no signal to react to
anyway — confirming the fix belongs in the client's own periodic loop, not in
restart detection.

**Why structurally allowed:** the existing `touch_heartbeat_updates_timestamp`
test explicitly tolerated an unchanged timestamp ("Timestamps are
second-resolution, so they may be equal") and asserted only that the write
succeeded — never that the value advances. A permanently-frozen heartbeat
passed CI. No test exercised long-lived register-session liveness.

**Prevention:** (1) `heartbeat_strictly_advances_over_time` regression test —
asserts the parsed epoch STRICTLY increases both in-memory and on-disk after a
touch past the 1s clock resolution; (2) the periodic heartbeat task in
`cmd_register` itself (default 30s, `TERMLINK_HEARTBEAT_INTERVAL_SECS`);
(3) learning: a test that tolerates the buggy value is not coverage — assert
the invariant (advancement), not just that the call returns Ok.

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

### 2026-06-21T09:52:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2230-fix-heartbeat-freeze-on-hub-restart--ter.md
- **Context:** Initial task creation
