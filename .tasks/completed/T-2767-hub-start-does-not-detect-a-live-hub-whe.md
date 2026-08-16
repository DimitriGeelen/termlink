---
id: T-2767
name: "hub start does not detect a LIVE hub when the pidfile is stale — allows a second hub to steal hub.sock"
description: >
  Measured in T-2766: PID 3869961 started a second hub against /var/lib/termlink at 16:52 while supervised PID 3093442 was already serving, and took over hub.pid and hub.sock. The already-running guard keys on the PIDFILE, so when the pidfile is missing or names a dead pid the check passes even though a hub is demonstrably bound to hub.sock and serving. Result is a split-brain: unix-socket clients reach one instance, TCP clients another, with the same topic names resolving differently on ONE host. Fix direction: probe the socket for liveness (or bind-exclusively) rather than trusting the pidfile alone, and refuse loudly per Directive 2.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/pidfile.rs, crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T15:01:33Z
last_update: 2026-08-16T15:22:49Z
date_finished: 2026-08-16T15:22:49Z
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

# T-2767: hub start does not detect a LIVE hub when the pidfile is stale — allows a second hub to steal hub.sock

## Context

Found live on `.107` during T-2766. Two hubs were serving one `runtime_dir`: the
systemd-supervised PID 3093442 and PID 3869961, started manually by another session
at 16:52, which took over both `hub.pid` and `hub.sock`.

**The predicate, stated exactly.** `pidfile::acquire()` decided entirely from
`check(pidfile)`, which reads the pidfile and asks whether that PID is alive:

- no pidfile → `NotRunning` → **write it and start**
- pidfile naming a dead PID → `Stale` → clean it and start
- pidfile naming a live PID → `Running` → refuse

So the guard trusts an *assertion about* liveness rather than *evidence of* it. When
the pidfile is the thing that goes missing — which is exactly what happened — the
hub that is demonstrably bound to `hub.sock` and accepting is invisible, and a second
instance starts cleanly.

**Why it matters more than a duplicate process.** The two instances split the
substrate: unix-socket clients reach one, TCP clients the other, and the same topic
name resolves to different state on a single host. That is the G-060 per-hub-state
property showing up where nobody expects it, and it is silent — both hubs are healthy
by every check that looks at one of them.

**Fix.** `acquire_with_socket(pidfile, socket)` keeps the pidfile fast-path (a live
PID is still reported *as a PID*, which is more actionable) and, when the pidfile
claims nobody is home, probes the socket. A successful `UnixStream::connect` proves a
listener; `ECONNREFUSED` means a leftover socket file with nothing behind it, which
must still start — that unclean-shutdown recovery is what the pidfile-only check was
written for, and breaking it would trade one failure for another.

**Deliberately not TCP-probed.** The check tests the unix socket only. Adding a
non-loopback `TcpStream::connect` in `termlink-hub/src` would trip T-2569's federation
tripwire, correctly — that guard exists to stop the hub acquiring outbound hub-speaking
behaviour, and a "liveness probe" is precisely the shape a federation feature would
first appear in. `UnixStream` is local by construction and cannot reach another host.
Tripwire re-run and green.

**Residual:** a hub serving ONLY `--tcp` with no unix socket is not detected by this.
That case is out of reach without the outbound TCP dial the tripwire forbids, so it
stays with preflight Check 6 (T-2358), which caught the live incident.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The current already-running check in `hub start` is located and its predicate
      stated exactly — established as pidfile-derived, which is why it passed at
      16:52 while PID 3093442 was demonstrably bound to `hub.sock` and serving
- [x] `hub start` refuses when a hub is LIVE on the target `runtime_dir` even if
      the pidfile is missing, empty, or names a dead PID — liveness decided by
      probing `hub.sock`, not by trusting the pidfile
- [x] The refusal is LOUD and actionable per Directive #2: it names the socket it
      found alive and what to do (`hub stop`, or `systemctl status termlink-hub`
      if the live hub is unit-supervised), never a bare non-zero exit
- [x] A stale pidfile with NO live socket still starts normally — the fix must not
      break recovery after an unclean shutdown, which is the case the pidfile-only
      check was presumably written for
- [x] Regression test pins BOTH directions: live-socket-and-stale-pidfile refuses,
      dead-socket-and-stale-pidfile starts
- [x] The test fails against the pre-fix code path (load-bearing, not merely green) —
      pinned as a contrast assertion inside `live_socket_with_no_pidfile_refuses`:
      the retained pidfile-only `acquire` is asserted to (wrongly) succeed on the
      same input, so the test cannot pass for the wrong reason
- [x] `cargo test --workspace` green — CONFIRMED post-change: 3593 passed, 0 failed
      across 24 targets, exit 0. (An earlier green run in the same session predated
      the pidfile edit and was deliberately not counted; this is the post-change run.)
      Also confirmed post-change: `cargo test -p termlink-hub` 21 pass, federation
      tripwire 3 pass, guard layer 41/41.

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

cargo test -p termlink-hub pidfile
# The federation tripwire must stay green — the fix adds a connect inside the hub crate.
cargo test -p termlink-hub --test no_federation_tripwire
cargo test --workspace
# Both start paths use the evidence-checking variant, not the pidfile-only one.
test 2 -eq $(grep -c 'acquire_with_socket' crates/termlink-hub/src/server.rs)
# The refusal names the socket and both remediations (Directive #2).
grep -q 'systemctl status termlink-hub' crates/termlink-hub/src/pidfile.rs
bash scripts/run-guard-layer.sh

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

### 2026-08-16T15:01:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2767-hub-start-does-not-detect-a-live-hub-whe.md
- **Context:** Initial task creation

### 2026-08-16T15:03:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-16T15:22:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
