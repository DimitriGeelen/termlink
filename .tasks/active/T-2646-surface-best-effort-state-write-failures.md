---
id: T-2646
name: "surface best-effort state-write failures in remote.rs fleet-doctor (escalation defeat + dup learnings)"
description: >
  surface best-effort state-write failures in remote.rs fleet-doctor (escalation defeat + dup learnings)

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
created: 2026-08-12T18:25:46Z
last_update: 2026-08-12T18:25:46Z
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

# T-2646: surface best-effort state-write failures in remote.rs fleet-doctor (escalation defeat + dup learnings)

## Context

Round-9 Usability/Reliability sweep (Directive #3 / #2 "no silent failures"),
silent-degradation class, verified in code. Two `let _ = std::fs::write(...)`
best-effort writes in `remote.rs` fleet-doctor machinery discard a failure that
has real operational consequences:

- **F1 (high):** `save_fleet_state` (remote.rs:6752, sole caller at :6837 inside
  `maybe_track_fleet_failure`) persists the per-hub `consecutive_failures`
  counter across runs. If the write silently fails (read-only `.context`, disk
  full, perms), the counter never accumulates → never reaches
  `FLEET_CONCERN_THRESHOLD` → the T-1053 concern-escalation for a chronically
  failing hub is **never registered**, with zero operator signal. The silent
  write defeats the entire escalation mechanism.
- **F5 (lower):** the learnings dedupe-marker refresh (remote.rs:6645) is
  `let _ = std::fs::write(&marker, ...)`. If it fails, the same fleet
  observation is re-appended as a **duplicate learning** next run — again silent.

Both are the same root cause: a fire-and-forget best-effort write whose failure
matters, with the `Result` dropped. Fix: route each write's outcome through a
pure decision helper that returns `Some(warning)` on failure (naming the
consequence + the fix), print to stderr — preserving the "never fail the caller"
best-effort contract while killing the silent part. Mirrors the T-2642
`bind_identity_messages` load-bearing pattern.

## Acceptance Criteria

### Agent
- [ ] `save_fleet_state` no longer uses `let _ =` on the state write — its outcome routes through a pure helper `fleet_state_save_warning(path, Result<(),String>) -> Option<String>` that returns `Some(..)` on failure, printed to stderr; the caller contract (`maybe_track_fleet_failure` still returns `Ok(())`, never fails) is unchanged.
- [ ] The dedupe-marker refresh (remote.rs:6645) no longer uses `let _ =` — its outcome routes through a pure helper `dedupe_marker_save_warning(path, Result<(),String>) -> Option<String>` returning `Some(..)` on failure, printed to stderr.
- [ ] Each helper's failure warning names the CONSEQUENCE (F1: concern escalation won't trigger; F5: duplicate learnings) AND a fix hint — actionable per PL-151.
- [ ] Load-bearing unit tests: `fleet_state_save_warning(Err(..))` is `Some` and contains the consequence token; `Ok(())` is `None`. Same for the dedupe helper. Proven load-bearing via temp-revert (Ok arm returning Some → test fails).
- [ ] `cargo build -p termlink` clean; `cargo test -p termlink --bins` for the new tests passes.

### Human (REMOVED — all criteria agent-verifiable)
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
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
cargo build -p termlink 2>&1 | tail -3
cargo test -p termlink --bins fleet_state_save_warning 2>&1 | tail -5
cargo test -p termlink --bins dedupe_marker_save_warning 2>&1 | tail -5
# No remaining silent `let _ = std::fs::write` on the two hardened sites:
test "$(grep -c 'let _ = std::fs::write(path, s)' crates/termlink-cli/src/commands/remote.rs)" = "0"
test "$(grep -c 'let _ = std::fs::write(&marker' crates/termlink-cli/src/commands/remote.rs)" = "0"

## RCA

**Symptom:** A chronically failing hub can silently escape the T-1053
concern-escalation mechanism forever; separately, the same fleet observation can
be re-appended as a duplicate learning — both with zero operator signal.

**Root cause:** Two best-effort writes in remote.rs fleet-doctor machinery use
`let _ = std::fs::write(...)`, discarding the `io::Result`. `save_fleet_state`
persists `consecutive_failures` across runs; if the write fails the counter never
accumulates and never reaches `FLEET_CONCERN_THRESHOLD`. The dedupe-marker write
gates duplicate-learning suppression; if it fails the dedupe is silently broken.

**Why structurally allowed:** The "best-effort, never fail the caller" contract
(legitimately no-ops outside framework projects with no `.context/`) was
conflated with "safe to ignore the error entirely". No convention distinguished
"skip cleanly when there is nothing to write" from "the write was attempted and
failed" — the latter is a real degradation that must surface. Same class as
PL-307 (Directive #2 no-silent-failures is convention-not-enforced for
best-effort side-effect writes) and PL-283.

**Prevention:** Route each write's outcome through a pure decision helper
(`Result<(),String> -> Option<String>`) returning `Some(warning)` on failure;
unit-test the decision. The helper is the tested seam so a future revert to
`let _ =` is caught by the Verification grep. General learning: best-effort /
fire-and-forget is about not FAILING the caller, not about ignoring the error —
observe it (throttled warning) even when you proceed.

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

### 2026-08-12T18:25:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2646-surface-best-effort-state-write-failures.md
- **Context:** Initial task creation
