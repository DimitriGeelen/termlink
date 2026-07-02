---
id: T-2316
name: "arc-004 WP1: be-reachable push-waker rings PTY doorbell on inbox.queued (T-2315 GO, Option A)"
description: >
  Background WS-push waker in the be-reachable lifecycle: holds a channel subscribe --push on the session inbox and fires the existing PTY doorbell ring on inbox.queued, degrading to poll on drop. Delivers arc-004's instant-wake value.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: ["arc:push-transport"]
components: [scripts/be-reachable.sh]
related_tasks: ["T-2315", "T-2314", "T-1800", "T-1834"]
arc_id: push-transport
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-02T21:23:41Z
last_update: 2026-07-02T21:34:01Z
date_finished: 2026-07-02T21:34:01Z
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

# T-2316: arc-004 WP1: be-reachable push-waker rings PTY doorbell on inbox.queued (T-2315 GO, Option A)

## Context

Build slice WP1 of the **T-2315 GO** (arc-004 `push-transport`, Option A): make the
shipped hub→client WS push (`channel subscribe … --push`, T-2309/2310/2313/2314)
*load-bearing* for live agents by adding a background push-waker to the `be-reachable`
lifecycle that rings the **existing** PTY doorbell on an inbound inbox deposit.

**Code-grounded design (refines the inception one-liner):** the hub emits `inbox.queued`
only for posts to `inbox:<id>` topics (channel.rs:748/752; a `dm:*` post does NOT fire it —
test channel.rs:3034). The frame carries `addressee_session_id`, `channel`, `message_offset`,
`enqueued_at`. So the waker subscribes to `inbox.queued --push`, filters to frames whose
`addressee_session_id` matches this session's inbox id, and fires the same ring
`agent-send.sh` uses: `termlink inject <pty_session> "/check-arc respond" --enter`.

Value scoping: the `dm:*` doorbell+mail rail already wakes the receiver instantly via the
sender's ring-1 inject; the waker adds instant wake for the **inbox-deposit / store-and-forward
/ no-live-sender** receive paths, where the receiver would otherwise wait for its next poll
cycle (the 15 s floor T-2303 §10.1 set out to remove). Additive, reversible, durability
untouched — WS is a faster trigger, never a source of truth. Bash implementation matches the
existing `be-reachable.sh` / `agent-send.sh` idiom (smallest change, no Rust). Predecessors:
T-2315 (inception GO), T-2314 (active reconnect the waker inherits), T-1800/1804/1834
(doorbell+mail + PTY ring reused unchanged).

## Acceptance Criteria

### Agent
- [x] A push-waker (`scripts/be-reachable-pushwaker.sh`) holds `termlink channel subscribe inbox.queued --push`, parses each frame, and on a frame whose `addressee_session_id` equals the configured inbox id fires `termlink inject <pty_session> "<doorbell-text>" --enter`; frames for other addressees are ignored (no false wake). — demo: positive ring 172 ms, negative filtered (no false wake)
- [x] The waker dedupes per `(addressee, message_offset)` within a short TTL so a single deposit rings at most once; cross-rail double-wake (push + sender ring) is bounded by `/check-arc respond` idempotency and documented in the script header. — `pushwaker_dedup_ok` unit-tested (dup within ttl skips; after ttl rings)
- [x] `be-reachable.sh start` spawns the waker detached (nohup setsid) when a `pty_session` is resolved, records its pid as `pushwaker_pid` in the state file, and does NOT spawn it when `pty_session` is empty (nothing to ring). — wired in cmd_start + state file
- [x] `be-reachable.sh stop` terminates the `pushwaker_pid` (SIGTERM then KILL fallback) alongside the heartbeat pid; `status` surfaces the waker pid and its alive state. — wired in cmd_stop + cmd_status
- [x] On WS drop the waker relies on the `--push` built-in active reconnect (T-2314) and the sender/poll path remains the durability floor — no change to the durable inbox / receipts / journal (WS is trigger-only). — no hub/durability code touched (bash-only)
- [x] A pure filter/dedup unit test (`scripts/test-pushwaker-filter.sh`) covers: frame-matches-self → ring; frame-for-other → skip; duplicate offset → skip. — 8/8 checks PASS
- [x] A loopback demo (`scripts/demo-pushwaker.sh`, isolated hub + HOME) proves: a deposit to `inbox:<id>` rings the PTY sub-second via push, and a non-matching deposit does NOT ring; report line `RESULT: PASS`. — RESULT: PASS (172 ms)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
bash -n scripts/be-reachable-pushwaker.sh
bash -n scripts/be-reachable.sh
bash scripts/test-pushwaker-filter.sh
out=$(bash scripts/demo-pushwaker.sh 2>&1); echo "$out" | grep -q "RESULT: PASS"

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

### 2026-07-02 — code reality refines the inception's wake-target
- **What changed:** The T-2315 inception framed the waker as `subscribe inbox:<self> --push`
  ringing on any inbound DM. Tracing the hub, `inbox.queued` fires ONLY for posts to
  `inbox:<id>` topics (channel.rs:748/752); a `dm:*` post explicitly does NOT emit it
  (test channel.rs:3034). Separately, the `dm:*` doorbell+mail rail already wakes the
  receiver instantly via the sender's ring-1 `inject` (agent-send.sh:366) — so there is
  no 15 s floor on that path to remove.
- **Plan impact:** The waker's delivered value narrows to the **inbox-deposit /
  store-and-forward / no-live-sender** receive paths (where the receiver waits for its
  own poll cycle). Subscribe target is the hub-wide `inbox.queued` aggregator stream,
  filtered client-side to `addressee_session_id == <self inbox id>` — not a per-topic
  `inbox:<self>` subscription. Implemented in bash (matches be-reachable/agent-send
  idiom), reusing the existing `termlink inject … --enter` ring unchanged.
- **Triggered:** WP2 (loopback degrade-to-poll + no-double-wake proof) may warrant its
  own task after WP1's core push→ring demo lands; decide at slice boundary.

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

### 2026-07-02T21:23:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2316-arc-004-wp1-be-reachable-push-waker-ring.md
- **Context:** Initial task creation

### 2026-07-02T21:34:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
