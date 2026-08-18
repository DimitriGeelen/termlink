---
id: T-2517
name: "command.signal targets host pid not PTY child — self-DoS + child orphan on
  --shell sessions"
description: >
  handle_command_signal delivers to reg.pid (std::process::id, the host RPC server)
  instead of the PTY child; --shell signal KILL kills the server itself and orphans
  the child + leaks the PTY fd. Fix: branch on ctx.pty (child_pid) with reg.pid fallback.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/handler.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-03T21:45:49Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-03T21:50:27Z
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
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2517: command.signal targets host pid not PTY child — self-DoS + child orphan on --shell sessions

## Context

`handle_command_signal` (crates/termlink-session/src/handler.rs) delivers the
requested signal to `reg.pid`, which `Registration::new` hard-codes to
`std::process::id()` (registration.rs:259 — the termlink host RPC-server
process). For a `--shell` (PTY) session the process the user means to control is
the forked shell **child** (distinct pid, held in `SessionContext.pty`), so
`termlink signal <session> KILL` SIGKILLs the *server itself*, `Drop for
PtySession` never runs, and the shell child is orphaned + its PTY master fd
leaks. The correct child target (`PtySession::child_pid()` / `signal()`) already
exists and is used for cleanup, but the RPC dispatch passes only
`&ctx.registration`, so the handler structurally cannot reach the child. The
sibling `handle_command_resize` (which takes full `ctx` and branches on
`ctx.pty`) is the fix template. Non-PTY registrations (`pty: None`) must keep
targeting `reg.pid` (the external registrant is the intended target there).
Found by T-2468 campaign firing #36 (PTY/session-control lens).

## Acceptance Criteria

### Agent
- [x] `handle_command_signal` takes `&SessionContext` and targets
      `ctx.pty.as_ref().map(|p| p.child_pid()).unwrap_or(ctx.registration.pid)`,
      and the success response's `pid` field reflects that resolved target (not
      unconditionally `reg.pid`).
- [x] The `COMMAND_SIGNAL` dispatch arm passes full `ctx` (not `&ctx.registration`).
- [x] A load-bearing regression test (`signal_targets_pty_child_not_host`) builds
      a `SessionContext::with_pty` over a real `PtySession`, calls the handler with
      `signal:0` (non-lethal existence probe), and asserts the response `pid`
      equals `pty.child_pid()` and does NOT equal `std::process::id()`. Verified
      load-bearing by temp-reverting the fix → test fails (target becomes the host pid).
- [x] Non-PTY path unchanged: `signal_non_pty_targets_registration_pid` asserts a
      `SessionContext::new` (pty=None) signal still targets `reg.pid`.
- [x] `cargo build --release -p termlink-session` compiles and both new tests pass
      (6 passed; 0 failed).

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
grep -q "send_signal(target_pid, signal)" crates/termlink-session/src/handler.rs
cargo test -p termlink-session --lib signal_ > /tmp/t2517-verify.txt 2>&1 && grep -q "test result: ok" /tmp/t2517-verify.txt

## RCA

**Symptom:** `termlink signal <session> <SIG>` against a `--shell` (PTY) session
does not affect the interactive shell. `signal <session> KILL` instead terminates
the termlink host RPC server itself; the shell child is orphaned and its PTY
master fd leaks (repeatable → fd exhaustion). `signal <session> INT` on a stuck
foreground command does nothing to that command.

**Root cause:** `handle_command_signal` delivered the signal to `reg.pid`.
`Registration::new` (registration.rs:259) hard-codes `pid = std::process::id()` —
the host process. For a PTY session the controllable process is the forked shell
**child** (distinct pid in `SessionContext.pty`), so the host was always the
wrong target. The correct child target (`PtySession::child_pid()` / `signal()`)
existed and was used for cleanup (`Drop for PtySession`, session.rs teardown) but
the RPC dispatch handed the handler only `&ctx.registration`, so it *structurally
could not* reach the child.

**Why structurally allowed:** the signal handler was the one PTY-affecting command
whose dispatch arm passed `&ctx.registration` instead of full `ctx` — its siblings
`command.inject` and `command.resize` both take `ctx` and branch on `ctx.pty`.
That single asymmetry silently scoped the child out of reach, and no test exercised
`command.signal` against a real PTY context (existing `command_signal_to_self` used
a non-PTY ctx where `reg.pid == the target` masks the bug entirely).

**Prevention:** `signal_targets_pty_child_not_host` drives the full dispatch→handler
path over a real `SessionContext::with_pty` and asserts the resolved target is the
child pid, not `std::process::id()` — proven load-bearing (reverting the fix fails
it). `signal_non_pty_targets_registration_pid` locks the external-registrant path
so a future refactor can't regress it the other way. The handler doc-comment now
states the reg.pid == host-process invariant explicitly.

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

### 2026-08-03T21:45:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2517-commandsignal-targets-host-pid-not-pty-c.md
- **Context:** Initial task creation

### 2026-08-03T21:46:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-5447ff4e
- **Timestamp:** 2026-08-03T21:50:28Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-03T21:50:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
