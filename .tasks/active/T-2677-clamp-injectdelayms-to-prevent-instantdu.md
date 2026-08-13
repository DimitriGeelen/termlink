---
id: T-2677
name: "clamp inject_delay_ms to prevent Instant+Duration overflow panic on command.inject"
description: >
  clamp inject_delay_ms to prevent Instant+Duration overflow panic on command.inject

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
created: 2026-08-13T11:49:12Z
last_update: 2026-08-13T11:49:12Z
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

# T-2677: clamp inject_delay_ms to prevent Instant+Duration overflow panic on command.inject

## Context

`command.inject` (MCP `termlink_inject` / `termlink_interact`) accepts a caller-supplied
`inject_delay_ms` at `crates/termlink-session/src/handler.rs:505-509` via `unwrap_or(10)`
with **no clamp**, then feeds it to `tokio::time::sleep(Duration::from_millis(delay_ms))`
at :536. This is the one caller-supplied duration on the session-control path that escaped
the T-2530/T-2611 clamp sweep. A peer passing `inject_delay_ms: u64::MAX` (plus a key list
whose 2nd+ entry is a special key — a trivially-satisfiable `is_special && i>0` gate) hits
the exact `Instant + Duration` overflow-panic class the repo already documents and clamps
everywhere else (see the T-2611 comment at handler.rs:788-790 and the pure-clamp helper
`effective_subscribe_timeout_ms` at :765). Fix mirrors that convention: a pure
`effective_inject_delay_ms` clamp helper + `MAX_INJECT_DELAY_MS` ceiling.

Found by charter-lens adversarial sweep (verb-4 "control terminal sessions"), verified in
code before build (PL-327 sweep record).

## Acceptance Criteria

### Agent
- [ ] A pure `effective_inject_delay_ms(requested: Option<u64>) -> u64` helper clamps the
      caller value to `[0, MAX_INJECT_DELAY_MS]` and applies the default (10) when absent,
      mirroring `effective_subscribe_timeout_ms`.
- [ ] The `command.inject` handler resolves `delay_ms` through the new helper (no raw
      `unwrap_or(10)` reaching `Duration::from_millis`).
- [ ] A unit test proves the clamp is load-bearing: `u64::MAX` in → `MAX_INJECT_DELAY_MS`
      out; default applied when absent; a small in-band value passes through unchanged.
- [ ] `cargo build -p termlink-session` and `cargo test -p termlink-session --lib` pass.

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

cargo build -p termlink-session
cargo test -p termlink-session --lib inject_delay

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

**Symptom:** A peer calling `command.inject` with `inject_delay_ms: u64::MAX` (and any key
list whose 2nd+ entry is a special key) drives `tokio::time::sleep(Duration::from_millis(u64::MAX))`,
which panics on `Instant + Duration` overflow when tokio computes the deadline — taking down
the inject call (and, with a large-but-finite value, hanging it indefinitely while holding
the session read lock, blocking every write-scoped op per the T-2521 lock geometry).

**Root cause:** `inject_delay_ms` was parsed with `unwrap_or(10)` and passed to
`Duration::from_millis` with no ceiling — the single caller-supplied duration on the
control path that the T-2530 (exec) / T-2611 (subscribe) clamp sweep missed.

**Why structurally allowed:** the clamp convention was enforced by discipline, not a check.
The T-2611 sweep clamped the subscribe timeout and exec timeout but did not enumerate ALL
`Duration::from_*(caller_value)` sites; inject's inter-key delay was not on the list. The
existing static checks (alloc-sink, drain-sink, silent-exit, busy-spin) do not cover the
"unclamped caller duration → tokio timer overflow" class.

**Prevention:** the load-bearing unit test (`u64::MAX` clamps to `MAX_INJECT_DELAY_MS`) fires
if the clamp is ever removed. The pure helper mirrors the two existing siblings, so the
control-path duration-clamp convention is now uniform across exec / subscribe / inject —
the three caller-supplied durations on the session path. (A general static check for
`Duration::from_*(<unclamped caller ident>)` is a reasonable follow-up but out of scope for
this one-bug task.)

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

### 2026-08-13 — MAX_INJECT_DELAY_MS ceiling value
- **Chose:** `MAX_INJECT_DELAY_MS = 60_000` (60s) via a dedicated const.
- **Why:** Inter-key inject delays are milliseconds-to-seconds in any legitimate use
  (slow type-out for a picky TUI). 60s per key is already far beyond that, eliminates the
  `Instant + Duration` overflow (60_000 vs u64::MAX), and — because this path holds the
  session read lock across the sleep (unlike subscribe, refactored off the lock in T-2521) —
  bounds worst-case lock-hold far tighter than reusing the 1-hour `MAX_SUBSCRIBE_TIMEOUT_MS`.
- **Rejected:** reusing `MAX_SUBSCRIBE_TIMEOUT_MS` (3_600_000). Consistent numerals, but a
  1-hour-per-keystroke ceiling is nonsensical for inter-key timing and would permit
  multi-hour read-lock holds. The clamp's job is overflow-safety + sane bound, not parity.
- **Scope note:** the lock-hold-across-sleep geometry itself (a T-2521-style refactor to
  dispatch inject detached from the read guard) is a SEPARATE concern, deliberately NOT
  bundled here (one bug = one task). This task only closes the overflow/unbounded-delay hole.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-13T11:49:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2677-clamp-injectdelayms-to-prevent-instantdu.md
- **Context:** Initial task creation
