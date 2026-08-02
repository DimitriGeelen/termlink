---
id: T-2496
name: "aggregator long-poll hot-loops on in-band RPC error (no backoff)"
description: >
  crates/termlink-hub/src/aggregator.rs (~:96-118) long-poll loop does 'if let Ok(data) = client::unwrap_result(resp)' — silently drops the Err (in-band JSON-RPC error) branch with NO sleep/backoff. The transport-error branch (Ok(Err(_))) sleeps 2s, but a session that responds with a JSON-RPC ERROR is re-polled in a tight hot loop (CPU spin + hammering the failing session). Fix: treat the dropped Err like the transport-error branch (log + same backoff). directive-#2 (observable, no silent tight-loop). Runner-up from firing-#15 adversarial audit (round 2). Verify in code before building (confirm the two branches + that Err currently has no sleep).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/aggregator.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T09:57:22Z
last_update: 2026-08-02T10:19:11Z
date_finished: 2026-08-02T10:19:11Z
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

# T-2496: aggregator long-poll hot-loops on in-band RPC error (no backoff)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

The hub event aggregator (`crates/termlink-hub/src/aggregator.rs`) spawns a per-session
long-poll loop over `event.subscribe`. On a Success transport response carrying an in-band
JSON-RPC **error**, `if let Ok(data) = client::unwrap_result(resp)` dropped the `Err` with
no log and NO backoff — and an RPC error returns immediately (vs the 5s server-side
long-poll on success), so the loop hot-spins, hammering the failing session. The transport-
error arm (`Ok(Err(e))`) already sleeps 2s; the in-band-error path had no equivalent.

## Acceptance Criteria

### Agent
- [x] The `Ok(Ok(resp))` arm handles the `unwrap_result` `Err` (in-band RPC error) case — logs it and sleeps the same backoff as the transport-error arm (no silent hot-loop)
- [x] The backoff decision is factored into a pure `poll_action(&PollOutcome) -> PollAction` that the loop actually dispatches on (single source of truth), so the rule is unit-testable without a live hub
- [x] Unit tests assert `poll_action` returns `Backoff` for BOTH `RpcError` and `Transport`, `Deliver` for `Success`, `IdleRetry` for the outer-timeout (idle) case
- [x] Event delivery + cursor advance on the success path are unchanged (no behavior change for the healthy case)

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

cargo test -p termlink-hub --lib aggregator
cargo check -p termlink-hub

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

**Symptom:** A session whose hub answers `event.subscribe` with a JSON-RPC error (auth
denied, bad params, session in a bad state) is re-polled in a tight CPU-spinning loop by
the aggregator, hammering the failing session, with no log line — invisible.

**Root cause:** the `Ok(Ok(resp))` (transport-OK) arm used `if let Ok(data) = unwrap_result`
and let the `Err` (in-band RPC error) fall through the `if let` with no `else` — no log, no
sleep. Unlike a success (where the server holds the long-poll ~5s), an RPC error returns
immediately, so the loop re-issues instantly → hot-loop. The symmetric transport-error arm
did sleep 2s; the two failure modes were treated asymmetrically.

**Why structurally allowed:** `if let Ok(x) = fallible()` with no `else` is the classic
silent-drop shape (sibling of PL-276) — it reads as "process on success" and hides the
error branch entirely. No test drove the in-band-error path (the loop had no tests and no
seam).

**Prevention:** the backoff decision is now a pure `poll_action(&PollOutcome)` the loop
dispatches on, with unit tests pinning that BOTH failure kinds map to `Backoff`. A future
change that re-drops the RPC-error branch fails the suite. Learning: treat transport and
in-band RPC errors symmetrically in any retry loop — both are failures that need backoff.

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

### 2026-08-02T09:57:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2496-aggregator-long-poll-hot-loops-on-in-ban.md
- **Context:** Initial task creation

### 2026-08-02T10:17:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-947887da
- **Timestamp:** 2026-08-02T10:19:18Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T10:19:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
