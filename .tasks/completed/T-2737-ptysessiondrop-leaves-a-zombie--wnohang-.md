---
id: T-2737
name: "PtySession::drop leaves a zombie — WNOHANG cannot reap a child killed microseconds earlier"
description: >
  PtySession::drop leaves a zombie — WNOHANG cannot reap a child killed microseconds earlier

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
created: 2026-08-15T13:51:42Z
last_update: 2026-08-15T13:51:42Z
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

# T-2737: PtySession::drop leaves a zombie — WNOHANG cannot reap a child killed microseconds earlier

## Context

Herdr adoption backlog rank 9 (worker 1, class M). `PtySession::drop`
(`crates/termlink-session/src/pty.rs:490-501`) sends `SIGKILL` and then
*immediately* calls `waitpid(pid, &mut status, WNOHANG)` under a comment reading
"Reap to avoid zombie processes". `WNOHANG` means "return 0 right now if the
child has not yet been reaped" — and a child SIGKILLed microseconds earlier has
almost certainly not been torn down by the kernel yet. So the call returns 0,
reaps nothing, and the zombie survives until the whole process exits. Long-lived
hosts (the MCP server, a hub) accumulate one zombie per spawned session:
bounded but real PID/fd pressure.

## Acceptance Criteria

### Agent
- [x] `reap_child_bounded(pid, budget)` exists in `pty.rs` and retries `WNOHANG` until the child is reaped or the budget elapses
- [x] It returns a `ReapOutcome` that distinguishes `Reaped` / `NoChild` / `TimedOut { waited_ms }` — the three outcomes are not collapsed into a bool
- [x] `PtySession::drop` calls it in place of the single `WNOHANG`, with a named budget constant carrying its rationale
- [x] `TimedOut` is not silent: it emits a `tracing::warn!` naming the pid and the elapsed budget (Directive #2 — a leaked zombie is a failure, not a no-op)
- [x] A test spawns a real child, kills it, and proves `reap_child_bounded` reaps it — a following `waitpid` reports `ECHILD`
- [x] A test proves the pre-fix shape is wrong: a single immediate `WNOHANG` after `SIGKILL` returns 0 (child not reaped) where the bounded reap succeeds on the same child
- [x] A test covers the `TimedOut` arm (zero budget against a live child) so the arm is not dead code
- [x] A test covers the `NoChild` arm (a pid that is not our child)
- [x] Load-bearing proof recorded: temp-reverting the reap to the single-`WNOHANG` shape fails `bounded_reap_waits_for_a_child_that_exits_later` and `bounded_reap_collects_a_killed_child`; restoring returns the tree to green with no residue in `git diff`
- [x] `cargo test -p termlink-session` green and `scripts/run-guard-layer.sh` clean

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

cargo test -p termlink-session --lib pty::tests
bash scripts/run-guard-layer.sh

## RCA

**Symptom.** Every `PtySession` that reached `drop` left a zombie behind. In a
long-lived host — the MCP server, a hub — that is one defunct process per
spawned session, retained until the host process itself exits.

**Root cause.** `drop` sent `SIGKILL` and then called
`waitpid(pid, &status, WNOHANG)` on the very next line. `WNOHANG` asks "is this
child collectable *at this instant*", and a child signalled microseconds earlier
is not: the kernel still has to schedule it, run the fatal-signal path, and tear
its address space down. So the call returned 0, collected nothing, and the
zombie survived. The temp-revert measured this rather than assuming it — with
the single-`WNOHANG` shape restored, `bounded_reap_collects_a_killed_child`
fails, which is the claim proven on real processes.

**Why structurally allowed.** Two reasons, and the second is the general one.
First, `Drop` has no return value and no caller, so its failure is invisible by
construction — there was nothing to check a result against and nothing that
could fail. Second, and this is the ninth instance of the class recorded as
PL-343: **the comment asserted the outcome the code could not achieve.** The
line read `// Reap to avoid zombie processes`, so every subsequent reader —
including reviewers looking specifically at process handling — saw an intent
statement where they needed a mechanism check, and moved on. A comment naming
the *goal* rather than the *mechanism* is unfalsifiable by reading; only running
it distinguishes the two.

**Prevention.** Four tests now pin the behaviour, and one of them,
`bounded_reap_waits_for_a_child_that_exits_later`, is deliberately built to be
*deterministic* rather than realistic: it uses a child that exits on its own
after ~50ms and never signals it, so the child is unambiguously not reapable at
call time. A test that killed the child first would depend on kill-delivery
timing and could pass by luck on a fast machine — precisely the flakiness that
would let this regress quietly. The timeout arm is covered too, so it cannot
rot into dead code, and `TimedOut` now warns rather than leaking silently
(Directive #2).

A static check for the "kill immediately followed by WNOHANG" shape was
considered and **not** built: this is the only site in the tree that reaps a
child it killed, so a guard would carry one permanent member and no realistic
second. The test is the proportionate prevention here; the transferable lesson
belongs to PL-343, which already carries it.

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

### 2026-08-15T13:51:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2737-ptysessiondrop-leaves-a-zombie--wnohang-.md
- **Context:** Initial task creation
