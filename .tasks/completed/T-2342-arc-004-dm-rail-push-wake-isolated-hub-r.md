---
id: T-2342
name: "arc-004 dm-rail push-wake isolated-hub regression demo"
description: >
  arc-004 dm-rail push-wake isolated-hub regression demo

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/demo-dm-rail-pushwake.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-04T00:11:03Z
last_update: 2026-07-04T00:21:41Z
date_finished: 2026-07-04T00:21:41Z
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

# T-2342: arc-004 dm-rail push-wake isolated-hub regression demo

## Context

The arc-004 dm-rail push-wake (T-2322 GO → S1 `T-2323` hub `dm.queued` emit + S2
`T-2324` waker dm rail) was verified live exactly ONCE (T-2325), manually, against
the shared `:9100` hub after an operator restart. It has unit tests
(`pushwaker_dedup_ok`, filter tests) but — unlike the inbox rail
(`demo-pushwaker-e2e.sh`) and the WS reconnect path (`demo-ws-reprobe-recovery.sh`)
— **no reusable isolated-hub regression reproducer**. This is the exact coverage
shape T-2341 filled for T-2340, where the E2E demo caught 2 real process-death
defects that unit tests + a happy-path smoke had missed (PL-240). This task adds
`scripts/demo-dm-rail-pushwake.sh`: an isolated fresh-binary hub proof that a
NON-live sender's `dm:` post push-wakes the receiver via the `dm.queued` rail,
with a false-wake negative and an inbox-rail no-regression check. Design ground:
`scripts/be-reachable-pushwaker.sh` (dm rail = `pushwaker_rail_loop dm.queued
<self_fp>`), `docs/reports/T-2322-arc-004-dm-rail-push-wake-inception.md`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/demo-dm-rail-pushwake.sh` exists, is executable, and is
  isolation-safe: runs entirely under a temp `TERMLINK_RUNTIME_DIR` + temp `HOME`,
  never touches the shared `:9100` hub or the operator's `~/.termlink`, and tears
  the hub down on exit (same contract as `demo-pushwaker-e2e.sh`). `bash -n` clean.
  *(chmod +x + `bash -n` clean; header documents the isolation contract; `cleanup()`
  trap stops be-reachable, kills the tmux PTY + hub, and rm's both temp dirs.)*
- [x] POSITIVE proven E2E: against the isolated fresh-binary hub, a `dm:<poster>:<self>`
  post by a NON-live sender (one-shot `channel post`, never registered) rings the
  receiver's dm rail — the waker log shows `pushwaker: rang … via dm.queued
  offset=<n>`. This exercises the load-bearing T-2323 hub `dm.queued` emit AND the
  T-2324 waker addressee==self-fp match through a real WS push, not a stub.
  *(Run 2026-07-04: `dm rail enabled: yes`; `dm.queued rings 0 -> 1`; PTY doorbell
  `marks 0 -> 2` — /check-arc landed. Poster fp `7f546799…` ≠ receiver fp
  `ad592138…`, minted via distinct TERMLINK_AGENT_ID.)*
- [x] NEGATIVE + no-regression proven: a `dm:` post addressed to a DIFFERENT
  participant does NOT ring the dm rail (no false wake), AND an `inbox:<self>`
  deposit in the SAME session still rings via `inbox.queued` (the dm rail does not
  regress the inbox rail).
  *(`negative (no wake): dm.queued rings 1 -> 1` — unchanged on dm:<poster>:<other>;
  `inbox no-regression: inbox.queued rings 0 -> 1`.)*
- [x] The demo runs green end-to-end (exit 0) on the current tree; the PASS
  transcript + any defect RCA is recorded in `docs/reports/T-2342-arc-004-dm-rail-pushwake-demo.md`.
  *(exit 0, no stderr spam after the `log_count` fix; report written with the
  transcript + the in-demo `grep -c || echo 0` bug RCA + honest "no product defect
  caught this time, value is the reusable reproducer" note.)*

## Verification

# Shell commands that MUST pass before work-completed. One per line.
test -x scripts/demo-dm-rail-pushwake.sh
bash -n scripts/demo-dm-rail-pushwake.sh
test -f docs/reports/T-2342-arc-004-dm-rail-pushwake-demo.md
#
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

This task is a regression-coverage add, not a bug fix — the RCA frames the
coverage gap it closes (title matches "regression", so the gate requires this).

**Symptom:** the arc-004 dm-rail push-wake had no automated reproducer. Its only
live evidence (T-2325) was a one-time manual run against the shared `:9100` hub;
nothing re-provable protects it from silent regression.

**Root cause:** the demo suite grew rail-by-rail — `demo-pushwaker*.sh` cover the
inbox rail, `demo-ws-*.sh` cover the WS reconnect/push path — but the dm rail
(T-2323/T-2324) shipped its verification as a manual live run gated on an operator
hub restart, so no isolated-hub demo was written for it.

**Why structurally allowed:** unit tests (`pushwaker_dedup_ok`, filter tests)
exercise the pure decide/dedup helpers but never the hub `dm.queued` emit → WS push
→ real inject chain; a happy-path manual run "proved it works" once and the loop
closed without a reusable artifact. T-2341/PL-240 already established that E2E
demos catch integration defects (`?`-exit process-death) that unit tests miss.

**Prevention:** `scripts/demo-dm-rail-pushwake.sh` — an isolated fresh-binary hub
reproducer runnable on any tree, so a future change that breaks the `dm.queued`
emit or the addressee==self-fp match fails the demo instead of shipping silently.

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

### 2026-07-04T00:11:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2342-arc-004-dm-rail-push-wake-isolated-hub-r.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-d96b5f68
- **Timestamp:** 2026-07-04T00:21:42Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-04T00:21:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
