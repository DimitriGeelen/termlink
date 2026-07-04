---
id: T-2344
name: "arc-004 webhook retry-loop E2E regression demo (flaky sink)"
description: >
  arc-004 webhook retry-loop E2E regression demo (flaky sink)

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
created: 2026-07-04T08:02:05Z
last_update: 2026-07-04T08:02:05Z
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

# T-2344: arc-004 webhook retry-loop E2E regression demo (flaky sink)

## Context

The webhook retry/backoff loop (T-2334: `classify_outcome` → `schedule_retry` →
`spawn_retry_loop` drain) is the exact T-2341/PL-240 defect class: a background
resilience loop whose mechanics are unit-tested but whose LIVE wiring was never
driven E2E — T-2336's smoke and T-2343's demo both test only a direct-success
dispatch. If `spawn_retry_loop` were unwired (or the runtime not shared), every
transient 5xx would silently dead-letter and nothing would surface it. This task
adds `scripts/demo-webhook-retry.sh`: a FLAKY sink (returns 503 for the first N
POSTs, then 204) proving the full retry chain live, including the
`webhook_retry_success_total` governor telemetry (T-2335). Reuses the T-2343
isolation pattern. Ground: `crates/termlink-hub/src/webhook.rs`,
`docs/operations/webhook-fan-out-recipe.md` §5.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/demo-webhook-retry.sh` exists, executable, `bash -n` clean,
  isolation-safe (temp RUNTIME_DIR + HOME + loopback flaky sink, never touches
  `:9100` or `~/.termlink`, teardown on exit), with the T-2343-style loud
  stale-binary guard and python3 SKIP path.
  *(chmod +x; `bash -n` clean; cleanup() trap; exit 2 pre-webhook guard; exit 4
  python3 SKIP.)*
- [x] RETRY LOOP proven live E2E: against an isolated hub (fast
  `TERMLINK_WEBHOOK_RETRY_INTERVAL_MS`), a `channel.post` whose first dispatch
  hits the sink's 503 is retried by the background loop until the sink returns
  204 — the sink's request log shows >=1 initial failure THEN a successful
  delivery of the SAME signed payload (HMAC verified on the final 204-served body).
  *(Run 2026-07-04: `503-served=2  204-served=1  total=3`; only the retry loop
  re-sends (fan_out fires once per post); `HMAC on retried: yes`.)*
- [x] Telemetry proven: after recovery, `hub status --governor --json` shows
  `.governor.webhook_retry_success_total >= 1` (and `webhook_enqueued_total >= 1`)
  — the T-2335 counters move with the real retry, not just the queue mechanics.
  *(`webhook_retry_success_total=1  webhook_enqueued_total=2` — inline fail
  enqueue + retry-1 503 re-enqueue + retry-2 success.)*
- [x] Demo green (exit 0) on the current tree; PASS transcript + any defect RCA in
  `docs/reports/T-2344-arc-004-webhook-retry-demo.md`.
  *(PASS first run vs fresh release binary; no product defect — wiring was
  correct; value is the reproducer.)*

## Verification

# Shell commands that MUST pass before work-completed. One per line.
test -x scripts/demo-webhook-retry.sh
bash -n scripts/demo-webhook-retry.sh
test -f docs/reports/T-2344-arc-004-webhook-retry-demo.md
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

Regression-coverage add (title matches "regression"; gate requires this section).

**Symptom:** the webhook retry loop had no live E2E evidence — every existing
smoke/demo (T-2336, T-2343) exercised only the direct-success dispatch path.

**Root cause:** T-2334 shipped the retry mechanics with 18 unit tests on the pure
helpers (classify/backoff/queue) but nothing drives a real hub through
fail-then-recover against a live consumer, so the `spawn_retry_loop` wiring is
unproven in integration.

**Why structurally allowed:** unit-tested mechanics read as "covered"; PL-240
(T-2341) established that background resilience loops can be unit-green yet
unreachable in the shipped wiring.

**Prevention:** `scripts/demo-webhook-retry.sh` — a flaky-sink reproducer that
fails if the retry loop stops draining, the classification regresses, or the
retry-success telemetry stops moving.

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

### 2026-07-04T08:02:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2344-arc-004-webhook-retry-loop-e2e-regressio.md
- **Context:** Initial task creation
