---
id: T-2340
name: "WS re-probe from steady poll so raw --push consumer recovers push after hard hub-down without restart"
description: >
  WS re-probe from steady poll so raw --push consumer recovers push after hard hub-down without restart

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
created: 2026-07-03T22:33:11Z
last_update: 2026-07-03T22:33:11Z
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

# T-2340: WS re-probe from steady poll so raw --push consumer recovers push after hard hub-down without restart

## Context

arc-004 push-transport follow-on, carved out of T-2338 (superseded). T-2314 built an
active WS reconnect loop for `channel subscribe --push` (`channel.rs:8568-8611`). But after
`WS_RECONNECT_MAX_ATTEMPTS` = 6 *consecutive sub-5s* reconnect failures (hub hard-down for a
few seconds), that loop `break`s to the steady poll loop (`channel.rs:8597-8601`) and never
re-probes the WS again — a long-lived raw `--push` consumer stays on the 1s poll floor until
process restart, silently losing the arc's sub-second-wake value. The push-waker path
self-heals (its outer script re-subscribes), but a raw CLI consumer does not. Fix: from the
steady poll loop, periodically re-probe the WS by re-entering the existing reconnect loop
(bounded cadence, no tight spin, reuse — not duplicate — the reconnect logic). `--push`-off
behavior must be unchanged.

## Acceptance Criteria

### Agent
- [x] After the reconnect cap degrades a raw `channel subscribe --push` consumer to the steady
      poll loop, the poll loop re-probes the WS on a bounded cadence (named constant, no tight
      spin) by re-entering the existing reconnect loop — no duplicated reconnect logic.
      → Reconnect loop extracted to `run_ws_reconnect_loop` (channel.rs:~542), called from BOTH
      the initial `if push` path AND the poll-floor re-probe (channel.rs poll tail). Cadence
      `WS_REPROBE_POLL_CYCLES = 30` (channel.rs:~460). Zero new `run_ws_push` call sites — the
      re-probe reuses the identical loop, which returns on its own anti-spin cap.
- [x] `push=false` (plain `channel subscribe`) control flow is unchanged: no re-probe, poll
      loop behaves exactly as before (guarded by the `push` flag).
      → Re-probe block is `if push { … }`; `poll_cycles_since_probe` only advances under `push`.
      Only reachable in `--follow` (the `!follow` path returns earlier). All pre-existing
      channel tests pass unchanged (468 in the targeted run).
- [x] The re-probe gating decision is a pure helper (poll-cycles → bool) unit-tested for
      below-threshold=false, at-threshold=true, and a sane threshold constant.
      → `should_ws_reprobe` (channel.rs:~468); 3 tests: `ws_reprobe_below_threshold_is_false`,
      `ws_reprobe_at_or_past_threshold_is_true`, `ws_reprobe_cadence_is_sane` — all pass.
- [x] `cargo build -p termlink` succeeds; targeted `cargo test -p termlink --bin termlink
      commands::channel` passes; FULL crate suite `cargo test -p termlink --bin termlink`
      passes (PL-238 — WS transport path touched, no filtered false-green).
      → build OK (40.7s); targeted 468 passed / 0 failed; FULL suite **956 passed / 0 failed /
      0 filtered** (30.1s). Live happy-path smoke: refactored `--push` held a healthy WS
      session 7s against the running hub with zero degrade/error on stderr.

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
out=$(cargo test -p termlink --bin termlink commands::channel 2>&1); echo "$out" | grep -q "test result: ok"
grep -q "WS_REPROBE" crates/termlink-cli/src/commands/channel.rs

## RCA

**Symptom:** A long-lived raw `termlink channel subscribe --push` consumer, after a hub
hard-down of a few seconds (6+ consecutive sub-5s WS reconnect failures), permanently falls
back to the 1s poll floor and never regains sub-second push until the process is restarted.
**Root cause:** The T-2314 reconnect loop `break`s to the steady poll loop on hitting
`WS_RECONNECT_MAX_ATTEMPTS`; the poll loop has no path back to the WS — it is a terminal
degrade, not a recoverable one.
**Why structurally allowed:** T-2314's cap was a deliberate anti-tight-spin guard, but "stop
spinning" was conflated with "stop trying forever". No test covered post-cap recovery, and the
waker path's outer re-subscribe masked the gap for the only consumer under regular observation.
**Prevention:** Re-probe on a bounded cadence from the poll loop (this fix) + a unit test on
the re-probe gating helper so the recover-after-cap contract is pinned.

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

### 2026-07-03T22:33:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2340-ws-re-probe-from-steady-poll-so-raw---pu.md
- **Context:** Initial task creation
