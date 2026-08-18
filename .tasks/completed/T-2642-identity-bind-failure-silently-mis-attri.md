---
id: T-2642
name: "identity-bind failure silently mis-attributes posts under --json/--quiet"
description: >
  session.rs bind_per_agent_identity_default suppresses the bind-failure warning unless
  verbose; under --json/--quiet a failed per-agent identity bind silently falls back
  to the shared host default and the operator's posts are mis-attributed with no signal
  (Directive #2 no-silent-failures + #3 actionable). Round-8 Usability sweep, verified
  in code.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/session.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T14:48:21Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-12T14:54:01Z
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
  - ts: '2026-08-18T18:56:55Z'
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
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2642: identity-bind failure silently mis-attributes posts under --json/--quiet

## Context

Round-8 Usability sweep (Directive #3), silent-degradation class. In
`crates/termlink-cli/src/commands/session.rs`, `bind_per_agent_identity_default`
binds a per-agent identity before `Session::register`. On the `Err` branch it
falls back to the shared host default identity — but the warning is gated behind
`if verbose` (`verbose = !json && !quiet`). So under `--json` or `--quiet`, a
failed bind is completely silent: the operator's posts get attributed to the
shared host identity with no signal. This violates Directive #2 (no silent
failures) and #3 (actionable errors). The failure warning goes to `eprintln!`
(stderr), so emitting it unconditionally does NOT corrupt `--json` stdout. The
success info line uses `println!` (stdout) and MUST stay `verbose`-gated (it
would corrupt `--json`). Verified in code this session.

## Acceptance Criteria

### Agent
- [x] A pure helper `bind_identity_messages(agent_id, outcome, key_path_display, verbose) -> (Option<String> stdout_info, Option<String> stderr_warn)` is extracted, so the print decision is unit-testable.
- [x] Bind FAILURE warning is returned unconditionally (stderr_warn is `Some` regardless of `verbose`); bind SUCCESS info is `Some` only when `verbose` (stays stdout-safe under `--json`).
- [x] The failure warning carries actionable remediation (how to bind correctly / what mis-attribution means), not just the raw error.
- [x] Load-bearing unit tests: Err→stderr_warn `Some` under BOTH verbose=true and verbose=false; Ok→stdout_info `Some` only when verbose (None when not); warning contains the remediation token.
- [x] Load-bearing proof recorded: temp-revert (re-gate the warning behind `verbose`) makes the verbose=false assertion FAIL; fix restores green.
- [x] `cargo test -p termlink --bins` green for the new tests; `cargo build -p termlink` clean.

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

out=$(cargo test -p termlink --bins commands::session::tests::bind_identity 2>&1); echo "$out" | grep -qE "test result: ok"
cargo build -p termlink 2>&1 >/dev/null

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

**Symptom:** Under `--json` or `--quiet`, a failed per-agent identity bind
silently falls back to the shared host default identity. The operator's
subsequent posts are attributed to the wrong (host) identity, and NOTHING is
printed — the operator has no way to know their identity binding failed.

**Root cause:** The `Err` branch's warning `eprintln!` is wrapped in
`if verbose { ... }`, where `verbose = !json && !quiet`. The gate was applied
symmetrically to both the success info (stdout `println!`, which legitimately
must be suppressed under `--json`) and the failure warning (stderr `eprintln!`,
which does NOT corrupt stdout and represents a real fault). A fault warning was
conflated with routine info and silenced by output-formatting flags.

**Why structurally allowed:** No test asserted the failure-path emits a signal
independent of verbosity. The print decision was inline control-flow inside a
side-effecting fn, so it was untestable without refactor — the exact shape that
lets a "no silent failures" (Directive #2) violation hide.

**Prevention:** Extract the print decision into the pure helper
`bind_identity_messages` and unit-test that the failure warning is `Some`
regardless of `verbose`. The load-bearing test fails if anyone re-gates the
warning. General learning: stderr warnings for FAULTS must not be gated by the
same flag that gates stdout INFO — the flag governs output format, not fault
visibility.

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

### 2026-08-12T14:48:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2642-identity-bind-failure-silently-mis-attri.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1c6cbf87
- **Timestamp:** 2026-08-12T14:55:15Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 2
     - evidence: `cargo build -p termlink 2>&1 >/dev/null`

### 2026-08-12T14:54:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
