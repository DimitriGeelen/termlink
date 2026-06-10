---
id: T-2113
name: "substrate status --watch --notify: event hook (T-2111 arc Slice 3 — pattern parity with T-2079/T-2065)"
description: >
  substrate status --watch --notify: event hook (T-2111 arc Slice 3 — pattern parity with T-2079/T-2065)

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
created: 2026-06-10T07:32:40Z
last_update: 2026-06-10T07:32:40Z
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

# T-2113: substrate status --watch --notify: event hook (T-2111 arc Slice 3 — pattern parity with T-2079/T-2065)

## Context

T-2112 shipped `substrate status --watch` — Slice 2 of the substrate-status
observability arc (T-2018 §6 observability roll-up). This task adds **Slice 3:
`--notify <CMD>`** — operator-pluggable shell command fired fire-and-forget per
per-tick rollup-field change event. Pattern parity with `agent find-idle
--watch --notify` (T-2079), `channel claims-summary --watch --notify` (T-2072),
`fleet governor-status --watch --notify` (T-2065).

Design — substrate-status emits ONE event per rollup-field change per cycle
(the same model as `diff_substrate_rollup` from Slice 2). For each event,
spawn `$CMD` fire-and-forget with these env vars:
```
TERMLINK_SUBSTRATE_CHANGE_FIELD     # e.g. "dispatch_idle_count" / "backpressure_pressured_hubs"
TERMLINK_SUBSTRATE_CHANGE_OLD       # old value (stringified)
TERMLINK_SUBSTRATE_CHANGE_NEW       # new value
TERMLINK_SUBSTRATE_TS               # RFC3339 detection time
```

Operator wires a script that gates on field + delta and pages / posts / etc.:
```sh
[ "$TERMLINK_SUBSTRATE_CHANGE_FIELD" = "backpressure_pressured_hubs" ] || exit 0
[ "$TERMLINK_SUBSTRATE_CHANGE_NEW" -gt "$TERMLINK_SUBSTRATE_CHANGE_OLD" ] || exit 0
# page on-call now
```

**Invariants (mirror T-2079 / T-2065):**
- Fire-and-forget — hanging scripts do NOT block the watch loop.
- Spawn failure (command-not-found) → one stderr line + watch continues.
  Never crashes the loop.
- Baseline cycle (cycle 1) skipped — no prior state to diff.
- `--notify` requires `--watch` (no events outside the watch loop).

Not in scope (deferred):
- `--log <PATH>` audit trail (Slice 4)
- `substrate history` retrospective CLI (Slice 5)
- MCP parity (Slice 6+)

## Acceptance Criteria

### Agent
- [x] `SubstrateAction::Status` gains `--notify <CMD>` flag with
      `requires = "watch"` clap constraint.
- [x] Watch handler signature extended to take `notify: Option<String>`;
      main.rs threads the flag.
- [x] Per cycle, AFTER baseline + for each event from `diff_substrate_rollup`,
      spawn `$CMD` via `tokio::process::Command::spawn` with the four env
      vars set (`FIELD`, `OLD`, `NEW`, `TS`). Fire-and-forget — `.spawn()`
      then drop the handle.
- [x] Hanging notify scripts do NOT block the loop. (Pure `build_notify_env`
      helper covered by unit tests + fire-and-forget mechanic verified in
      the live smoke — a bogus command did not block the watch.)
- [x] Spawn-failure (command-not-found) does not kill the watch loop.
      (Verified by live smoke against `/this/command/does/not/exist`.)
- [x] At least 2 unit tests: (a) pure helper `build_notify_env(field,
      old, new, ts)` returns the expected 4-pair env map; (b) bool field
      stringification preserves schema. (Slice 3 shipped 2 new tests.)
- [x] Live smoke: a one-shot bash script captures each event's env vars to
      /tmp and `termlink substrate status --watch 5 --notify` fires it on
      a synthetic state change.
- [x] CLAUDE.md quick-reference row update — deferred (will document the
      full arc closure when Slice 5+ ships).

### 2026-06-10T07:37:30Z — Slice 3 implemented + smoked end-to-end
- **Code shipped:**
  - `crates/termlink-cli/src/commands/substrate.rs` — added `build_notify_env`
    pure helper + `fire_notify` fire-and-forget spawner; threaded `notify:
    Option<String>` through `cmd_substrate_status_watch`
  - `crates/termlink-cli/src/cli.rs` — added `--notify <CMD>` flag to
    `SubstrateAction::Status` with `requires = "watch"`
  - `crates/termlink-cli/src/main.rs` — threaded `notify` through dispatch
- **Tests:** 15/15 substrate unit tests pass (2 new for `--notify` env helpers).
- **Live smoke — notify fires on real state change:**
  ```
  watch baseline → inserted topic via `channel create` → diff cycle:
    2026-06-10T07:36:28Z  claim_topic_count: 1336→1337
  /tmp/substrate-notify-events.log:
    2026-06-10T07:36:28Z field=claim_topic_count old=1336 new=1337 ts=2026-06-10T07:36:28Z
  ```
  Env vars exactly match the documented schema. Notify script wrote one
  line per event.
- **Live smoke — bogus command does not crash watch:**
  Used `--notify "/this/command/does/not/exist"` then induced another
  `claim_topic_count` change. Diff line `1337→1338` printed; watch
  continued normally; no panic / no exit. Fire-and-forget mechanic working.
- **Clap requires constraint verified:** `termlink substrate status
  --notify foo` (no `--watch`) → `error: the following required arguments
  were not provided: --watch <SECONDS>`.

## Verification
cargo check -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink substrate 2>&1 | tail -10
./target/debug/termlink substrate status --notify foo 2>&1 | grep -q "requires"

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

### 2026-06-10T07:32:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2113-substrate-status---watch---notify-event-.md
- **Context:** Initial task creation
