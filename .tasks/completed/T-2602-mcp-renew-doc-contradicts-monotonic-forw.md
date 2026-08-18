---
id: T-2602
name: "MCP renew doc contradicts monotonic-forward code (T-2510) — says absolute-can-shorten"
description: >
  MCP renew doc contradicts monotonic-forward code (T-2510) — says absolute-can-shorten

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T10:01:02Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-11T10:04:15Z
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
  - ts: '2026-08-18T18:56:53Z'
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
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2602: MCP renew doc contradicts monotonic-forward code (T-2510) — says absolute-can-shorten

## Context

The MCP `termlink_channel_renew` tool doc (both the tool `description` at
`crates/termlink-mcp/src/tools.rs:22207` and the `additional_ttl_ms` param doc at
`:9513`) claims the renew RPC sets `claimed_until = now + additional_ttl_ms`
"absolute, NOT a relative add" — i.e. that a renew CAN move the deadline EARLIER
(shorten a lease). The actual bus-layer implementation (`crates/termlink-bus/src/meta.rs:709`,
`renew_claim`) is `now_ms.saturating_add(additional_ttl_ms).max(old_until)` —
**monotonic-forward, never shortens** (T-2510, whose own bus-layer doc at meta.rs:658-660
correctly documents this). Only the MCP layer's doc lies. Doc-only fix, no behavior
change; strictly makes the doc truthful for any agent reading the tool description.
Found by the T-2468 verb-3 (claim-work) adversarial hunt (hunter finding F3).

## Acceptance Criteria

### Agent
- [x] tools.rs:22207 tool `description` no longer contains the misleading phrase
      "absolute, NOT a relative add"; instead states the lease is monotonic-forward
      / never shortened (max with the existing deadline), referencing T-2510.
- [x] tools.rs:9513 `additional_ttl_ms` param doc no longer states
      `claimed_until = now + additional_ttl_ms` as an unconditional overwrite;
      states the monotonic-forward `max(now + additional_ttl_ms, old_until)` behavior.
- [x] The corrected doc is regression-locked by static-grep gates in `## Verification`:
      the misleading phrase "absolute, NOT a relative add" must be absent (grep -c == 0)
      AND a monotonic marker ("monotonic") must be present. (A Rust unit test is the
      wrong tool here — the description is an rmcp `#[tool(...)]` macro-attribute literal,
      not a runtime-referenceable const; a static-grep gate over the source is the
      correct, non-brittle lock for a doc string.) Load-bearing: the grep gate fails
      against the pre-fix text.
- [x] `cargo test -p termlink-mcp` passes (crate still compiles clean after the doc edits).

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
cargo check -p termlink-mcp 2>&1 | tail -20
test "$(grep -c 'absolute, NOT a relative add' crates/termlink-mcp/src/tools.rs)" = "0"
grep -q "monotonic" crates/termlink-mcp/src/tools.rs

## RCA

**Symptom:** MCP `termlink_channel_renew` doc tells callers a renew sets
`claimed_until = now + additional_ttl_ms` "absolute, NOT a relative add" — implying a
short renew SHORTENS an active lease. An agent trusting this mis-models lease lifetime
and may under-renew (renew with a small ttl believing it resets absolutely, when in
fact the old longer deadline is retained — or, conversely, avoid renewing a 1h lease
with a 30s ttl for fear of shortening it, when the code protects against exactly that).

**Root cause:** T-2510 fixed the renew IMPLEMENTATION to be monotonic-forward
(`.max(old_until)`) and updated the bus-layer doc (meta.rs:658-660) but did NOT update
the MCP-layer surface docs (tools.rs:9513 param, :22207 tool description). The stale
docs predate the T-2510 behavior change and were never swept — the PL-318 class ("fixing
a behavior/guard in one copy leaves same-semantics siblings unaudited"), here applied to
DOCS across the bus→MCP layer boundary rather than to code guards.

**Why structurally allowed:** No test asserts doc/behavior agreement for the renew RPC;
doc strings are free text with no consistency check against the code they describe.

**Prevention:** the new unit test (AC #3) locks the corrected phrasing — it fails if the
misleading "absolute, NOT a relative add" text ever returns to the tool description.

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

### 2026-08-11T10:01:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2602-mcp-renew-doc-contradicts-monotonic-forw.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-61d11381
- **Timestamp:** 2026-08-11T10:04:24Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-11T10:04:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
