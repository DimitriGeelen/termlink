---
id: T-2670
name: "cmd_agent_negotiate subscribe-retry Err arm has no backoff — CPU busy-loop, silent"
description: >
  cmd_agent_negotiate's event.subscribe retry loop (agent.rs:239-241) logs+loops on Err with no backoff sleep; a dead/half-open hub errors instantly so the loop pins a CPU core silently until timeout_dur expires. Same T-2658/T-2636/T-2640 busy-spin class; sibling cmd_watch_hub already backs off 500ms.

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
created: 2026-08-12T22:27:54Z
last_update: 2026-08-12T22:28:16Z
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

# T-2670: cmd_agent_negotiate subscribe-retry Err arm has no backoff — CPU busy-loop, silent

## Context

Found during the T-2468 critical-review mandate (round 16) via the busy-spin probe that
also produced T-2669. `cmd_agent_negotiate` (agent.rs:155) runs a `loop {}` around
`client::rpc_call(reg.socket_path(), "event.subscribe", …)`. Its `Err(e)` arm
(agent.rs:239-241) does `tracing::warn!("Subscribe error: {}", e);` and falls through to
the loop top with **no backoff sleep**. This is the exact T-2658 defect that was fixed in
`cmd_collect` — and the same file's `cmd_watch_hub` (events.rs:900) already sleeps 500ms on
this identical error.

## Acceptance Criteria

**Scope note:** agent.rs has TWO sibling subscribe loops with the identical no-backoff Err
arm — `cmd_agent_negotiate` (line 240) and `cmd_agent_listen` (line 348). Same bug class,
same root cause (agent.rs loops never swept when T-2658 hardened events.rs), same one-line
fix. `cmd_agent_listen` is worse: its `timeout_dur` is `Option`, so with no `--timeout` the
loop is UNBOUNDED — an infinite busy-spin on a dead hub. Both are fixed here (splitting one
root-cause omission across two tasks would dilute causality, not preserve it).

### Agent
- [x] BOTH `cmd_agent_negotiate` and `cmd_agent_listen` subscribe `Err` arms sleep `Duration::from_millis(500)` before the loop repeats, mirroring `cmd_watch_hub` (events.rs:900) and the T-2636/T-2640/T-2658 backoff convention.
- [x] A `// T-2670` comment at each site cites the divergence + names the sibling convention (events.rs cmd_collect/cmd_watch_hub, dispatch.rs COLLECT_ERR_BACKOFF) so the sleep reads as load-bearing, not incidental.
- [x] The sleep is on the `Err` path ONLY — the `Ok` path (hub-side long-poll pacing) is untouched, so live-hub latency is unchanged.
- [x] `cargo build -p termlink` clean.
- [x] Structural check (temp-revert-provable): BOTH `event.subscribe` `Err` arms in agent.rs contain a `tokio::time::sleep(` — the check counts 2 sleep-bearing blocks; removing either makes it fail (non-zero), restoring passes.

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
cargo build -p termlink 2>&1 | tail -1
# Both subscribe Err arms must carry a backoff sleep — count must be exactly 2 (revert either → 1 → fails)
out=$(grep -A11 'tracing::warn!("Subscribe error' crates/termlink-cli/src/commands/agent.rs 2>&1); n=$(echo "$out" | grep -c "tokio::time::sleep"); [ "$n" -eq 2 ]

## RCA

**Symptom:** `termlink agent negotiate` (and callers of `cmd_agent_negotiate`) pin a CPU
core at 100% for the entire `timeout_dur` window when the target hub is down or half-open —
silently, because the only signal (`tracing::warn!`) is gated out at the default log level.

**Root cause:** the subscribe-retry loop's `Err` arm re-dispatches `event.subscribe` with no
delay. On a live hub `event.subscribe` long-polls (blocks ~`effective_timeout` ms, pacing the
loop); on a dead/half-open hub `client::rpc_call` returns `Err` near-instantly, so the loop
becomes a zero-delay busy-spin bounded only by the outer `timeout_dur`.

**Why structurally allowed:** this is the T-2658 "sibling loops diverged, only one got the
guard" pattern (events.rs:586 comment). The 500ms sleep-on-error is convention across
events.rs (cmd_collect:1349, cmd_watch_hub:900, multi-session tick:805) and dispatch.rs
(COLLECT_ERR_BACKOFF:454), but nothing enforces it — `cmd_agent_negotiate` lives in a
different file (agent.rs) and was never swept when T-2658 hardened the events.rs loops.

**Prevention:** the fix's own structural check (AC #5) proves the sleep is present + load-
bearing. The general regression-guard for this class (a static check flagging any `loop {}`
whose error arm continues with no backoff) is the broader candidate noted in the round-16
leave-off; this task fixes the one concrete divergent site rather than building the check.

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

### 2026-08-12T22:27:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2670-cmdagentnegotiate-subscribe-retry-err-ar.md
- **Context:** Initial task creation

### 2026-08-12T22:28:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
