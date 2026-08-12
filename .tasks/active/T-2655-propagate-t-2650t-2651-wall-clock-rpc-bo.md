---
id: T-2655
name: "Propagate T-2650/T-2651 wall-clock RPC bound to structural twins in agent.rs + agent_find_idle.rs (naked rpc_call ignores --timeout)"
description: >
  Divergence class: agent ask/negotiate emit+subscribe and find-idle one-shot use naked rpc_call/rpc_call_addr with no wall-clock bound, twins of the T-2650/T-2651 execution.rs fixes.

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
created: 2026-08-12T19:55:23Z
last_update: 2026-08-12T19:56:23Z
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

# T-2655: Propagate T-2650/T-2651 wall-clock RPC bound to structural twins in agent.rs + agent_find_idle.rs (naked rpc_call ignores --timeout)

## Context

Round-11 divergence hunt (verified in code). The T-2650/T-2651 finding
(execution.rs `cmd_request` emit + cursor RPCs use naked `client::rpc_call`
with no wall-clock bound, so a `--timeout` flag does not cover the emit/connect
phase — a wedged hub hangs forever) has **structurally identical twins** that
were never migrated. The hardened primitive already exists:
`termlink_session::client::rpc_call_addr_with_timeout` (client.rs:306, T-2641 —
bounds BOTH connect and read; already load-bearing-tested by
`rpc_call_addr_with_timeout_bounds_a_silent_server`, client.rs:641). The default
timeout convention is `rpc_read_timeout()` (channel.rs:364 — env
`TERMLINK_RPC_READ_TIMEOUT_SECS`, clamp [1,600], default 30s).

Unmigrated twin sites (all verified present):

1. **agent.rs:103** — `cmd_agent_ask` emits `event.emit` via naked `rpc_call`
   on a `--timeout` command (timeout_dur built line 140); the timeout bounds
   only the reply-wait loop, never the emit. Direct twin of T-2651.
2. **agent.rs:455** — `cmd_agent_negotiate` emits `negotiate.attempt` via naked
   `rpc_call` on a `--timeout` path (line 468). Same shape, second command.
3. **agent.rs:78 / :165 / :293 / :408 / :483** — the pre-emit cursor-snapshot
   and in-loop `event.subscribe` calls are all naked `rpc_call`; connect is
   unbounded, so a half-open socket hangs before any deadline check. Twin of
   T-2650.
4. **agent_find_idle.rs:443** — the one-shot find-idle query uses naked
   `rpc_call_addr` with no bound, while its `--watch` twin (line 398-403) was
   explicitly bounded by T-2637 via `bounded_watch_fetch(rpc, interval)` — whose
   own comment cites the divergence risk. This is the cleanest twin: a pure
   one-shot with an in-file hardened sibling.
5. **execution.rs:252** — `cmd_request`'s in-loop `event.subscribe` (distinct
   from the emit+cursor sites already filed as T-2650/T-2651) is also naked;
   connect can bypass the loop's `remaining` deadline. Folds into the same class.

**Why FILED, not auto-fixed:** same reason as T-2651 — proving the fix
load-bearing needs a hung-hub fixture (accept-connect-then-never-respond) at
the CLI layer, and the reply-loop sites interact with the deadline/correlation
semantics (T-2650 territory). The one-shot find-idle site (#4) is the most
readily buildable subset (drop-in `rpc_call_addr` → `rpc_call_addr_with_timeout`
with `rpc_read_timeout()`), but batching keeps the same-root-cause class in one
place for a coherent fix + shared fixture.

## Acceptance Criteria

### Agent
- [ ] agent.rs emit sites (`cmd_agent_ask` :103, `cmd_agent_negotiate` :455) bound the `event.emit` RPC by the command's timeout (via `rpc_call_addr_with_timeout` or `tokio::time::timeout`), so a wedged hub cannot hang past `--timeout`
- [ ] agent.rs subscribe/cursor sites (:78/:165/:293/:408/:483) bound the connect+read phase (a half-open socket yields a retryable/timeout error, not an infinite hang)
- [ ] agent_find_idle.rs one-shot (:443) migrated from naked `rpc_call_addr` to `rpc_call_addr_with_timeout` with a sensible default (mirror `rpc_read_timeout()`), matching its already-bounded `--watch` twin
- [ ] execution.rs `cmd_request` in-loop subscribe (:252) bounded consistent with the T-2650/T-2651 emit+cursor fix
- [ ] A hung-hub fixture test (accept-connect, never-respond) proves at least one migrated CLI path returns a timeout error instead of hanging (load-bearing for the class)
- [ ] `cargo build -p termlink` + `cargo test -p termlink --bins` pass
- [ ] Consider a static-check (grep) prevention: flag naked `rpc_call(`/`rpc_call_addr(` on any handler path that also binds a user `--timeout`/deadline — noted in T-2651 RCA

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

**Symptom:** `termlink agent ask <sess> --timeout 30`, `agent negotiate ...
--timeout N`, and `agent find-idle` (one-shot) against a half-open hub (accepts
the socket, never writes a response) hang indefinitely — the `--timeout` flag
does NOT fire because it bounds only the reply-wait loop, not the emit/connect
phase, and find-idle one-shot has no bound at all.

**Root cause:** the T-2641 hardened primitive `rpc_call_addr_with_timeout`
(bounds connect AND read) exists and is used on the watch/dispatch paths, but
the request/reply and one-shot CLI paths in agent.rs + agent_find_idle.rs still
call the naked `rpc_call`/`rpc_call_addr`. This is a migration-incompleteness
divergence: one instance (execution.rs) was found and filed (T-2650/T-2651),
its structural twins were not swept.

**Why structurally allowed:** naked vs bounded RPC is a call-site choice with no
lint. A `--timeout` flag on the command reads as "this is bounded" but only the
loop honors it; the emit/connect underneath is silently unbounded. Nothing tests
the half-open-hub case per handler.

**Prevention:** migrate all twin sites to the bounded primitive + a hung-hub
fixture test; add a static check (grep) flagging naked `rpc_call(` on any handler
that binds a user timeout/deadline, so the next such divergence is caught at
author time rather than by an adversarial hunt.

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

### 2026-08-12T19:55:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2655-propagate-t-2650t-2651-wall-clock-rpc-bo.md
- **Context:** Initial task creation
