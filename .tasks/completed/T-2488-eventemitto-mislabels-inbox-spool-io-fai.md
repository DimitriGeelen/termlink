---
id: T-2488
name: "event.emit_to mislabels inbox-spool IO failure as SESSION_NOT_FOUND and drops
  the error unlogged"
description: >
  event.emit_to mislabels inbox-spool IO failure as SESSION_NOT_FOUND and drops the
  error unlogged

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-hub/src/router.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T07:30:38Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-08-02T08:17:10Z
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
  - ts: '2026-08-18T18:56:48Z'
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
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:11Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2488: event.emit_to mislabels inbox-spool IO failure as SESSION_NOT_FOUND and drops the error unlogged

## Context

`handle_event_emit_to` (`crates/termlink-hub/src/router.rs:369`) spools file.* events for an
offline target via `crate::inbox::deposit`, which returns `std::io::Result<bool>`: `Ok(true)` =
spooled, `Ok(false)` = not a spoolable file event, `Err(e)` = a REAL inbox I/O failure
(`create_dir_all`/`fs::write` on ENOSPC / EACCES / read-only fs). The current `if let Ok(true) = …`
collapses BOTH `Ok(false)` AND `Err(e)` into the same fall-through, returning `SESSION_NOT_FOUND`
(:383) and dropping the error **unlogged**. Net: a disk failure masquerades as a missing session
and the file event vanishes with no trace of the true cause — a directive-#2 (no silent failures)
violation. Any network client calling `event.emit_to` with a `file.*` topic at a non-locally-
registered target reaches this path. Found by firing-#10 adversarial audit; verified current.

## Acceptance Criteria

### Agent
- [x] `handle_event_emit_to` distinguishes the three `deposit` outcomes via an explicit `match`:
      `Ok(true)` → existing spooled-success response (unchanged); `Ok(false)` → fall through to
      `SESSION_NOT_FOUND` (genuinely not spoolable); `Err(e)` → surfaced, not swallowed.
- [x] On `Err(e)` the handler emits a `tracing::warn!` naming target + topic + error, AND returns
      `ErrorResponse::internal_error` (−32603) — NOT `SESSION_NOT_FOUND` (the failure is loud + correctly classified).
- [x] New unit test: with the inbox dir pointed at an unwritable path, `event.emit_to` for a
      `file.init` payload to an unknown target returns `INTERNAL_ERROR`, not `SESSION_NOT_FOUND`.
      (`emit_to_file_event_surfaces_inbox_io_error_as_internal_error` — runtime_dir pointed at a
      regular file so `create_dir_all` fails with NotADirectory.)
- [x] Existing emit_to tests still pass (spooled-success + unknown-non-file-target → SESSION_NOT_FOUND unchanged).
- [x] `cargo test -p termlink-hub --lib` passes (439/439); `cargo check -p termlink-hub` clean.

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
cargo test -p termlink-hub --lib emit_to
cargo check -p termlink-hub

## Ready-to-apply fix (verified against router.rs — apply verbatim next session)

In `crates/termlink-hub/src/router.rs`, inside `handle_event_emit_to`'s `Err(_) => { … }` arm,
replace the `if let Ok(true) = crate::inbox::deposit(...)` block (and the trailing SESSION_NOT_FOUND
return stays as-is AFTER the match) with an explicit 3-arm match:

```rust
match crate::inbox::deposit(target, topic, &payload, from) {
    Ok(true) => {
        crate::channel::mirror_inbox_deposit(target, topic, &payload, from).await;
        return Response::success(id, json!({
            "ok": true, "spooled": true, "target": target,
            "message": format!("Target '{}' offline — file event spooled to inbox", target),
        })).into();
    }
    Ok(false) => { /* not a spoolable file event — fall through to SESSION_NOT_FOUND below */ }
    Err(e) => {
        tracing::warn!(target = target, topic = topic, error = %e,
            "event.emit_to: inbox spool FAILED (I/O error) — returning internal_error, not SESSION_NOT_FOUND");
        return ErrorResponse::internal_error(
            id, &format!("inbox spool failed for target '{target}' topic '{topic}': {e}")).into();
    }
}
// existing SESSION_NOT_FOUND return stays here (reached only on Ok(false))
```

Then add a unit test near `emit_to_unknown_target_returns_error` (router.rs ~3240): set the inbox
runtime dir (via `discovery::runtime_dir` env override) to an unwritable path, call
`handle_event_emit_to` with a `file.init` payload + `transfer_id` at an unknown target, assert the
response code is `INTERNAL_ERROR` (−32603), not `SESSION_NOT_FOUND`. Verify: `cargo test -p
termlink-hub --lib emit_to` + `cargo check -p termlink-hub`, then tick ACs + `fw task update
T-2488 --status work-completed`. `ErrorResponse::internal_error` confirmed to exist at
termlink-protocol/src/jsonrpc.rs:137.

## RCA

**Symptom:** A client calling `event.emit_to` with a `file.*` topic at an offline target receives
`SESSION_NOT_FOUND` even when the target session genuinely exists offline and the true failure was
a disk error (full/read-only/permission) while spooling to the inbox. The file event is lost and
nothing is logged — an operator debugging "target not found" has no clue the disk failed.

**Root cause:** `router.rs:369` used `if let Ok(true) = crate::inbox::deposit(...)`, which pattern-
matches only the success case and collapses the two remaining, semantically-distinct outcomes —
`Ok(false)` (not a spoolable file event → correctly "not found") and `Err(e)` (a real I/O failure)
— into one fall-through returning `SESSION_NOT_FOUND`. The `Err` value was dropped with no logging.

**Why structurally allowed:** `if let Ok(true)` is a lossy match — it silently discards `Err`. The
handler treated a three-outcome `io::Result<bool>` as a boolean, so the type system's error channel
was thrown away at the call site. No test exercised the deposit-I/O-error path (only success and
not-a-file-topic), so the misclassification was invisible.

**Prevention:** the explicit `match` forces all three arms to be handled (a future refactor can't
silently drop `Err` again without an unhandled-arm), and the new unit test (unwritable inbox dir →
`INTERNAL_ERROR`, not `SESSION_NOT_FOUND`) locks the loud-failure contract.

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

### 2026-08-02T07:30:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2488-eventemitto-mislabels-inbox-spool-io-fai.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-6ca0ac00
- **Timestamp:** 2026-08-02T08:17:16Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T08:17:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
