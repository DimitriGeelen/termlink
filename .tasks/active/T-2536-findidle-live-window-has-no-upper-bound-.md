---
id: T-2536
name: "find_idle LIVE window has no upper bound — future-dated (clock-skew) heartbeat keeps a dead worker on the idle roster"
description: >
  Bus find_idle_agents (lib.rs:604) and find_idle_agents_from_hint (lib.rs:704) filter presence heartbeats with only a lower bound (last_heartbeat_ms > now-window); no upper bound. env.ts_unix_ms is client-signed (hub cannot substitute its clock without breaking sig verify), so a host with a fast/skewed clock posts a future ts; when that process dies the dead worker stays on the find-idle roster until real-time catches up — orchestrator dispatches to a corpse, work silently stalls (G-063/G-069 'why no response?' class). Repo already establishes the convention: tools.rs:3555 'future-clock safety' (ts>now_ms skip) + tools.rs:30133 clock-skew age clamp; find_idle omits it. Fix: symmetric bound reusing live_window_ms as skew tolerance (ts <= now+window) so gross-future corpses are excluded but sub-window-skewed LIVE workers are not falsely dropped. Small, in-authority, buildable.

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
created: 2026-08-08T08:18:30Z
last_update: 2026-08-08T08:18:42Z
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

# T-2536: find_idle LIVE window has no upper bound — future-dated (clock-skew) heartbeat keeps a dead worker on the idle roster

## Context

Found by the campaign's DISCOVER-verb hunter (2026-08-08), confirmed in code.

`find_idle_agents` (`crates/termlink-bus/src/lib.rs:604`) and
`find_idle_agents_from_hint` (`:704`) filter presence heartbeats with only a
LOWER bound: `a.last_heartbeat_ms > cutoff_ms` (`cutoff = now - live_window`).
`last_heartbeat_ms` is `env.ts_unix_ms`, which is **client-supplied and signed**
(`channel.post` reads `params["ts"]`, hub channel.rs:617; the CLI signs its own
`SystemTime::now()` at channel.rs:912 and the hub cannot substitute its clock
without breaking signature verification). So a host with a fast/skewed clock
posts a future-dated heartbeat; when that process dies, the dead worker stays on
the `find-idle` roster until wall-clock catches up. The orchestrator
`find-idle → claim → contact` hands a unit to a corpse and it silently stalls —
the exact G-063/G-069 "why is there still no response?" class.

**In-repo precedent (why this is a defect, not a design choice):** the codebase
already guards future timestamps — `tools.rs:3555` (`ts > now_ms` skip, labelled
"future-clock safety") and `tools.rs:30133` (clock-skew age clamp). The DISCOVER
path omits it.

**Fix decision — symmetric bound, not zero-tolerance.** A naive `ts <= now_ms`
(the tools.rs stats convention) would drop a *legitimately LIVE* worker whose
clock is even 1s ahead — trading one silent-discovery-failure for another. Since
find-idle is DISPATCH (dropping a live worker is harmful), reuse `live_window_ms`
as a **symmetric** skew tolerance: accept `cutoff_ms < ts <= now + live_window`.
Gross-future corpses (≫ one window ahead) are excluded; sub-window skew is
tolerated. No new constant, no human policy call — reuses existing config.

## Acceptance Criteria

### Agent
- [x] Both `find_idle_agents` (lib.rs:604) and `find_idle_agents_from_hint` (lib.rs:704) reject heartbeats with `last_heartbeat_ms > now_ms + live_window_ms.max(0)` (symmetric `future_cutoff_ms` bound) in addition to the existing lower bound
- [x] Unit test `find_idle_filters_future_dated_heartbeat`: 60s window — `now-1_000` (fresh) + `now+5_000` (sub-window skew) BOTH returned; `now+600_000` (gross future / dead) EXCLUDED; total 2
- [x] Load-bearing PROVEN: temp-revert (remove walk-path upper-bound clause) → `future_dead` reappears → test FAILS "gross-future-dated heartbeat must be excluded (corpse-looks-live bug)"; restore → green
- [x] `cargo build -p termlink-bus` clean; `cargo test -p termlink-bus --lib find_idle` 18/18 pass (existing stale/anti-join/overflow tests unaffected)

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

cargo test -p termlink-bus --lib find_idle_filters_future_dated_heartbeat

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

**Symptom:** `agent find-idle` keeps a DEAD worker on the idle roster (and the
orchestrator dispatches work that never gets a receipt) whenever that worker ran
on a host whose clock was ahead of the hub — the G-063/G-069 "why is there still
no response?" class, but originating in DISCOVER rather than the comms rail.

**Root cause:** the LIVE-window filter clamped only the lower bound
(`last_heartbeat_ms > now - live_window`) with no upper bound. `last_heartbeat_ms`
is the client-SIGNED `ts_unix_ms`; the hub cannot rewrite it without breaking
signature verification, so a future-dated heartbeat survives the filter until
wall-clock time catches up to the forged/skewed timestamp.

**Why structurally allowed:** the future-timestamp guard exists elsewhere in the
repo (`tools.rs:3555` "future-clock safety", `tools.rs:30133` skew clamp) but was
never applied to the DISCOVER path, and the only find-idle staleness test
(`find_idle_filters_stale_outside_live_window`) covered the PAST boundary only —
the future/skew case was untested, so CI was blind to it since find-idle shipped.

**Prevention:** add the symmetric upper bound (reusing `live_window_ms` as skew
tolerance) to both find_idle paths + a regression test asserting a gross-future
heartbeat is excluded WHILE sub-window skew is tolerated (the load-bearing AC).
Campaign follow-up thought (noted, not built): a general "signed client ts used
in a liveness/window comparison without an upper clamp" is a small class — grep
for `> cutoff`/`> now -` filters on `ts_unix_ms`-derived fields would surface any
sibling; deferred (one-bug-one-task).

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

### 2026-08-08T08:18:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2536-findidle-live-window-has-no-upper-bound-.md
- **Context:** Initial task creation

### 2026-08-08T08:18:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
