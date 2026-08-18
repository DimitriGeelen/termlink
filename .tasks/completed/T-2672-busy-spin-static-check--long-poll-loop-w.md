---
id: T-2672
name: "busy-spin static check — long-poll loop with no sleep-on-error backoff"
description: >
  busy-spin static check — long-poll loop with no sleep-on-error backoff

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
created: 2026-08-13T07:04:41Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-13T07:17:53Z
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
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
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

# T-2672: busy-spin static check — long-poll loop with no sleep-on-error backoff

## Context

G-019 prevention for the T-2658/T-2636/T-2640/T-2670/T-2671 busy-spin class.
A `loop {}` that re-dispatches a long-poll RPC (`event.subscribe` /
`event.collect` / `event.poll`) whose error arm re-iterates with **no
`tokio::time::sleep`** busy-spins a CPU core the instant the hub goes
dead/half-open: on a live hub the long-poll paces the loop, but a dead hub
errors near-instantly, so a bare `continue` re-dispatches with zero delay —
silently (the `warn!` is gated out at the default log level). T-2670 fixed two
such sites in `agent.rs` and T-2671 fixed one in `tools.rs`; the 500ms
sleep-on-error is the established convention (`events.rs:805/900/1349`,
`dispatch.rs` `COLLECT_ERR_BACKOFF`). But the convention was **discipline-only**
— nothing detected a NEW long-poll loop shipped without the backoff. This task
makes the convention load-bearing with a source-level static check, the 4th
sibling of `check-silent-exit.sh` (T-2666), `check-alloc-sink-clamps.sh`
(T-2527), and `check-drain-sink-caps.sh` (T-2531).

The signature is narrow and low-FP by construction: the candidate set is ONLY
`loop {}` blocks whose brace-matched body dispatches one of the three long-poll
RPC method strings. The 74 analytics loops in `tools.rs` and the 5 known-safe
loops (bounded recv / accept-blocks / Err-arm-breaks / bounded-SQLite-drain)
do not dispatch those strings, so they are excluded automatically. A candidate
loop is CLEAN when its body contains a `tokio::time::sleep` (the backoff) OR
every long-poll dispatch is guarded such that the error path exits the loop;
confirmed-safe exceptions are acknowledged in an allowlist with cited reasons.

## Acceptance Criteria

### Agent
- [x] `scripts/check-busy-spin.sh` exists: brace-matches each `loop {}` body,
      selects only bodies dispatching `"event.subscribe"`/`"event.collect"`/
      `"event.poll"`, and FIRES on any such body with no `tokio::time::sleep`
      and not cleared by the allowlist. Exit 0=clean / 1=firing / 2=tooling;
      `--json`, `--quiet`, `--no-heartbeat`, `--root` (repeatable), `--allowlist`.
- [x] The current tree scans CLEAN (0 unacknowledged) — every convention-correct
      long-poll loop carries the backoff; any genuinely-safe exception is
      allowlisted in `.context/working/.busy-spin-allowlist` with a cited reason.
      (14 long-poll loops scanned: 3 fixed in T-2673, 4 allowlisted exit-on-error.)
- [x] Load-bearing proof: `tests/busy-spin-check-fixtures.sh` (no live binary)
      asserts the check FIRES on a synthetic no-sleep long-poll loop and is
      CLEAN on the same loop with a sleep added. (8/8 fixtures pass.)
- [x] `--root`/`--allowlist` fixture seams work (used by the fixtures test).
- [x] CLAUDE.md documents the check under the source-level static-check family.

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

bash scripts/check-busy-spin.sh
bash tests/busy-spin-check-fixtures.sh
out=$(grep -c "check-busy-spin" CLAUDE.md 2>&1); [ "${out:-0}" -ge 1 ]

## RCA

**Symptom:** The busy-spin class (a long-poll retry loop whose error arm
re-iterates with no `tokio::time::sleep`) kept recurring — T-2658/T-2636/T-2640
as runtime instances, then T-2670/T-2671 fixed three sibling sites — and on this
check's very first run THREE MORE un-migrated instances surfaced
(`execution.rs cmd_request`, `file.rs cmd_file_receive`, `remote.rs
cmd_remote_events`, fixed in T-2673). The class pins a CPU core the instant a hub
goes dead/half-open, silently (the `warn!` is gated out at the default log level).

**Root cause:** the "500ms sleep-on-error before re-dispatching a long-poll RPC"
convention was followed by DISCIPLINE ONLY (`events.rs`/`dispatch.rs`) — nothing
detected the next long-poll loop shipped without the backoff. Each omission was a
one-line gap invisible until a human read that exact loop.

**Why structurally allowed:** the framework had no source-level guard for the
convention. Fixing an instance (T-2670/T-2671) removed that instance but left the
mechanism that lets the next one land undetected — the G-019 "fix the symptom,
then close the blindness" gap.

**Prevention:** THIS task — `scripts/check-busy-spin.sh`, a source-level static
check (4th sibling of the T-2527/T-2531/T-2666 checks) that makes the convention
load-bearing: any long-poll `loop {}` with no sleep-on-error fires until fixed or
allowlisted-with-reason. Proven load-bearing by `tests/busy-spin-check-fixtures.sh`
(8/8) and by the real-tree revert proof (reverting the T-2673 sleep re-fires it).
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

### 2026-08-13 — check found 3 genuine defects + 4 safe sites; split fix from prevention
- **Chose:** On first run the check fired on 7 long-poll loops. Verified each in
  code. 4 are SAFE (error arm exits the loop): `events.rs:987 cmd_wait`
  (`Err(_) => bail!`), `tools.rs:13202 termlink_request`, `tools.rs:13346
  termlink_wait`, `tools.rs:14739 termlink_agent_ask` (all `Err => return
  json_err`) — allowlisted with cited reasons. 3 are GENUINE busy-spin defects
  (error arm re-iterates with no backoff, T-2670/T-2671 class): `execution.rs:233
  cmd_request`, `file.rs:788 cmd_file_receive` (`Ok(Err(e))` instant-error arm),
  `remote.rs:1920 cmd_remote_events` (both arms, UNBOUNDED loop) — routed to a
  separate bug-class fix task **T-2673** (mirrors the T-2666-check /
  T-2667-migration split). The check lands clean only after T-2673 fixes land +
  the 4 safe sites are allowlisted.
- **Why:** The check is the PREVENTION (G-019); the 3 instances are bug-class and
  each deserves a proper RCA — folding them into a "static check" build would
  bury their causality (one-bug-one-task / the T-2667 precedent). Separating
  keeps the check's clean-tree claim honest: it fires on real defects until they
  are fixed, not silenced.
- **Rejected:** (a) fold all fixes into T-2672 — conflates prevention with
  instances, no bug-class RCA. (b) allowlist all 7 — would silence 3 real
  defects, defeating the check's purpose.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-13T07:04:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2672-busy-spin-static-check--long-poll-loop-w.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c519a8fd
- **Timestamp:** 2026-08-13T07:18:11Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-13T07:17:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
