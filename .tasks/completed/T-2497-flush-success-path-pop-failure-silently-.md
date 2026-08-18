---
id: T-2497
name: "flush success-path pop failure silently re-POSTs queued message — unbounded
  busy-loop plus durable duplication"
description: >
  flush success-path pop failure silently re-POSTs queued message — unbounded busy-loop
  plus durable duplication

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/bus_client.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T11:22:18Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-08-02T11:25:31Z
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

# T-2497: flush success-path pop failure silently re-POSTs queued message — unbounded busy-loop plus durable duplication

## Context

`BusClient::flush` (`crates/termlink-session/src/bus_client.rs`) drains the durable
offline queue. On the success path — hub accepted the POST, `parse_post_response`
returned `Ok` — the loop ran `let _ = self.queue.pop(id);` (line 208), discarding the
result of the local SQLite delete. If that delete fails (disk-full / IO / lock), the
head row is never removed: the loop `continue`s, `peek_oldest_with_attempts` returns
the SAME row, and it is re-POSTed on every iteration — an unbounded busy-loop that
duplicates the message on the durable topic (for posts without a `client_msg_id`;
T-2049 dedupe absorbs the re-post otherwise, but the hot-spin remains). This is the
exact failure class the sibling T-2452 fallback-pop guard (60 lines below, same fn)
already breaks on — the success-path instance was simply missed. Directive-#2 breach
(silent failure + durable-message duplication).

## Acceptance Criteria

### Agent
- [x] Success-path pop failure surfaces LOUD (`tracing::error!` with `queue_id` + error) instead of a bare `let _ =`
- [x] On success-path pop failure the flush pass `break`s (yields) rather than re-POSTing the undeleted head row unboundedly — mirrors the T-2452 fallback-pop guard
- [x] The "pop-failed ⇒ abort the pass, pop-ok ⇒ continue" decision is factored into a pure classifier (mirrors T-2496 `poll_action`) so the rule is unit-testable without a live hub or a queue fault-injection seam
- [x] Regression tests assert the classifier aborts on pop-error and continues on pop-success
- [x] `cargo test -p termlink-session --lib bus_client` passes (existing + new)

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
cargo test -p termlink-session --lib bus_client 2>&1 | tail -5 | grep -q "test result: ok"

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

**Symptom:** Under disk pressure (the exact degraded condition the offline queue
exists to survive), a successfully-delivered queued POST whose local SQLite delete
fails causes `flush()` to re-POST the same head row on every loop iteration — an
unbounded hub-hammering busy-loop that duplicates the message on the durable topic
(no `client_msg_id` ⇒ genuine duplicate append; with one ⇒ T-2049 dedupe absorbs it
but the hot-spin still burns CPU + hub RPCs). Entirely silent — no log, false
`report.sent += 1`.

**Root cause:** The success arm at `bus_client.rs:208` used `let _ = self.queue.pop(id);`,
discarding the `Result`. Every OTHER fallible call in the same `flush()` loop was
hardened (transient-reject → break+log; poison → dead_letter+`error!`; T-2452
fallback-pop → break+`error!`; T-2439 bump_attempts → `warn!`) — the success-path
pop was the one instance left with a bare discard and no `break`, so a pop failure
leaves the row at head and the loop re-POSTs it forever.

**Why structurally allowed:** A bare `let _ = fallible()` reads as "fire-and-forget"
and is invisible to the type checker (the `Result` is explicitly discarded, not
`unwrap`ed). The sibling guards were added incident-by-incident (T-1439/T-2243/
T-2452/T-2439) as each failure mode was discovered; the success-path pop never had
its own incident, so it was never hardened. No lint flags `let _ =` on a
`Result`-returning call.

**Prevention:** The fix factors the load-bearing decision ("pop-failed ⇒ abort the
pass, don't re-POST") into a pure `pop_action` classifier (mirrors the T-2496
`poll_action` pattern), locked with unit tests — so a future edit that silently
reverts to "continue on pop failure" breaks a test. Captured as a learning in the
same no-silent-failures class as PL-276 / PL-282.

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

### 2026-08-02T11:22:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2497-flush-success-path-pop-failure-silently-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-3834ffb4
- **Timestamp:** 2026-08-02T11:25:33Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 31
     - evidence: `cargo test -p termlink-session --lib bus_client 2>&1 | tail -5 | grep -q "test result: ok"`

### 2026-08-02T11:25:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
