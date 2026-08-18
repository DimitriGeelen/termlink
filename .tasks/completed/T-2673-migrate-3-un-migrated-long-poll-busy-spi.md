---
id: T-2673
name: "migrate 3 un-migrated long-poll busy-spin sites (cmd_request, cmd_file_receive,
  cmd_remote_events)"
description: >
  migrate 3 un-migrated long-poll busy-spin sites (cmd_request, cmd_file_receive,
  cmd_remote_events)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/execution.rs, 
      crates/termlink-cli/src/commands/file.rs, 
      crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-13T07:09:48Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-13T07:16:12Z
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
  - ts: '2026-08-18T18:56:56Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2673: migrate 3 un-migrated long-poll busy-spin sites (cmd_request, cmd_file_receive, cmd_remote_events)

## Context

The T-2672 busy-spin static check, on its first run against the real tree, fired
on 3 genuine un-migrated instances of the T-2670/T-2671 busy-spin class — a
long-poll retry loop whose error arm re-iterates with no `tokio::time::sleep`
backoff. On a live hub the long-poll (`event.subscribe`/`event.collect`) paces
the loop; on a dead/half-open hub the RPC errors near-instantly, so a bare
warn-and-fall-through re-dispatches with zero delay and pins a CPU core:

1. `execution.rs:233 cmd_request` — `Err(e) => tracing::warn!(...)` then
   fall-through (the EXACT T-2670 shape). Timeout-bounded, so it busy-spins for
   the whole `--timeout` window rather than forever, but silently (warn gated
   out at the default log level).
2. `file.rs:788 cmd_file_receive` — the `Err(_)` (outer timeout) arm already
   waited `rpc_timeout` before `continue` (paced), but the `Ok(Err(e))`
   INSTANT-error arm (connection-refused on a dead hub) does `warn!` then
   fall-through with no backoff.
3. `remote.rs:1920 cmd_remote_events` — BOTH error arms
   (`RpcResponse::Error(e)` and `Err(e)`) print "…Retrying…" then fall through.
   This loop is UNBOUNDED (exits only on Ctrl+C / `max_count`), so on a dead hub
   it busy-spins forever (loud-ish via `eprintln!`, but still pins a core).

Same root-cause omission as T-2670/T-2671; blessed remediation is the 500ms
sleep-on-error (convention: `events.rs:805/900/1349`, `dispatch.rs`
`COLLECT_ERR_BACKOFF`).

## Acceptance Criteria

### Agent
- [x] `execution.rs cmd_request` `Err(e)` arm sleeps 500ms before the next iteration.
- [x] `file.rs cmd_file_receive` `Ok(Err(e))` instant-error arm sleeps 500ms
      (the `Err(_)` timeout arm is already paced and is left unchanged).
- [x] `remote.rs cmd_remote_events` BOTH error arms (`RpcResponse::Error` and
      `Err`) sleep 500ms before re-collecting.
- [x] `cargo build -p termlink` succeeds.
- [x] `scripts/check-busy-spin.sh` no longer fires on these 3 sites (they drop
      out of the firing set once the sleeps land).

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
out=$(sed -n '285,300p' crates/termlink-cli/src/commands/execution.rs 2>&1); echo "$out" | grep -q "tokio::time::sleep"
out=$(grep -A9 'tracing::warn!("RPC error' crates/termlink-cli/src/commands/file.rs 2>&1); echo "$out" | grep -q "tokio::time::sleep"
out=$(grep -c "from_millis(500)" crates/termlink-cli/src/commands/remote.rs 2>&1); [ "${out:-0}" -ge 2 ]

## RCA

**Symptom:** On a dead / half-open hub, `termlink request`, `termlink file receive`,
and `termlink remote events` (and their MCP peers where applicable) pin a CPU core.
`request` / `file receive` do it silently (the `tracing::warn!` is gated out at the
default log level) for the duration of the command's timeout window; `remote events`
does it forever (unbounded loop) while flooding stderr with "Retrying…".

**Root cause:** each is a long-poll retry loop (`event.subscribe` / `event.collect`)
whose error arm re-iterates with no `tokio::time::sleep`. The long-poll RPC paces the
loop ONLY on a live hub (the server blocks until events or timeout); when the hub is
dead/half-open the RPC returns an error near-instantly, so the bare fall-through
re-dispatches with zero delay.

**Why structurally allowed:** the 500ms sleep-on-error was a convention followed by
discipline (events.rs / dispatch.rs) but never enforced — T-2670/T-2671 fixed three
sibling sites in agent.rs/tools.rs, but these three predated the check that would have
surfaced them and were never swept. The framework was blind to the class until the
T-2672 static check existed.

**Prevention:** `scripts/check-busy-spin.sh` (T-2672) — the source-level static check
whose first run FOUND these three sites. Reverting any of these sleeps re-fires the
check on that loop, so the fix and its guard cannot silently drift apart.

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

### 2026-08-13T07:09:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2673-migrate-3-un-migrated-long-poll-busy-spi.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-60aa08e7
- **Timestamp:** 2026-08-13T07:16:46Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#5 (Agent)** — `scripts/check-busy-spin.sh` no longer fires on these 3 sites (they drop
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/check-busy-spin.sh in: `scripts/check-busy-spin.sh` no longer fires on these 3 sites (they drop`

### 2026-08-13T07:16:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
