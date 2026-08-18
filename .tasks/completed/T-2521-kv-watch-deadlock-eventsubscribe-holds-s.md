---
id: T-2521
name: "KV watch deadlock: event.subscribe holds session read guard across long-poll,
  blocking kv.set/kv.delete"
description: >
  event.subscribe (termlink_kv_watch / termlink kv watch) is read-scoped, so server.rs
  holds the session RwLock read guard across the entire long-poll wait (up to timeout_ms,
  default 5000). kv.set/kv.delete are write-scoped (session.write().await) and cannot
  acquire while the read guard is held, so a live KV watch never observes a concurrent
  kv.change AND the write stalls for the full timeout (per-session write DoS). Fix:
  dispatch event.subscribe detached from the session lock — clone the Arc<Mutex<EventBus>>
  under a brief read lock, release the session guard, then wait.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/handler.rs, 
      crates/termlink-session/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-04T10:07:32Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-04T10:13:22Z
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
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2521: KV watch deadlock: event.subscribe holds session read guard across long-poll, blocking kv.set/kv.delete

## Context

Campaign firing #40 (T-2468 subtract-and-deepen review, KV-store lens). `event.subscribe`
(what `termlink_kv_watch` / `termlink kv watch` call) is read-scoped, so `server.rs`
holds the session `RwLock` **read** guard across the entire long-poll wait
(`handle_event_subscribe`, up to `timeout_ms`, default 5000). `kv.set`/`kv.delete` are
the only write-scoped *event emitters* (`needs_write`, handler.rs:81), so they must
`session.write().await` — which cannot proceed while the subscribe holds the read guard.
Result: a live KV watch **never observes a concurrent `kv.change`** (returns empty), AND
the `kv.set` stalls for the watch's full remaining timeout (per-session write DoS). The
handler itself already scopes the *bus* mutex correctly (locks `ctx.events` only to
`subscribe()`+replay, then waits on the detached `rx`); the defect is purely that the
*caller* holds the session guard across the wait. It went unnoticed because `event.emit`
is read-scoped (two readers coexist), so ordinary event watches work — only write-scoped
emitters expose it. Fix: dispatch `event.subscribe` **detached** from the session lock —
clone the `Arc<Mutex<EventBus>>` under a brief read lock, drop the session guard, then wait.

## Acceptance Criteria

### Agent
- [x] The post-auth lock-scoping branch is extracted into a testable `handler::dispatch_scoped(session, req)` with three arms: write-scoped (`needs_write`), detached long-poll (`event.subscribe`), and read-scoped (default); `server.rs` calls it.
- [x] `event.subscribe` is dispatched detached: the session read guard is released (only the `Arc<Mutex<EventBus>>` is retained) before the long-poll wait, so a concurrent `kv.set`/`kv.delete` can acquire the write lock and emit while the watch waits.
- [x] A regression test `event_subscribe_does_not_block_concurrent_kv_set` drives a live subscribe + a concurrent `kv.set` through `dispatch_scoped` and asserts (a) the subscribe receives the `kv.change` (count==1) and (b) the `kv.set` completes well under the watch timeout. Proven load-bearing (FAILS when the detached arm is reverted to the read-guarded path).
- [x] `cargo test -p termlink-session --lib` passes (403 tests); `cargo build --release -p termlink-hub` succeeds (exit 0).

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
cargo test -p termlink-session --lib event_subscribe_does_not_block_concurrent_kv_set > /tmp/.t2521-test.out 2>&1 && grep -q "test result: ok" /tmp/.t2521-test.out
grep -q "fn dispatch_scoped" crates/termlink-session/src/handler.rs

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

**Symptom:** A live KV watch (`termlink_kv_watch` / `event.subscribe {topic:"kv.change"}`)
never reports a concurrent `kv.set`/`kv.delete` — it returns `{events:[],count:0}` — and the
`kv.set` itself hangs for the watch's full `timeout_ms` before applying (per-session write DoS).

**Root cause:** `event.subscribe` is not in `needs_write`, so `server.rs` dispatches it via the
read-scoped path, holding the session `RwLock` **read** guard for the whole
`handle_event_subscribe` long-poll. `kv.set`/`kv.delete` are write-scoped and block on
`session.write().await` for the entire wait; their `kv.change` emit therefore lands only after
the watcher's guard drops — i.e. after the watcher has already returned empty. A reader/writer
lock inversion: the *reader* (a passive long-poll) starves the *writer* (the event source).

**Why structurally allowed:** `event.emit` (the usual emitter) is also read-scoped, and two
`RwLock` readers coexist — so every ordinary event watch works, masking the defect. Only a
*write-scoped* emitter (`kv.set`/`kv.delete`) collides with the read guard, and no test paired a
live subscribe with a concurrent write-scoped emit. The lock scope is decided per-method by a
coarse `needs_write` boolean that conflates "mutates session state" with "may hold the session
lock for a long time" — a long-poll needs neither a write lock nor a *held* read lock.

**Prevention:** `dispatch_scoped` makes the long-poll a distinct third dispatch class that is
structurally detached from the session lock, plus a regression test that pairs a live subscribe
with a concurrent `kv.set` and asserts both make progress. Any future long-poll method added to
the detached set inherits the non-blocking guarantee; forgetting it is a visible omission.

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

### 2026-08-04T10:07:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2521-kv-watch-deadlock-eventsubscribe-holds-s.md
- **Context:** Initial task creation

### 2026-08-04T10:08:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-5e85327e
- **Timestamp:** 2026-08-04T10:13:25Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-04T10:13:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
