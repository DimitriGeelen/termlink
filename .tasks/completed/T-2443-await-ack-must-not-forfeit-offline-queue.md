---
id: T-2443
name: "await-ack must not forfeit offline-queue durability — route Unix post through the durable queue"
description: >
  await-ack must not forfeit offline-queue durability — route Unix post through the durable queue

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
created: 2026-07-21T21:37:47Z
last_update: 2026-07-21T21:37:47Z
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

# T-2443: await-ack must not forfeit offline-queue durability — route Unix post through the durable queue

## Context

Round-8 review found that `channel post --await-ack` (T-2286) posts via
`rpc_call_authed` directly, never through `BusClient`/`OfflineQueue`. If the hub
is unreachable, the first post returns `Err` → the command hard-fails and the
message is DROPPED — whereas a plain `channel post` durably queues it to
`~/.termlink/outbound.sqlite`. A reliability flag (`--await-ack`) that *removes*
durability is a foot-gun that violates the Reliability directive. Fix: on the
Unix path, route the first post through the durable queue. If the hub is down,
the message is queued (durable) and the ack-await is skipped with a clear
message — you cannot await a receipt while offline, but the message is not lost;
it flushes + dedups on reconnect. TCP cross-hub behaviour is unchanged (the
normal TCP post path also bypasses the Unix-only queue — consistent).

## Acceptance Criteria

### Agent
- [x] On the Unix socket path, `run_await_ack` performs its first post through `BusClient`/`OfflineQueue` (not raw `rpc_call_authed`), so a hub-down post is durably enqueued rather than dropped.
- [x] When the first post is `Queued` (hub unreachable), the command reports a durable-queued outcome (queue_id + "ack not awaited while offline") and exits 0 — the message is NOT lost.
- [x] When the first post is `Delivered`, the existing ack-retry loop still runs (re-posts reuse `client_msg_id` and are hub-deduped) — happy-path behaviour unchanged.
- [x] TCP cross-hub `--await-ack` behaviour is unchanged (documented as consistent with the normal TCP bypass).
- [x] `cargo test -p termlink --bin termlink` and `cargo test -p termlink-session --lib` green.

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

cargo test -p termlink-session --lib offline_queue

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

**Symptom:** `channel post --await-ack <dm-topic> "msg"` while the local hub is
unreachable exits non-zero and the message is silently dropped — no offline
queue row, no awaiting-ack row. The same post WITHOUT `--await-ack` durably
queues to `~/.termlink/outbound.sqlite` and flushes on reconnect.

**Root cause:** `run_await_ack`'s `post_fn` calls `rpc_call_authed` directly
and the ack-retry loop treats a post error as fatal (`AckRetryError::Post` →
`?`). The durable `BusClient`/`OfflineQueue` path — which the non-await post
uses — was never wired into the await-ack path. So the reliability *flag*
bypassed the reliability *mechanism*.

**Why structurally allowed:** T-2286 built the ack-retry loop as a self-contained
posting path (so the retry could own the exactly-once re-post via a stable
`client_msg_id`) and, in doing so, forked away from the offline-queue path that
plain posts use. No test exercised `--await-ack` against a down hub, so the
durability regression was invisible — the happy path (hub up) worked perfectly.

**Prevention:** The first post now goes through `BusClient::post`, so the
`PostOutcome::Queued` branch is reachable and durable; a unit test drives the
queued branch (hub-down) and asserts a row lands in the offline queue. Any
future refactor that re-bypasses the queue fails that test.

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

### 2026-07-21T21:37:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2443-await-ack-must-not-forfeit-offline-queue.md
- **Context:** Initial task creation
