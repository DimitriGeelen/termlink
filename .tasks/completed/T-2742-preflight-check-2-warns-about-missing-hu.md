---
id: T-2742
name: "Preflight Check 2 warns about missing hubs.toml on a correctly-configured local-only
  install"
description: >
  Preflight Check 2 warns about missing hubs.toml on a correctly-configured local-only
  install

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [scripts/substrate-preflight.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T14:47:22Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-15T19:12:20Z
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
  - ts: '2026-08-18T18:56:58Z'
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
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2742: Preflight Check 2 warns about missing hubs.toml on a correctly-configured local-only install

## Context

Herdr adoption backlog rank 15 (worker 5, F4). `config.rs:112-128` treats a
missing `hubs.toml` as a normal empty config, and the local hub is reached at
`runtime_dir()/hub.sock` with no profile at all. A purely-local install
therefore never needs the file — yet preflight Check 2
(`substrate-preflight.sh:428-435`) WARNs whenever it is absent, telling a
correctly-configured operator on first run that something is wrong.

That is the PL-219 class this repo already names: a guard that fires on a
healthy state teaches its operator to stop reading it, which costs more than the
warning was ever worth. The fix is to make the check conditional on whether a
profile is actually needed, not to delete it — an operator who genuinely has no
local hub AND no profiles still has nothing to talk to.

## Acceptance Criteria

### Agent
> **LOAD-BEARING PROOF (2026-08-15, recorded after the budget gate cleared).**
> Replacing the new conditional's test with `if false;` — which reproduces the
> pre-T-2742 behaviour exactly, since the socket branch becomes unreachable and
> every missing-hubs.toml run falls through to the warn — makes fixture 1 fail
> **2 assertions** (`.status` is `warn` not `pass`; the message carries the
> no-hub-to-talk-to text instead of the by-design text). Fixtures 2, 3 and 4
> stay green throughout, which is the shape that matters: the narrowing touched
> only the local-only branch, so the controls must not move. Restoring returns
> the suite to **10/10** and the tree to a zero-diff state
> (`git diff scripts/` empty, `TEMP-REVERT` count 0).

- [x] Check 2 distinguishes "no hubs.toml AND a local hub socket exists" (a valid local-only install) from "no hubs.toml AND no local hub" (genuinely nothing to talk to)
- [x] The local-only case emits `pass`, not `warn`, and its message states that fleet verbs are empty *by design* so the operator is not left wondering
- [x] The no-hub-at-all case still warns and still names `termlink fleet profile add` — the guard is narrowed, not removed
- [x] The socket path is resolved with the script's existing `resolve_runtime_dir`, not a hardcoded `/tmp` path (the T-2729 trap)
- [x] The "present but no [hubs.NAME] sections" branch keeps its existing behaviour
- [x] Fixture coverage for all three states (local-only pass, no-hub warn, populated pass), driven by a test seam rather than the real `$HOME`
- [x] Load-bearing proof recorded: reverting the conditional makes the local-only fixture fail
- [x] `bash -n scripts/substrate-preflight.sh` parses, the preflight fixture suites pass, and `scripts/run-guard-layer.sh` is clean

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

bash -n scripts/substrate-preflight.sh
bash tests/substrate-preflight-hubs-toml-fixtures.sh
bash tests/substrate-preflight-runtime-dir-fixtures.sh
bash scripts/run-guard-layer.sh

## RCA

**Symptom:** On a host with a working local hub and no `~/.termlink/hubs.toml`,
`/preflight` Check 2 reported `warn` — "hubs.toml missing" — on every single
run, even though nothing was wrong and no operator action would have helped.

**Root cause:** The check treated the presence of `hubs.toml` as the question,
when the question that matters is whether there is any hub to talk to.
`config.rs:112-128` parses an absent file as an empty config, and the local hub
is reached at `runtime_dir()/hub.sock` with no profile involved. A
purely-local install therefore never needs the file, and the check had no way
to express that.

**Why structurally allowed:** the check was written from the fleet operator's
vantage point, where a profile is always needed, and the local-only install was
simply never a case anyone enumerated. Nothing in the guard layer asks whether
a check can fire on a healthy state — the fixture suites that exist prove
checks *fire*, and a check that over-fires passes those just as well as a
correct one. That is the same asymmetry PL-219 names.

**Prevention:** `tests/substrate-preflight-hubs-toml-fixtures.sh` pins all four
states, and fixture 1 is specifically the healthy-state-must-not-warn case —
the assertion class that was missing. It runs off the
`TERMLINK_PREFLIGHT_HUBS_TOML` seam and a scratch `TERMLINK_RUNTIME_DIR` with a
real AF_UNIX socket, so it is hermetic and cannot pass by accident on a host
that happens to be configured a particular way.

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

### 2026-08-15T14:47:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2742-preflight-check-2-warns-about-missing-hu.md
- **Context:** Initial task creation

### 2026-08-15T19:12:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
