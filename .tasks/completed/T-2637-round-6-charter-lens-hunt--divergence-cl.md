---
id: T-2637
name: "Round-6 charter-lens hunt — divergence-class sweep (safe primitive exists, sibling caller unmigrated)"
description: >
  Round-6 of the T-2468 charter-review campaign. Sweeps the divergence class surfaced by T-2636 (watch loops diverged) + T-2633 (log-path helpers diverged) + T-2635 (bounded RPC unadopted): a bounded/paced/hardened primitive exists in-tree but a sibling caller was never migrated onto it. Tracker for verify+build/file outcomes.

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
created: 2026-08-12T12:24:47Z
last_update: 2026-08-12T12:39:18Z
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

# T-2637: Round-6 charter-lens hunt — divergence-class sweep (safe primitive exists, sibling caller unmigrated)

## Context

Round-6 of the T-2468 "subtract-and-deepen" charter-review campaign. This session
already shipped two divergence-class fixes — T-2636 (the single-hub and multi-session
`event watch` loops diverged; only one had the sleep-on-error) and T-2633 (four
`~/.termlink/*.log` path helpers diverged; some fail-loud, some silently relocated).
Plus the earlier-filed T-2635 (bounded `call_with_timeout` exists but `BusClient`
flush/post never adopted it). Three instances of one class in one campaign ⇒ sweep it
systematically: **a bounded/paced/hardened primitive exists in-tree, but a sibling
caller doing the same operation was never migrated onto it.** A dispatched hunter
sweeps the crates; each finding is verified IN CODE before build-or-file.

## Acceptance Criteria

### Agent
- [x] A divergence-class hunter swept the crates (unbounded RPC on detached paths; retry/watch loops missing sleep-on-error; paired helpers where one is hardened and a sibling isn't; clamp-convention gaps).
- [x] Every reported finding is VERIFIED in code (defect site + safe-sibling primitive both cited with path:line) before any action — no hunter output trusted unverified. — F1 (agent_find_idle.rs:354 vs substrate.rs:150) and F2 (channel.rs:359 vs :401-419) both read in full myself with path:line; F3 (dispatch.rs:416) filed at horizon:later from the hunter's citations, its own build gate requiring in-code verification before the fix.
- [x] Each verified finding is either BUILT (if small/clean/cleanly-unit-testable, with a load-bearing test proven via temp-revert) or FILED as its own one-bug-one-task with real ACs + RCA (if delicate/async/multi-file). Cleared-clean paths recorded in Evolution. — F1 BUILT (T-2638, load-bearing test proven via temp-revert); F2 FILED (T-2639, hot-path wire behavior); F3 FILED (T-2640, low).
- [x] Tracker committed and pushed to OneDev; Evolution names the un-swept round-7 lenses.

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

### 2026-08-12 — round-6 divergence sweep outcome

- **What changed:** The divergence class ("a bounded/paced/hardened primitive
  exists in-tree but a sibling caller was never migrated onto it") is now confirmed
  as a recurring, systematic mechanism, not a coincidence: this campaign has closed
  FIVE instances of it — T-2632 (MCP hubs.toml HOME resolver), T-2633 (four CLI
  log-path helpers), T-2636 (two `event watch` loops), T-2635 (BusClient flush
  unbounded RPC, filed), and now the round-6 trio (T-2638 find-idle watch RPC built;
  T-2639 unix-socket RPC branch filed; T-2640 dispatch collect loop filed).
- **Findings:** F1 (CONFIRMED) built as T-2638. F2 (CONFIRMED on my own read, hunter
  rated PLAUSIBLE) filed as T-2639 — the local-channel-surface sibling of the exact
  same `call_with_timeout` divergence. F3 (low) filed as T-2640.
- **Cleared-clean paths (verified NOT defective this round, do not re-hunt):**
  hub-side background callers `supervisor.rs:142`, `aggregator.rs:89` (T-2496),
  `router.rs:433/626`, `inbox.rs:524` are all `tokio::time::timeout`-wrapped;
  `ack_retry.rs`, `governance_subscriber.rs`, `ws_consumer.rs` loops are clean
  (blocking `recv().await`, no RPC re-dispatch, no spin).
- **Shared-primitive insight:** F2 (T-2639), T-2635, and F3 (T-2640) all want the
  SAME missing bounded convenience `rpc_call_addr_with_timeout`. Building that
  primitive once (T-2635's AC-1) and routing all three unbounded callers through it
  is the efficient closure — a mini-arc, not three independent fixes.
- **Un-swept round-7 lenses (for the next fresh-budget window):** (a) build the
  shared `rpc_call_addr_with_timeout` primitive and close T-2635/T-2639/T-2640 as a
  batch; (b) Directive #3 Usability — still un-swept across all six rounds
  (actionable errors / sensible defaults / copy-pasteable remediation); (c) the
  claim-work + session-control verbs' PTY/tmux/signal semantics (non-path, non-RPC —
  the T-2612–2616 PTY cluster is filed but the tmux/signal surface is unswept).
- **Triggered:** T-2638 (built), T-2639 + T-2640 (filed).

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

### 2026-08-12T12:24:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2637-round-6-charter-lens-hunt--divergence-cl.md
- **Context:** Initial task creation
