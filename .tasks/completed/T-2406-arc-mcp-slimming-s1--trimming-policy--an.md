---
id: T-2406
name: "arc mcp-slimming S1 — trimming policy + anti-regrowth length guard + trim worst-offender MCP descriptions"
description: >
  arc mcp-slimming S1 — trimming policy + anti-regrowth length guard + trim worst-offender MCP descriptions

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: ["arc:mcp-slimming"]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-11T14:14:55Z
last_update: 2026-07-11T14:18:33Z
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

# T-2406: arc mcp-slimming S1 — trimming policy + anti-regrowth length guard + trim worst-offender MCP descriptions

## Context

Anchor slice of arc `mcp-slimming`. Measured baseline (crates/termlink-mcp/src/tools.rs,
45,646 lines): **273 tool descriptions, ~156KB total (~39k tokens)** loaded into every
agent's context each session. max=11,751 chars, 24 exceed 1000, 94 exceed 600, median 480.
The bloat is task-ID archaeology (T-XXXX lineage chains), PL cross-refs that belong in
docs, and restatement of params already present in the JSON schema. This slice locks the
POLICY, adds an anti-regrowth guard so it can't creep back, and proves the approach by
trimming the worst offenders — without losing agent-critical guidance (what the tool does,
non-obvious param gotchas, critical safety notes stay).

## Acceptance Criteria

### Agent
- [x] **Trimming policy documented.** A short policy (in the arc doc or task) states what
  STAYS (one-line purpose, non-obvious param semantics, safety/mutex gotchas) and what GOES
  (T-XXXX lineage chains, PL-NNN cross-refs → move to docs/task files, restatement of
  schema-visible param types/defaults, redundant sibling-tool prose). **DONE:** `docs/operations/mcp-description-policy.md` — keep/cut rules + a before/after `recent_dm` example (~1500→~250 chars) + per-slice process (trim → compile → tighten ceiling → report).
- [x] **Anti-regrowth guard exists + passes.** A test (e.g. `scripts/test-mcp-desc-budget.sh`
  or a `#[test]` in termlink-mcp) fails if any tool `description` exceeds a set ceiling
  (initial ceiling generous enough to pass after this slice's trims; tightened in later
  slices). Wired so `cargo test`/CI catches a regrowth. The guard reports the current max +
  total bytes for visibility. **DONE:** `scripts/test-mcp-desc-budget.sh` — reports count/total-bytes/~tokens/max, FAILs if max>MAX_DESC_CEILING (12000) or total>TOTAL_DESC_CEILING (160000); PASSes at baseline (273 tools, 156525 bytes, max 11751). `--report-only` mode. Ceilings tighten per slice.
- [x] **Worst offenders trimmed + still compiles.** DONE (fresh session 2026-07-11): trimmed
  the 11,751-char tool-catalog meta-tool → 1,546 AND all 24 descriptions over 1000 chars per
  policy. `cargo build -p termlink-mcp` passes; **tool count unchanged (273)** — text trimmed,
  no tools removed. **Bytes reclaimed: 156,525 → 133,220 = 23,305 (~5,800 tokens/agent/session,
  ~15% of the tool-catalog tax).** Max single 11,751 → 1,546. Guard ceilings tightened in
  lockstep: MAX 12000→1600, TOTAL 160000→135000; `bash scripts/test-mcp-desc-budget.sh` PASSES.
  Updated the T-1962 drift-guard test (`help_macro_description_documents_post_t1953_fields`) to
  the arc-005 contract — description documents modes+input-params, not every envelope field
  (fields stay discoverable via `tool_detail`/`summary` + JSON schema). `cargo test -p termlink-mcp`
  green.

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
test -f docs/operations/mcp-description-policy.md
bash scripts/test-mcp-desc-budget.sh
cargo build -p termlink-mcp 2>&1 | tail -1 | grep -q Finished

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

### 2026-07-11 — worst-offender trim + the drift-guard tension
- **What changed:** The single 11,751-char `termlink_help` meta-tool was ~7.5% of the
  entire tool-catalog tax by itself. Trimming it + the 24 over-1000 descriptions reclaimed
  23,305 bytes (~5,800 tokens/agent) — more than the naive "trim the long tail" framing
  predicted, because the head is so heavy (Pareto: a handful of tools carry most of the bytes).
- **Plan impact:** S1 alone already banks the majority of the achievable win. S2/S3 (the
  600-1000 and long-tail bands) are real but each yields far less than this head trim — the
  arc's value is front-loaded, which is the right shape for a "born-in-progress" arc.
- **Triggered:** The T-1962 drift-guard test (`help_macro_description_documents_post_t1953_fields`)
  FAILED on the trim — it asserted the meta-tool description restate ~30 envelope field names,
  i.e. it *enforced the exact bloat the arc removes*. Rewrote it to the arc-005 contract
  (document modes+input-params; envelope fields stay discoverable via `tool_detail`/`summary`
  + JSON schema at runtime). This is the structural-flaw pattern: the fix surfaced a test that
  codified the anti-pattern. No new sub-task needed — handled in-slice.

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

### 2026-07-11T14:14:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2406-arc-mcp-slimming-s1--trimming-policy--an.md
- **Context:** Initial task creation
