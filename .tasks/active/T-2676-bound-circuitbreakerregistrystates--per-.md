---
id: T-2676
name: "Bound CircuitBreakerRegistry.states — per-session-id map never pruned on deregister (slow OOM)"
description: >
  Bound the hub CircuitBreakerRegistry per-session-id map; it is insert-only on route failure and never pruned when a session deregisters (slow unbounded in-memory growth). Sibling of T-2675.

status: captured
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
created: 2026-08-13T08:31:44Z
last_update: 2026-08-13T08:31:44Z
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

# T-2676: Bound CircuitBreakerRegistry.states — per-session-id map never pruned on deregister (slow OOM)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

Sibling of T-2675 (bounded PresenceTracker). The hub's session-level circuit
breaker `CircuitBreakerRegistry.states: Mutex<HashMap<String, CircuitState>>`
(`crates/termlink-hub/src/circuit_breaker.rs:65`, held by
`static REGISTRY: LazyLock<CircuitBreakerRegistry>`) is insert-only for live
operation: `record_failure` does `states.entry(session_id).or_default()`
(circuit_breaker.rs:104-107) when a route to a session fails (router.rs
~1469-1477 / 1511 / 1528). The ONLY removal is `#[cfg(test)] reset()` → whole-map
`clear()` (:118-121). There is no per-entry remove, no cap, no TTL, and — the
core gap — **deregistering a session never prunes its `CircuitState`**. So every
session that ever experiences a failed route leaves a permanent entry keyed by a
now-dead session_id. Growth is gated by failure events (slower than T-2675's
presence map, hence MED not HIGH), but it is genuinely unbounded over the hub's
lifetime. Found by the same round-20 adversarial unbounded-growth hunt as T-2675;
verified in code. (The sibling `ModelCircuitBreaker` is model-name-keyed →
bounded cardinality → NOT affected.)

**Why this is filed, not built inline (delicacy):** the eviction predicate
interacts with the half-open/cooldown state machine that T-2495 *just* fixed a
subtle re-arm bug in. Evicting an entry whose circuit is currently OPEN (mid
cooldown) for a still-LIVE session would silently forget the breaker and defeat
it — the opposite of T-2495's fix. A naive cap/TTL prune (à la T-2675) is
therefore unsafe here. The design must ensure only safe-to-forget entries are
removed. See Decisions for the two candidate approaches.

## Acceptance Criteria

### Agent
- [ ] A session's `CircuitState` is pruned when that session deregisters (or an
      equivalent bound is added), so the map size tracks live/recent sessions,
      not lifetime distinct session_ids.
- [ ] The chosen eviction is proven SAFE against the half-open/cooldown state
      machine: it must NOT remove an entry whose circuit is currently OPEN for a
      live session (would defeat the breaker — the inverse of T-2495). A unit
      test asserts a live open circuit survives a prune pass while a
      dead/closed/expired entry is removed.
- [ ] A load-bearing unit test proves the bound (map does not grow with
      deregistered/expired session_ids); reverting the prune fails it.
- [ ] `cargo build -p termlink-hub` succeeds and existing circuit_breaker tests
      (incl. the T-2495 re-arm test) still pass.

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

**Symptom:** The hub's session circuit-breaker map (`static REGISTRY`,
`HashMap<session_id, CircuitState>`) grows without bound over hub lifetime: any
session that ever has a failed route leaves a permanent entry, never removed even
after the session deregisters.

**Root cause:** `record_failure` inserts via `.entry().or_default()`; no code path
removes a per-session entry outside `#[cfg(test)] reset()`. The session-teardown
(deregister) path has no hook into the registry, and the registry has no cap/TTL.

**Why structurally allowed:** Same class as T-2675 — a peer/session-keyed
in-memory map added without a retention bound, invisible to the on-disk
topic-growth canaries (T-2252/T-2562). The circuit breaker was added focused on
correctness of the OPEN/half-open logic (T-2495), not on entry lifecycle.

**Prevention:** Add a bound (prune-on-deregister preferred) + a load-bearing test
that fails if the bound is removed AND a safety test that a live OPEN circuit is
never evicted. Same learning as T-2675 (audit every long-lived peer/session-keyed
in-memory map against the negative-trail of already-capped siblings).

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

Two candidate approaches (to be decided at build time):

### Approach A — prune-on-deregister (semantically correct, cross-file)
- Add `CircuitBreakerRegistry::forget(session_id)` and call it from the session
  deregister/teardown path (server.rs / router.rs / supervisor).
- **Pro:** exactly bounds the map to live sessions; no interaction with the
  cooldown state machine (a deregistered session's breaker is meaningless).
- **Con:** must find the single authoritative deregister site and confirm it
  covers ALL teardown paths (graceful deregister, connection drop, supervisor
  reap) — a missed path re-leaks. Cross-file.

### Approach B — opportunistic self-contained prune (in-registry, delicate)
- On insert (or a periodic pass), evict entries that are safe to forget: a fully
  CLOSED entry (`consecutive_failures == 0 && opened_at.is_none()`) carries no
  info, and an entry whose `opened_at` is older than cooldown + margin has
  effectively expired.
- **Pro:** single-file, no lifecycle wiring.
- **Con:** MUST NOT evict a currently-OPEN circuit for a live session (defeats
  the breaker — inverse of the T-2495 bug). The predicate is subtle; needs the
  safety test in AC #2. Also a hard cap backstop for the pathological churn case.

Recommendation: A (prune-on-deregister) as the primary bound, with B's hard-cap
as a cheap backstop. Confirm the deregister path coverage first.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-13T08:31:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2676-bound-circuitbreakerregistrystates--per-.md
- **Context:** Initial task creation
