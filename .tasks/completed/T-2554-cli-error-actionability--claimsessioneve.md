---
id: T-2554
name: "CLI error actionability — claim/session/event paths name the fix"
description: >
  Usability lens (Directive #3): bare CLI errors on channel claim/renew/release, exec/interact
  session-not-found, and event emit_to hub-down describe the failure but not the recovery.
  Add per-path actionable hints.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, 
      crates/termlink-cli/src/commands/events.rs, 
      crates/termlink-cli/src/commands/session.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-08T20:30:55Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-08T20:37:01Z
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
  - ts: '2026-08-18T18:56:51Z'
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
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2554: CLI error actionability — claim/session/event paths name the fix

## Context

Usability lens (Constitutional Directive #3, T-2468 purpose-review campaign).
Sibling of T-2553 (which fixed the MCP hub-down path). These are the CLI-side
findings #2–#5 from the usability hunter — user-typed commands whose errors
describe the failure but not the recovery, missing the project's own
"suggest the next command" bar:

- **#2** `channel claim/renew/release` leak raw `ClaimError` (channel.rs ~10887,
  10928, 10964): "offset X is already claimed" with no "run `termlink channel
  claims <topic>`" pointer.
- **#3** `exec`/`interact` session-not-found (session.rs ~927, 1015): no hint
  that `termlink list-sessions` shows valid IDs.
- **#4** "Failed to connect to session" (session.rs ~949, 1028): a stale/dead
  PTY registration, no cue to run `termlink clean`.
- **#5** `event emit_to` hub-connect failure (events.rs ~487) lacks the "Start
  it with: termlink hub start" hint its dispatch.rs sibling already carries.

Finding #6 (generic `.context("Hub rpc_call failed")` classification) is
LOWER-traffic and needs a small design choice (per-call vs classify in
`rpc_call_authed`) — deferred to a separate task, not built here (keeps this
one scoped to the mechanical per-site hints).

## Acceptance Criteria

### Agent
- [x] #2: `channel claim` conflict error names `termlink channel claims <topic>` (see who holds / pick another offset); `renew`/`release` NotFound/Expired/NotOwned name the recovery (`termlink channel claim` to re-claim, or `termlink channel claims` to see the holder)
- [x] #3: `exec`/`interact` session-not-found error names `termlink list-sessions`
- [x] #4: "Failed to connect to session" error names `termlink clean` (stale registration) and/or `termlink list-sessions`
- [x] #5: `event emit_to` hub-connect failure names "termlink hub start" (parity with dispatch.rs)
- [x] `cargo build -p termlink` succeeds; a unit test covers at least the #2 claim-conflict message actionability (load-bearing — reverting the hint fails it)

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
out=$(cargo test -p termlink -- claim_conflict_error_is_actionable claim_notfound_error_names_reacquire 2>&1); echo "$out" | grep -q "test result: ok. 2 passed"
grep -q "termlink list-sessions" crates/termlink-cli/src/commands/session.rs
grep -q "termlink hub start" crates/termlink-cli/src/commands/events.rs

## RCA

**Symptom:** User-typed CLI commands on high-traffic paths returned errors that
described the failure but not the recovery: `channel claim` conflict ("offset 5
is already claimed") with no pointer to `termlink channel claims`; `exec`/
`interact` session-not-found with no `termlink list-sessions` hint; "Failed to
connect to session" (stale PTY registration) with no `termlink clean` cue;
`event emit_to` hub-down missing the "termlink hub start" hint its dispatch.rs
sibling already carried.

**Root cause:** Same class as T-2553 — error-message actionability is a
convention, not enforced. The claim paths wrapped the raw typed `ClaimError`
verbatim (`anyhow!("channel.claim failed: {e}")`); the session/event paths used
bare `.context()` strings. Each was written to describe, not direct, and the
"suggest the next command" bar that exists elsewhere (dispatch.rs, fleet doctor,
the /claim skill) never reached these surfaces.

**Why structurally allowed:** No gate asserts a caller-facing error names a
recovery action, so descriptive-but-non-directive messages ship silently. The
inconsistency (emit_to vs its dispatch.rs sibling) shows the bar is applied
case-by-case rather than structurally.

**Prevention:** The claim family now routes through ONE helper
(`claim_err_actionable`) so future claim call sites inherit the actionable form,
and two load-bearing unit tests (`claim_conflict_error_is_actionable`,
`claim_notfound_error_names_reacquire`) assert the recovery command is present —
reverting the Conflict hint to `String::new()` fails the test (proven this
session). The session/event hints are per-site strings verified by grep in the
Verification block. A broader "errors must name a fix" lint across all
caller-facing paths remains a candidate for a future Level-C tooling task
(noted alongside finding #6, deferred).

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

### 2026-08-08T20:30:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2554-cli-error-actionability--claimsessioneve.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-efc10cd5
- **Timestamp:** 2026-08-08T20:37:33Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-08T20:37:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
