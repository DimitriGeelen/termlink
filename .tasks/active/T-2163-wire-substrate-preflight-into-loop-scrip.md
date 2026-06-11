---
id: T-2163
name: "Wire substrate-preflight into loop-script startup (T-2154 closure)"
description: >
  Wire substrate-preflight.sh into substrate-orchestrator-loop.sh and substrate-worker-loop.sh as a startup check. The two loop scripts are the load-bearing 'real' substrate-runtime entry points (every production worker/orchestrator wraps them). Today they will happily start with TERMLINK_RUNTIME_DIR=/tmp and wedge silently on the next reboot — PL-021. Add a preflight call at the top of each loop: on exit 2 (FAIL) refuse to start, on exit 1 (WARN) print and continue, on exit 0 (PASS) silent. Add --skip-preflight flag for CI/test paths where preflight is already known clean. Closes the substrate-arc safety arc: preflight CLI (T-2154) → /preflight skill (T-2158) → cron canary (T-2160) → loop-startup gate (this slice).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [substrate, preflight, T-2018, safety]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-11T14:00:47Z
last_update: 2026-06-11T14:00:57Z
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

# T-2163: Wire substrate-preflight into loop-script startup (T-2154 closure)

## Context

Both `scripts/substrate-orchestrator-loop.sh` (T-2148) and `scripts/substrate-worker-loop.sh` (T-2146) are the load-bearing entry points for production substrate use — every service-style worker/orchestrator wraps one of them. Today they will happily start with `TERMLINK_RUNTIME_DIR` on `/tmp` and wedge silently after the next reboot (PL-021). The substrate-arc safety set already has the read-side (T-2154 preflight CLI, T-2158 `/preflight` skill, T-2160 nightly cron canary) — what's missing is the *runtime-entry* gate. This task wires a single preflight call into each loop's startup banner: FAIL refuses to start, WARN prints and continues, PASS is silent. Adds `--skip-preflight` for CI/test paths.

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-orchestrator-loop.sh` calls `scripts/substrate-preflight.sh` before its first hub-touching operation
- [x] `scripts/substrate-worker-loop.sh` does the same
- [x] On preflight exit 2 (FAIL), both loops refuse to start: print the preflight output to stderr and exit 4 (new code, distinct from existing 2=usage / 3=claimer-unresolved / 11=claim-fail / 12=release-fail / 130=signal)
- [x] On preflight exit 1 (WARN), both loops print one stderr WARNING line plus the preflight body and continue
- [x] On preflight exit 0 (PASS), both loops are silent (no extra output)
- [x] Both loops accept `--skip-preflight` to bypass the check entirely (CI/test paths)
- [x] `--help` text on both loops mentions `--skip-preflight` and the new exit code 4
- [x] Live exec: `scripts/substrate-orchestrator-loop.sh --help` succeeds and shows `--skip-preflight`
- [x] Live exec: `scripts/substrate-worker-loop.sh --help` succeeds and shows `--skip-preflight`
- [x] Live exec: `TERMLINK_RUNTIME_DIR=/tmp/preflight-loop-test scripts/substrate-worker-loop.sh --topic smoke --offset 0 --cmd 'true'` exits 4 with a `[FAIL] runtime_dir` line on stderr (no hub contact attempted)
- [x] Live exec: `scripts/substrate-worker-loop.sh --skip-preflight ...` does NOT call preflight (verified via missing preflight output in stderr)

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

bash -n scripts/substrate-orchestrator-loop.sh
bash -n scripts/substrate-worker-loop.sh
scripts/substrate-orchestrator-loop.sh --help 2>&1 | grep -q -- "--skip-preflight"
scripts/substrate-worker-loop.sh --help 2>&1 | grep -q -- "--skip-preflight"
out=$(TERMLINK_RUNTIME_DIR=/tmp/preflight-loop-test scripts/substrate-worker-loop.sh --topic smoke --offset 0 --cmd 'true' 2>&1); ec=$?; [ "$ec" -eq 4 ] && echo "$out" | grep -q "FAIL"
out=$(TERMLINK_RUNTIME_DIR=/tmp/preflight-loop-test scripts/substrate-orchestrator-loop.sh --work-topic smoke 2>&1); ec=$?; [ "$ec" -eq 4 ] && echo "$out" | grep -q "FAIL"

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

### 2026-06-11T14:00:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2163-wire-substrate-preflight-into-loop-scrip.md
- **Context:** Initial task creation

### 2026-06-11T14:00:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
