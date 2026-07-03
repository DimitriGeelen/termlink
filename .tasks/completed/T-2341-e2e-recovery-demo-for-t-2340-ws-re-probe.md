---
id: T-2341
name: "E2E recovery demo for T-2340 WS re-probe + env-tunable cadence knob"
description: >
  E2E recovery demo for T-2340 WS re-probe + env-tunable cadence knob

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/channel.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-03T23:35:53Z
last_update: 2026-07-03T23:56:49Z
date_finished: 2026-07-03T23:56:49Z
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

# T-2341: E2E recovery demo for T-2340 WS re-probe + env-tunable cadence knob

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

T-2340 shipped the WS re-probe from the steady poll floor so a raw `channel subscribe --push`
consumer recovers sub-second push after a hard hub-down without a process restart. Its evidence
is unit-tested gating + a happy-path smoke, but there is no END-TO-END demo proving the actual
recovery (every other arc-004 slice got `demo_evidence`). This task (a) makes the re-probe
cadence env-tunable (`TERMLINK_WS_REPROBE_POLL_CYCLES`) — mirrors the codebase's
`TERMLINK_WEBHOOK_RETRY_INTERVAL_MS` clamp pattern; gives operators a real knob AND makes the
demo deterministic instead of a flaky ~50s timing scrape — and (b) adds a self-contained
recovery demo (isolated hub, never touches shared :9100) that proves post→push works, a hard
hub-down degrades to poll, and after restart the re-probe restores live push WITHOUT restarting
the consumer.

## Acceptance Criteria

### Agent
- [x] The re-probe cadence is env-tunable via `TERMLINK_WS_REPROBE_POLL_CYCLES` (default 30,
      clamped to a sane range), read once per subscribe; parsing/clamp is a pure helper unit-tested
      for default-on-absent, clamp-low, clamp-high. Existing `should_ws_reprobe` behavior preserved.
      → `clamp_reprobe_cycles` (default/clamp 1..=3600) + `ws_reprobe_poll_cycles` (env read),
      `should_ws_reprobe` now `(cycles, threshold)`; 2 new `clamp_reprobe_cycles_*` tests + the 3
      `ws_reprobe_*` tests updated. Cadence read once into `reprobe_threshold` before the poll loop.
- [x] **(demo-caught defect)** In a long-lived consumer (`--follow` or the inherently-live
      `--push`) the poll loop SURVIVES a down hub: neither a failed poll RPC (`Connection refused`)
      nor a transient hub-level error (`-32013: unknown topic` after a restart) `?`-exits the
      consumer anymore — it logs, advances the cadence, sleeps, and continues so the re-probe
      recovers push. A true single-shot (`!follow && !push`) still errors. `--push` no longer
      single-shots the poll floor; the re-probe moved to the loop top + emits an observable line.
      → Both `?`-exit sites converted to `Err(e) if follow || push => { … continue }`.
- [x] `scripts/demo-ws-reprobe-recovery.sh` is a self-contained reproducer against an ISOLATED
      hub (temp `TERMLINK_RUNTIME_DIR` + temp `HOME`, never the shared hub) that: (1) shows
      post→push delivery, (2) hard-stops the hub and observes degrade-to-poll, (3) restarts the
      hub (same runtime_dir → no secret/cert rotation) and observes the re-probe restore live push
      to the SAME consumer process, (4) exits non-zero if recovery is not observed.
      → PASS (exit 0): baseline push ✓, `WS reconnect cap (6) reached — degrading to poll` ✓,
      `re-probing WS from poll floor` ✓, post-restart DM delivered to same consumer ✓.
- [x] `docs/reports/T-2341-arc-004-ws-reprobe-recovery-demo.md` records the demo run with the
      observed evidence (the degrade-to-poll line, the re-probe reconnect line, post-recovery push).
      → Written, includes the full PASS transcript + the two-defect RCA.
- [x] `cargo build -p termlink` succeeds; targeted `cargo test -p termlink --bin termlink
      commands::channel` passes; FULL crate suite `cargo test -p termlink --bin termlink` passes
      (PL-238 — WS transport path touched).
      → build clean; FULL suite **958 passed / 0 failed / 0 filtered**.

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

cargo build -p termlink 2>&1 | tail -1
out=$(cargo test -p termlink --bin termlink commands::channel 2>&1); echo "$out" | grep -q "test result: ok"
test -f scripts/demo-ws-reprobe-recovery.sh
test -x scripts/demo-ws-reprobe-recovery.sh
test -f docs/reports/T-2341-arc-004-ws-reprobe-recovery-demo.md
grep -q "TERMLINK_WS_REPROBE_POLL_CYCLES" crates/termlink-cli/src/commands/channel.rs

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

### 2026-07-04 — the demo caught a defect in shipped T-2340; fix-forward here vs. new bug task
- **Chose:** Fix the two poll-loop `?`-exit defects (RPC-level + hub-level) inside T-2341 and
  let the demo prove the *complete* recovery, rather than filing a separate T-2342 bug task.
- **Why:** The defects and the demo are not independent — the demo's entire purpose is to
  validate the hard-down recovery, and it found the recovery incomplete. The poll-floor
  resilience IS what makes the T-2340 re-probe reachable; splitting them would leave T-2341's
  own deliverable (a passing recovery demo) blocked on a sibling. Causality is preserved via the
  RCA in `docs/reports/T-2341-*.md` and this note. One coherent unit: "make hard-down recovery
  real and prove it."
- **Rejected:** (a) File T-2342 for the resilience fix — creates an awkward T-2341→T-2342
  dependency for what is one mechanism. (b) Add `--follow` to the demo to dodge the `!follow`
  exit — would MASK that bare `--push` (the actual user invocation) still exits on hard-down.

### 2026-07-04 — re-probe cadence env-tunable (not a fixed const)
- **Chose:** `TERMLINK_WS_REPROBE_POLL_CYCLES` (default 30, clamp 1..=3600), mirroring
  `TERMLINK_WEBHOOK_RETRY_INTERVAL_MS`.
- **Why:** Makes the recovery demo deterministic (cadence=2 → ~2s re-probe instead of ~30s) AND
  gives operators a real tuning knob (fast recovery vs. re-probe churn on a busy hub).
- **Rejected:** Hardcoded const — would force a flaky ~50s timing-scrape demo and give operators
  no control.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-03T23:35:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2341-e2e-recovery-demo-for-t-2340-ws-re-probe.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-7c7fbee3
- **Timestamp:** 2026-07-03T23:57:59Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-03T23:56:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
