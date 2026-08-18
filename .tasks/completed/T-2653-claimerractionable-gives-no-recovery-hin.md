---
id: T-2653
name: "claim_err_actionable gives no recovery hint for Hub/Transport ClaimError variants
  (RATE_LIMITED/AT_CAPACITY/auth/unreachable dead-end)"
description: >
  claim_err_actionable gives no recovery hint for Hub/Transport ClaimError variants
  (RATE_LIMITED/AT_CAPACITY/auth/unreachable dead-end)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/channel.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T19:44:27Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-12T19:48:39Z
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

# T-2653: claim_err_actionable gives no recovery hint for Hub/Transport ClaimError variants (RATE_LIMITED/AT_CAPACITY/auth/unreachable dead-end)

## Context

Round-11 usability hunt (verified in code). `claim_err_actionable`
(`crates/termlink-cli/src/commands/channel.rs:10922`, T-2554, Directive #3
"name the recovery command per variant") appends a next-step hint for four
`ClaimError` variants (Conflict/NotFound/Expired/NotOwned) but its
`_ => String::new()` catch-all gives **no hint** for `ClaimError::Hub{code}`
— which is exactly where RATE_LIMITED (-32008), HUB_AT_CAPACITY (-32019),
AUTH_REQUIRED (-32009), and AUTH_DENIED (-32010) land — nor for
`ClaimError::Transport` (hub unreachable). The CLAUDE.md `/claim` loud-refusal
taxonomy already documents the intended hints (RATE_LIMITED→governor,
HUB_AT_CAPACITY→governor, AUTH→reauth+doctor) but the code catch-all silently
drops them. Sibling proof: the four named arms at channel.rs:10928-10942 each
append a `run \`termlink ...\`` recovery command. PL-306: caller-facing error
actionability is a convention, not enforced — this is a live instance.

Clean single-file pure-helper fix (same shape as T-2652/T-2554): route the
code→hint mapping through a new pure `claim_hub_code_hint(code) -> String` so
it is unit-testable, and add a `Transport` arm.

## Acceptance Criteria

### Agent
- [x] `claim_err_actionable` gains a `ClaimError::Hub { code, .. }` arm dispatching on the named `error_code` constants: RATE_LIMITED / HUB_AT_CAPACITY → `termlink fleet governor-status`; AUTH_REQUIRED / AUTH_DENIED → `termlink fleet reauth` + `fleet doctor`; other codes → a generic "hub rejected the claim" hint
- [x] `claim_err_actionable` gains a `ClaimError::Transport(_)` arm hinting `termlink fleet doctor` (hub unreachable)
- [x] The code→hint mapping is a pure helper `claim_hub_code_hint(code: i64) -> String` (unit-testable, no I/O)
- [x] Unit test proves RATE_LIMITED and HUB_AT_CAPACITY map to a hint naming `governor-status`, auth codes map to a hint naming `reauth`, and an unknown code maps to a non-empty generic hint (load-bearing: reverting the arm to `String::new()` makes the test fail)
- [x] `cargo build -p termlink` + `cargo test -p termlink --bins claim_hub_code_hint` pass

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
out=$(cd /opt/termlink && grep -n "fn claim_hub_code_hint" crates/termlink-cli/src/commands/channel.rs); echo "$out" | grep -q "claim_hub_code_hint"
out=$(cd /opt/termlink && grep -c "ClaimError::Hub { code" crates/termlink-cli/src/commands/channel.rs); [ "$out" -ge 1 ]
cd /opt/termlink && cargo test -p termlink --bins claim_hub_code_hint 2>&1 | grep -q "test result: ok"

## RCA

**Symptom:** `termlink channel claim <topic> <offset>` (and renew/release) that
fails with a hub-level backpressure or auth error — RATE_LIMITED (-32008),
HUB_AT_CAPACITY (-32019), AUTH_REQUIRED/DENIED — prints only
`channel.claim failed: hub error: code=-32008 message=RATE_LIMITED` with NO
next step, while a claim-Conflict on the same command names the recovery
command. An unreachable hub (Transport) is equally mute.

**Root cause:** `claim_err_actionable`'s `match` handles four named
`ClaimError` variants but routes `Hub`, `Transport`, and `Protocol` through a
`_ => String::new()` catch-all — so the exact error classes an operator most
needs a recovery command for (backpressure, auth-drift, hub-down) get the
empty hint. The intended hints are already documented in the CLAUDE.md `/claim`
loud-refusal taxonomy but were never wired into this helper.

**Why structurally allowed:** caller-facing error actionability (Directive #3)
is a convention, not enforced (PL-306, T-2555). A wildcard match arm silently
absorbs new/other variants with no compiler warning — the helper compiles and
"works", it just under-serves. No test pinned the Hub/Transport hint.

**Prevention:** the code→hint mapping is extracted to a pure
`claim_hub_code_hint(code)` with a unit test pinning RATE_LIMITED/AT_CAPACITY →
governor, auth → reauth, unknown → generic. The test is load-bearing (revert
the arm to `String::new()` → test fails), so a future refactor that drops the
hint is caught. Broader enforcement of PL-306 across all `_ => String::new()`
error-hint catch-alls is a separate static-check candidate (noted, not built
here — one-bug-one-task).

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

### 2026-08-12T19:44:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2653-claimerractionable-gives-no-recovery-hin.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-8f95baa1
- **Timestamp:** 2026-08-12T19:49:12Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 33
     - evidence: `cd /opt/termlink && cargo test -p termlink --bins claim_hub_code_hint 2>&1 | grep -q "test result: ok"`

### 2026-08-12T19:48:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
