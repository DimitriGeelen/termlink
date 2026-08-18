---
id: T-2478
name: "P4 Stage 2 — retire CLI-entangled analytics tools (Groups A/B/D)"
description: >
  T-2468 P4 continuation of T-2471 (Stage 1 retired the 12 zero-consumer Group C tools).
  Groups A (reactions/emoji ~11), B (stars/pins leaderboards ~13), D (typing indicators
  + polls ~11) each have CLI-entangled members — the MCP tool and its CLI twin (+
  shared compute helper) must be retired together, per-group, with the same build+test+count
  guard. Investigate each group's CLI/slash consumers before deleting. Reversible
  via git.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-01T20:17:38Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-08-01T23:14:16Z
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
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2478: P4 Stage 2 — retire CLI-entangled analytics tools (Groups A/B/D)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

T-2468 P4 Stage 2, continuation of T-2471 (Stage 1 deleted 12 zero-consumer Group C
tools). **Human directive (2026-08-02): DEPRECATE, not delete.** Groups A
(reactions/emoji), B (stars/pins/leaderboards), D (typing/polls) are social-media
analytics that don't trace to the coordination-substrate charter, but they have CLI
twins and could be scripted against — so deprecation (reversible, keeps them
callable) is the right first step, not deletion.

**Mechanism:** mark each tool `[DEPRECATED]` at the agent-facing surface (MCP tool
description + `help_categories()` line) and hide + warn at the CLI twin, while
KEEPING the handler + compute helper intact (still callable, tests stay green).
Deletion is a later step gated on the deprecation soak.

## Acceptance Criteria

### Agent
- [x] Every MCP tool in Groups A (reactions/emoji=13), B (stars/pins=16), and
      D (typing/polls=11) — **40 total** — has its `#[tool]` description prefixed
      `[DEPRECATED — use termlink_channel_post]` (agent-facing signal), with the
      handler + `Parameters` struct LEFT INTACT so the tool is still callable
      (deprecate, not delete). Verified: `grep -c` = 40.
- [x] The corresponding `help_categories()` entries carry a
      `(deprecated P4/T-2478) (use termlink_channel_post instead)` marker (mirrors
      the `remote_inbox_*` T-1166 convention; trips `is_deprecated()` + satisfies the
      `every_deprecated_tool_has_replacement_hint` invariant). Verified: `grep -c` = 40.
- [x] CLI twins of the deprecated tools are hidden from `--help` via
      `#[command(hide = true)]` (15 clap variants: 7 ChannelAction + 8 AgentAction),
      while remaining functional. (`AgentAction::Poll` = event-bus poll, correctly
      untouched.) *Scope note:* a runtime warning-on-invocation was descoped — these
      verbs have zero CLI/script consumers, so hide is the load-bearing signal;
      warning-on-invocation is an optional future enhancement, not shipped here.
- [x] `cargo build -p termlink-mcp` and `cargo build -p termlink` (CLI package) both
      succeed.
- [x] `cargo test -p termlink-mcp --lib` passes: **880 passed, 0 failed**; both the
      `every_deprecated_tool_has_replacement_hint` and `help_registry_covers_all_real_tools`
      invariants stay green (entries marked, not removed → count consistency held).
- [x] No non-analytics consumer is broken: grep of `.claude/commands/` and `scripts/`
      for all 40 tools/verbs returned **zero** matches (same zero-consumer profile as
      the Stage-1 deletions).
- [x] Deprecation recorded in `docs/operations/p4-surface-reduction.md` (per-group
      inventory + mechanism + Stage-1/Stage-2 history).

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

test "$(grep -c '(use termlink_channel_post instead)' crates/termlink-mcp/src/tools.rs)" -eq 40
test "$(grep -c '\[DEPRECATED — use termlink_channel_post\]' crates/termlink-mcp/src/tools.rs)" -eq 40
test "$(grep -c '#\[command(hide = true)\]' crates/termlink-cli/src/cli.rs | head -1)" -ge 15
out=$(cargo test -p termlink-mcp --lib 2>&1); echo "$out" | grep -q "0 failed"
test -f docs/operations/p4-surface-reduction.md

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

### 2026-08-01T20:17:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2478-p4-stage-2--retire-cli-entangled-analyti.md
- **Context:** Initial task creation

### 2026-08-01T22:58:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-520cf15a
- **Timestamp:** 2026-08-01T23:14:38Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-01T23:14:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
