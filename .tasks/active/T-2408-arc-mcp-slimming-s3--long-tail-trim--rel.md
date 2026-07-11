---
id: T-2408
name: "arc mcp-slimming S3 — long-tail trim + relocate archaeology to docs + tighten guard to final ceiling"
description: >
  Closing slice of arc-005 mcp-slimming. Trim the remaining 400-600 char descriptions, relocate any genuinely-useful T-XXXX/PL lineage from descriptions into docs/ or task files (not lost, just moved), and tighten the anti-regrowth guard ceiling to the final target. Arc close demo: before/after total-bytes + a diff showing a representative slimmed tool + guard test green.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-11T14:16:26Z
last_update: 2026-07-11T21:33:03Z
date_finished: 2026-07-11T21:32:35Z
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

# T-2408: arc mcp-slimming S3 — long-tail trim + relocate archaeology to docs + tighten guard to final ceiling

## Context

Final slice of arc `mcp-slimming`. S1 (T-2406) + S2 (T-2407) took the tool-description total
from 156,525 → 112,319 bytes (~44KB / ~11k tokens/agent reclaimed) by trimming the >1000 and
600–999 bands. This slice sweeps the sub-600 long tail: **65 descriptions under 600 chars still
carry task-ID archaeology, "MCP parity for CLI verb" cross-refs, or "arc Slice N" provenance**
(~28KB across them). Trim those per `docs/operations/mcp-description-policy.md`, relocate any
genuinely-useful rationale (not ID soup) to docs, then tighten the guard to its final ceiling
and close the arc.

## Acceptance Criteria

### Agent
- [x] **Sub-600 archaeology swept.** APPLIED 2026-07-11 (fresh session, budget ok). Ran the
  staged `.context/working/mcp-s3-apply.py`: 73 fragment-pairs applied, tool **count 273
  (unchanged)**, `cargo build -p termlink-mcp` clean, `cargo test -p termlink-mcp` green
  (879 + 99 + 24 passed, incl. the rewritten `help_macro_description_documents_post_t1953_fields`
  drift-guard). **112,319 → 108,051 bytes = 4,268 reclaimed** (~1,067 more tokens/agent).
  Diff character verified policy-correct: drops "MCP-side equivalent of `agent X` (CLI T-NNNN)",
  "T-15xx" archaeology, "arc Slice N" / "NO new RPC" prose; keeps purpose + return-shape
  (`Returns {ok, ...}`) + non-obvious param semantics. Safety words on the claim-lifecycle
  tools (`ack=true advances cursor, ack=false reopens slot`) were preserved in S2 and untouched
  by S3.
- [x] **Guard tightened to final ceiling.** `scripts/test-mcp-desc-budget.sh` auto-patched by
  the apply script: `TOTAL_DESC_CEILING` 113000 → 109000, `MAX_DESC_CEILING` stays 1560
  (the 1546-char `termlink_help` is out of band). Guard PASSES (273 tools / 108,051 bytes /
  max 1546). Ceiling-history comment gained the S3 line.
- [x] **Arc-close demo prepared + surfaced for human decision.** `fw arc close mcp-slimming`
  is HUMAN-GATED (carries inception-level authority; agent may not self-approve per §Autonomous
  Mode Boundaries). Demo evidence below; the close command is queued as a Human AC.
  **Arc total: 156,525 → 108,051 bytes = 48,474 reclaimed (~12,100 tokens/agent/session,
  ~31% of the original tool-catalog tax).** Tool count unchanged (273) across all three slices.
  Memory + arc yaml updated.

### Human
- [ ] [RUBBER-STAMP] Close arc mcp-slimming with the demo evidence.
  **Steps:**
  1. `cd /opt/termlink && .agentic-framework/bin/fw task review T-2406`
  2. Review the before/after: arc total **156,525 → 108,051 bytes** (~48KB / ~12k tokens
     per agent per session reclaimed, ~31% of the original tool-catalog tax); tool count
     unchanged (273); full test suite green; anti-regrowth guard locks it at 109,000 bytes.
  3. `cd /opt/termlink && .agentic-framework/bin/fw arc close mcp-slimming --decision "shipped: 156525→108051 bytes, ~12k tokens/agent reclaimed, guard-locked, 273 tools intact"`
  **Expected:** arc yaml status → closed; `fw arc list` no longer shows mcp-slimming in-progress.
  **If not:** re-run with `--i-am-human` if the CLADUECODE guard blocks a genuine human session.

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

bash scripts/test-mcp-desc-budget.sh
test $(python3 -c "import re;print(len(re.findall(r'description\s*=\s*\"', open('crates/termlink-mcp/src/tools.rs').read())))") -eq 273

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

## Recommendation

**Recommendation:** GO — close arc mcp-slimming.

**Rationale:** All three slices shipped and are guard-locked. The refactor is done, verified,
and reversible-proof (the anti-regrowth guard fails CI if descriptions creep back). The only
remaining action is the human-authority arc-close stamp — there is no engineering risk left to
weigh, only the sovereignty formality that arc closure belongs to the human.

**Evidence:**
- Arc total: **156,525 → 108,051 bytes = 48,474 reclaimed (~12,100 tokens/agent/session, ~31%
  of the original tool-catalog tax)**. Per slice: S1 23.3KB (1 meta-tool) · S2 20.9KB (86 tools)
  · S3 4.3KB (65 tools).
- Tool count unchanged (273) across all three slices — trims were text-only, no tool removed.
- `cargo test -p termlink-mcp` green: 879 + 99 + 24 passed, including the rewritten
  `help_macro_description_documents_post_t1953_fields` drift-guard.
- Anti-regrowth guard `scripts/test-mcp-desc-budget.sh` PASS at ceiling 109,000 (max 1560);
  ratcheted at every slice so the win can't silently regrow.
- Commits: cb5d74ca (S1) · ebdd5f5f + 612a8d84 (S2) · a833a127 + e24e1e19 (S3), all on OneDev.

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

### 2026-07-11 — S3 applied a session late; the staging pattern paid off
- **What changed:** S3 was authored + dry-run-verified in the prior session but blocked from
  applying by the framework budget gate (source edits forbidden at critical). Staging the
  entire trim as a self-contained apply script (`.context/working/mcp-s3-apply.py`, fragment
  pairs + count assertion + guard auto-patch) meant the fresh session applied it in ONE command
  with zero re-derivation — build+test+guard all green on first run.
- **Plan impact:** Confirms the per-slice discipline scales across a session boundary: the guard
  is the objective success signal, so an interrupted slice resumes as a mechanical replay, not a
  re-think. No plan change.
- **Triggered:** Arc close. The head-heavy shape held — S3's sub-600 long tail yielded 4.3KB vs
  S1's 23.3KB (one meta-tool) and S2's 20.9KB (86 tools). Diminishing returns past the 600-char
  band; no S4 warranted. Guard now locks the win at 109,000 bytes so it can't creep back.

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

### 2026-07-11T14:16:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2408-arc-mcp-slimming-s3--long-tail-trim--rel.md
- **Context:** Initial task creation

### 2026-07-11T16:53:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-6d0d51d4
- **Timestamp:** 2026-07-11T21:32:37Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-11T21:32:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
