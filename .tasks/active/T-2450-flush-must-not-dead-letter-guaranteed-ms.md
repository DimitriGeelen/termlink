---
id: T-2450
name: "flush must not dead-letter guaranteed msg on transient RATE_LIMITED — classify transient vs poison (round-11 F1)"
description: >
  flush must not dead-letter guaranteed msg on transient RATE_LIMITED — classify transient vs poison (round-11 F1)

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
created: 2026-07-22T06:01:23Z
last_update: 2026-07-22T06:01:23Z
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

# T-2450: flush must not dead-letter guaranteed msg on transient RATE_LIMITED — classify transient vs poison (round-11 F1)

## Context

Round-11 durable-delivery review Finding 1. `BusClient::flush`
(`bus_client.rs` ~194-256) treats ANY hub-reject identically: each reject bumps
`attempts`, and after `POISON_THRESHOLD` (10) passes the entry is dead-lettered
out of auto-delivery. But `parse_post_response` collapses all JSON-RPC errors
into `HubError(String)`, so a **transient** `RATE_LIMITED` (-32008) or
`HUB_AT_CAPACITY` (-32019) — the exact fleet-wide-hub-bounce backpressure the
T-2055 jitter targets — is indistinguishable from a permanent poison (unknown
topic, bad signature). A hub bounce returning RATE_LIMITED for >~50s pulls a
**durably-queued guaranteed message** out of the delivery path into
`dead_letters`, requiring manual recovery. This violates termlink's core
"write is guaranteed to be delivered" reliability contract.

Fix: make `HubError` carry the JSON-RPC `code`; add a pure
`is_transient_hub_reject` classifier; in `flush`, a transient reject `break`s
(preserves FIFO, retries next flush) WITHOUT bumping `attempts` or counting
toward poison. Permanent rejects keep the existing dead-letter behavior.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `BusClientError::HubError` carries `{ code: i64, message: String }`; `parse_post_response` populates the code from the JSON-RPC error.
- [x] Pure `is_transient_hub_reject(&BusClientError) -> bool` returns true for `RATE_LIMITED` (-32008) and `HUB_AT_CAPACITY` (-32019), false for permanent codes and non-HubError variants.
- [x] `flush` reject arm: a transient reject `break`s WITHOUT bumping `attempts`/dead-lettering (guaranteed msg stays queued); permanent rejects retain existing poison-threshold dead-letter behavior.
- [x] Unit test for the classifier (both transient codes true, a permanent code + a non-HubError false); existing `flush_poison_dead_letters_instead_of_silent_drop` (-32601 permanent) still green. `cargo test -p termlink-session --lib` green (391 passed, +1 new); CLI+MCP build clean.

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

cargo test -p termlink-session --lib is_transient_hub_reject
cargo test -p termlink-session --lib flush_poison_dead_letters_instead_of_silent_drop
cargo test -p termlink-session --lib

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

**Symptom:** During a fleet-wide hub bounce, a durably-queued *guaranteed*
message returning `RATE_LIMITED`/`HUB_AT_CAPACITY` for >~50s (10 flush passes)
was dead-lettered out of the auto-delivery path, needing manual `queue-status`
recovery — a guaranteed write silently stopped being delivered.

**Root cause:** `parse_post_response` collapsed every JSON-RPC error into
`HubError(String)`, erasing the code. `flush`'s poison logic then treated
transient backpressure identically to permanent poison, bumping `attempts` on
each rate-limit and crossing `POISON_THRESHOLD`.

**Why structurally allowed:** The poison mechanism (T-1439/T-2243) was designed
for permanent rejects (unknown topic, bad signature) and never distinguished the
transient class, because the error code was thrown away at the parse boundary
before `flush` could see it. No test exercised a transient reject against the
poison counter — the existing poison test uses -32601 (permanent), so the gap
was invisible.

**Prevention:** The error code now survives to `flush`; `is_transient_hub_reject`
is unit-tested for both transient codes AND a permanent code, so a future change
that re-collapses the code or mis-classifies a backpressure reject fails a test
rather than silently re-opening the dead-letter-on-bounce window.

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

### 2026-07-22T06:01:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2450-flush-must-not-dead-letter-guaranteed-ms.md
- **Context:** Initial task creation
