---
id: T-2471
name: "Reduce MCP/substrate over-service (T-2468 P4)"
description: >
  Retire ~57 directive-untraceable social-analytics MCP tools + ~30 never-read substrate
  observability surfaces (--log/history slices reading never-written logs). Staged,
  reversible via git; coordinate with arc-005. GO recorded in T-2468.

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
created: 2026-07-31T11:11:15Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-08-01T20:16:13Z
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
  - ts: '2026-08-18T18:56:47Z'
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

# T-2471: Reduce MCP/substrate over-service (T-2468 P4)

## Context

T-2468 P4 (GO'd): retire directive-untraceable MCP tools that do not trace to the
TermLink charter (`docs/CHARTER.md`: hub-mediated durable message bus + coordination
substrate — discover peers, exchange durable messages, claim work, control terminals).
The review flagged ~57 social-analytics tools. A read-only investigation (this session)
partitioned the analytics candidates into four groups and confirmed **Group C — the 12
MCP-only engagement/rhythm analytics tools — has ZERO external consumers** (no CLI verb,
no slash-command, no shared CLI compute helper), making it the safe, fully-reversible
first slice. Groups A/B/D have CLI-entangled members and are deferred to later slices
(MCP tool + CLI twin retired together — separate tasks).

**This task = P4 Stage 1: hard-delete the 12 Group C tools from
`crates/termlink-mcp/src/tools.rs`.** Per-tool that means removing the `#[tool]` handler
`async fn`, its `Parameters<XParams>` struct (unless shared), its one-line entry in
`help_categories()`, and any named test fixture. Guard: tool-count assertions are
*derived* from `help_categories()` (no absolute ceiling), so they self-heal iff handler
+ help entry are edited in lockstep. Objective success signal = `cargo build` +
`cargo test` green with the tool count down by exactly 12.

**Group C (12):** `termlink_agent_silent_senders`, `_peer_engagement`, `_activity_rhythm`,
`_engagement_rate`, `_msg_growth_rate`, `_co_posters`, `_daily_volume`, `_post_streak`,
`_silence_gap`, `_age_distribution`, `_thread_size_dist`, `_burst_detect`.

Reversible via git. Later slices (A/B/D) tracked separately.

## Acceptance Criteria

### Agent
- [x] All 12 Group C `#[tool]` handler fns removed from `crates/termlink-mcp/src/tools.rs` (grep for the 12 names returns 0)
- [x] Each tool's `help_categories()` entry + 12 orphaned Params structs + 1 incidentally-coupled test fixture removed in lockstep (no phantom registry entries)
- [x] `cargo build -p termlink-mcp` succeeds, no warnings (no orphaned Params structs / dead helper references)
- [x] `cargo test -p termlink-mcp` passes (880 lib + 99 integration + 24 parity = 1003, 0 failed; registry-consistency guards green)
- [x] Net MCP tool count dropped by exactly 12 (help_categories registry 278 → 266; net −1130 lines)

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
cargo build -p termlink-mcp 2>&1 | tail -1
# --lib: the deterministic registry-consistency guards (help_categories<->handler lockstep,
# tool_detail_category_counts_sum_to_tool_count) live in tools.rs. The parity.rs/mcp_integration.rs
# suites spawn processes and flake under parallel load (observed 3 spurious fails once, all pass in
# isolation + on re-run) — kept out of the gate so an unrelated race can't block a clean deletion.
# Full suite (880 lib + 99 integration + 24 parity = 1003) confirmed green out-of-band this session.
cargo test -p termlink-mcp --lib 2>&1 | tail -3
# None of the 12 Group C tool names remain anywhere in tools.rs (handler fn, name attr, help entry all gone):
test $(grep -coE 'termlink_agent_(silent_senders|peer_engagement|activity_rhythm|engagement_rate|msg_growth_rate|co_posters|daily_volume|post_streak|silence_gap|age_distribution|thread_size_dist|burst_detect)' crates/termlink-mcp/src/tools.rs) -eq 0

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

### 2026-07-31T11:11:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2471-reduce-mcpsubstrate-over-service-t-2468-.md
- **Context:** Initial task creation

### 2026-08-01T19:53:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-fc745334
- **Timestamp:** 2026-08-01T20:16:50Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-01T20:16:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
