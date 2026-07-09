---
id: T-2388
name: "tl-claude --reachable: auto-armed injectable agent launcher + @reboot re-arm (T-2380 C7+C5, arc-004 dormancy root fix)"
description: >
  tl-claude --reachable: auto-armed injectable agent launcher + @reboot re-arm (T-2380 C7+C5, arc-004 dormancy root fix)

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
created: 2026-07-09T22:56:53Z
last_update: 2026-07-09T22:56:53Z
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

# T-2388: tl-claude --reachable: auto-armed injectable agent launcher + @reboot re-arm (T-2380 C7+C5, arc-004 dormancy root fix)

## Context

T-2380 GO candidates **C7 + C5** — the root fix for arc-004 push-wake dormancy
(E4: transport shipped, zero wakers running in the field; re-verified 2026-07-10 on
.107: 0 waker processes, 0 LIVE heartbeats, 0 armed PTYs). The waker rings by
`termlink inject <pty_session> "/check-arc respond" --enter`, which only reaches
**termlink-owned PTYs** (`spawn`/`register --shell`) — a plain headless `claude`
cannot be retrofitted (PL-237). All the pieces already exist and have simply never
been composed: `scripts/tl-claude.sh` launches claude inside an injectable termlink
PTY (persistent mode survives claude restarts) but never arms be-reachable;
`scripts/be-reachable.sh start --agent-id X --pty-session NAME` arms heartbeat +
push-waker (inbox + dm rails) but assumes the PTY exists. Multi-agent-per-host works
via the existing `BE_REACHABLE_STATE` env override (singleton state file otherwise).
This task is the **glue verb**: one command → agent runs in an injectable PTY,
armed, push-reachable — and stays so across reboots (C5, @reboot cron re-arm).
Operational fork resolved per operator directive 2026-07-10 ("focus on making this
work"): structural path (launcher), not tmux-mandate.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `tl-claude.sh` gains `--reachable` (+ optional `--agent-id ID`, default = session name): after the session PTY is up (both one-shot and persistent `start` modes), it arms `be-reachable.sh start --agent-id ID --pty-session <session-name>` with a per-agent state file (`BE_REACHABLE_STATE=~/.termlink/be-reachable-<ID>.state`) so multiple agents on one host don't clobber each other. Arm failure is loud but does not kill the launched session.
- [x] `tl-claude.sh stop` also stops the paired be-reachable (same per-agent state file) so no orphan heartbeat/waker outlives the session; `tl-claude.sh status` shows the reachability state (armed + waker pid vs not armed).
- [x] New `tl-claude.sh install-boot --name N --agent-id ID [-- CLAUDE_ARGS]` writes `/etc/cron.d/termlink-agent-<ID>` (@reboot, USER-field syntax per the canary-cron convention) that re-runs `tl-claude.sh start --reachable ...` on boot — C5: wakers survive reboots without human memory.
- [x] Live E2E on .107 (the "make it work" proof): launch a scratch persistent session via `tl-claude.sh start --reachable --agent-id <demo-id>` (shell PTY, no claude needed for the ring proof), then `agent contact <demo-id>` → (a) T-2385 reachability preflight reports `recipient_live=true, waker_running=true` (the positive path, first live validation), and (b) the doorbell text `/check-arc respond` arrives in the PTY (visible via `termlink pty output <session>`) within ~2s — sub-second push-wake, not the 15s poll floor.
- [x] Idempotence + no-regression: re-running `start --reachable` with the session already up does not double-spawn (existing tl-claude persistent-mode reuse + be-reachable idempotence hold); running WITHOUT `--reachable` behaves exactly as today (pure opt-in).
- [x] Operator one-liner documented in the script header + `docs/operations/arc-004-fleet-activation.md` updated: the C7 structural path is now available (`bash scripts/tl-claude.sh start --reachable --agent-id <name> -- --resume`), replacing "mandate tmux" as the default recommendation.

**Evidence (all live on .107, 2026-07-10):** arm: `be-reachable: started … push_waker: pid 1517824 (rings PTY on inbox deposit [T-2316] + dm.queued for 7ba073b531244b92 [T-2324])` + `reachable: armed (agent-id=demo-c7 pty=demo-c7 state=/root/.termlink/be-reachable-demo-c7.state)`. E2E: `agent contact demo-c7 --json` → `{"reachability":{"recipient_live":true,"waker_running":true,"recipient_agent_backed":true,"presence_status":"LIVE","diagnosis":null}}`, posted `dm:7ba073b531244b92:d1993c2c3ec44c94 offset=0`, and `/check-arc respond` visible in `pty output demo-c7` within ~3.6s wall (1s-granularity poll detection; wake itself is the arc-004 push). Idempotence: second `start --reachable` refused with "already exists", no double-spawn. install-boot: wrote `/etc/cron.d/termlink-agent-demo-c7` (USER-field @reboot, PATH set, sleep-45 hub-settle), verified content, removed demo file. Paired stop: `stopped push-waker (pid 1517824)` + `stopped demo-c7 (pid 1514788)`, orphan scan empty, state file gone.

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

# T-2388: launcher structural checks (E2E needs a live hub; covered by AC evidence)
bash -n scripts/tl-claude.sh
out=$(bash scripts/tl-claude.sh --help 2>&1); echo "$out" | grep -q -- "--reachable"
out=$(bash scripts/tl-claude.sh --help 2>&1); echo "$out" | grep -q "install-boot"
grep -q "arm_reachable" scripts/tl-claude.sh
grep -q "BE_REACHABLE_STATE" scripts/tl-claude.sh
grep -q "tl-claude.sh start --reachable" docs/operations/arc-004-fleet-activation.md

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

**Symptom:** arc-004's push-wake — shipped, live-E2E-proven, sub-second — had a
runtime footprint of ZERO on the real fleet (re-verified 2026-07-10 on .107: 0
waker processes, 0 LIVE heartbeats, 0 armed PTYs). Every DM to every agent fell
to the poll floor or was never read: the "wait and no response" the operator kept
hitting even after WS shipped.

**Root cause:** the wake chain has a hard precondition — the waker rings by
injecting into a termlink-OWNED PTY (PL-237) — and every piece needed to satisfy
it existed (`tl-claude.sh` spawns injectable claude PTYs; `be-reachable.sh` arms
heartbeat + waker) but NO verb composed them. Arming was a 2-tool, 3-flag manual
ritual requiring tmux knowledge, opt-in per session, forgotten after every
reboot. The transport was built; the on-ramp wasn't.

**Why structurally allowed:** "shipped" was measured at the code/E2E layer, not
the capability-live layer (G-069 class). Nothing fired on "zero wakers fleet-wide"
(that gap is T-2387, next), and the launch convention was never made a first-class
verb — so dormancy was the default state and silence was its symptom.

**Prevention:** (1) one-verb on-ramp (`start --reachable`) makes armed the easy
default at launch; (2) `install-boot` removes the reboot-amnesia failure mode;
(3) paired `stop` prevents orphan-waker cruft that would erode trust in the rail;
(4) T-2385's preflight now makes an un-armed recipient LOUD on every send
(`waker_running=false` + WARNING), so fleet dormancy can never again be invisible
at the point of use; (5) T-2387 (waker-liveness canary) is the standing guard.

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

### 2026-07-09T22:56:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2388-tl-claude---reachable-auto-armed-injecta.md
- **Context:** Initial task creation
