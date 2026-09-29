---
id: T-2662
name: "cmd_file_receive swallows mid-transfer RPC error into tracing.warn — user waits
  out full timeout"
description: >
  file receive swallows RPC error into tracing warn

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
created: 2026-08-12T20:34:48Z
last_update: 2026-09-29T16:14:08Z
date_finished:
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
  - ts: '2026-09-08T21:30:30Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-08T21:30:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=172,acs=6)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-27T21:34:05Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (1-file-ref-derived-T-3189); tier=2 
      (workflow:build); effort=8 (lines=172,acs=6)
    rubric_sha: e4a00f38e801
---

# T-2662: cmd_file_receive swallows mid-transfer RPC error into tracing.warn — user waits out full timeout

## Context

Verified in code (round-12 silent-failure hunt, 2026-08-12).
`cmd_file_receive` (file.rs:810-812) handles a mid-transfer RPC error as
`Ok(Err(e)) => { tracing::warn!("RPC error: {}", e); }` — logged ONLY via
`tracing::warn` (invisible on the default `termlink=info` filter), then the loop
retries silently until the generic `"Timeout waiting for file transfer ({}s)"`
fires. The user waits out the FULL timeout and never learns the source session
dropped. The loud sibling is `cmd_wait` (events.rs:998-1008), which treats a
subscribe-loop `Err` as a hard, user-visible `"Session '{}' disconnected while
waiting"` bail (+ JSON `reason:"disconnected"`). Directive #2 (no silent failure).

## Acceptance Criteria

### Agent
- [x] A resolution is chosen + recorded in `## Decisions`: surface persistent/disconnect RPC errors to the user (stderr / JSON `reason`) vs. bail-on-disconnect like `cmd_wait`. (Design call: how many consecutive errors before surfacing? Is a single transient error tolerable?)
- [x] `cmd_file_receive` no longer swallows a disconnect into `tracing::warn` only — the user sees an actionable message before/instead-of waiting out the full timeout
- [x] Behavior proven (fixture: a source session that drops mid-transfer) OR structural check as fallback
- [x] `cargo build -p termlink` clean

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

cargo test -q -p termlink --bin termlink receive_
cargo build -q -p termlink
bash scripts/check-busy-spin.sh --no-heartbeat

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

**Symptom:** `termlink file receive` against a source session that died mid-transfer printed nothing and waited out the whole `--timeout`, then reported a generic "Timeout waiting for file transfer".
**Root cause:** the instant-error arm (`Ok(Err(e))`) only called `tracing::warn!`, which the default `termlink=info` filter hides, and then retried until the overall timeout.
**Why structurally allowed:** the busy-spin check (T-2672) forced a sleep into this arm but only asks "does it spin"; it does not ask "does it tell anyone". The silent-exit check only covers bare `exit`. A warn-only error arm in a retry loop matches neither shape.
**Prevention:** a bail helper with a unit test, plus a structural test pinning the arm's wiring (the count, the `if … {` bail, the `reason:"disconnected"` payload, the reset on success). Mutant `if false && …`: red.

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

**Symptom:** `termlink file receive` against a source that disconnects mid-transfer
appears to hang for the full transfer timeout, then reports a generic timeout — the
real cause (peer disconnected) is never shown.

**Root cause:** the `Ok(Err(e))` arm at file.rs:810 logs to `tracing::warn` (below
the default log level) and continues, converting a hard disconnect into a silent
wait-out-the-timeout.

**Why structurally allowed:** no convention forces RPC-loop error arms to surface to
the user; the loud sibling (`cmd_wait`) exists but the pattern was never applied
here. Same silent-discard class as T-2657 (broadcast) + T-2644 (attach).

**Prevention:** a shared "loop-error surfacing" convention + the round-12
`if let Ok(_)` / `tracing::warn`-only-on-user-path static-check candidate.

**Filed not built:** requires a design call (retry-tolerance vs. fail-fast) + an
async transfer fixture — design-decision + fixture class.

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

### 2026-09-29 — surface, then bail after a short streak (T-3211 R6)
- **Chosen:** the first transport error is printed to stderr (`RPC error talking to '<t>': <e> (retrying)`, text mode). After **3 consecutive** errors (`RECEIVE_RPC_ERROR_BAIL_AFTER`) the command bails like `cmd_wait`: text `Session '<t>' disconnected during file receive (… last: <e>); received N/M chunks`, JSON `{ok:false, reason:"disconnected", chunks_received, chunks_expected}`. A success resets the streak.
- **Why 3, not 1:** a single refused connect can be a blip, and the arm already backs off 500ms (T-2673), so 3 consecutive errors means ~1.5s of refused connections: the source is gone. **Why not "surface only"**: printing and then waiting out the full timeout still leaves the user waiting for nothing.
- **Out of scope, filed as a finding, not fixed:** the `Err(_)` RPC-*timeout* arm `continue`s past the loop-bottom overall-timeout check, so a session that accepts but never replies can keep the receive looping beyond `--timeout`. That is a separate bug (one bug = one task).

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

### 2026-09-29 — T-3211 R6 evidence
- AC3 met by the **structural fallback**, not a dropped-source fixture: `cmd_file_receive` needs a registered session via `manager::find_session`, so a live-drop fixture was not built. Tests `receive_bails_after_a_short_rpc_error_streak_not_on_one_blip` + `receive_error_arm_is_wired_to_the_bail` pass. The first draft of the structural test was too weak (mutant survived) and was tightened; the mutant is now red.
- CLI suite 1163/1163; build 0 warnings; check-busy-spin rc 0; check-silent-exit rc 0.

### 2026-08-12T20:34:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2662-cmdfilereceive-swallows-mid-transfer-rpc.md
- **Context:** Initial task creation

### 2026-09-29T16:14:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
