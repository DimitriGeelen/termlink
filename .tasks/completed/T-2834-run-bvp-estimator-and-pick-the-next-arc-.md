---
id: T-2834
name: "run BVP estimator and pick the next arc-scoped HV/LC landing"
description: >
  run BVP estimator and pick the next arc-scoped HV/LC landing

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T19:10:28Z
last_update: 2026-08-23T19:15:55Z
date_finished: 2026-08-23T19:15:55Z
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
  - ts: '2026-08-23T19:13:29Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 4
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=4 (body:rubric-routable)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-23T19:13:48Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2834: run BVP estimator and pick the next arc-scoped HV/LC landing

## Context

Ran both BVP axes over `started-work` tasks and produced the quadrant, per the
recipe in CLAUDE.md. `ruamel` is present (0.19.1), so the estimator took the
comment-preserving path rather than the `yaml.safe_dump` fallback that silently
strips the frontmatter's own documentation (T-2809).

- value axis: `fw bvp estimate all --statuses started-work` → 93 tasks, 36 wrote, 0 errored
- cost axis: `estimator.py cost-all --statuses started-work` → 93 tasks, 36 wrote, 0 errored

Both axes matter: `fw bvp` does not surface the `cost-*` subcommands, so a
value-only run leaves the cost axis empty and `--quadrant hv-lc` answers "No
tasks match" — which reads as "nothing is high-value/low-cost" when it actually
means "half the quadrant was never computed".

**HV/LC head:** T-2713 (92), T-2714 (92), T-2197 (81), T-2203 (80), T-2715 (80),
T-2721 (80), T-2016 (78), T-2687 (76), T-2695 (75), T-2679 (73), T-2680 (73).
HV/HC is deliberately not drawn from until HV/LC is exhausted.

### The quadrant is contaminated, and that is the real finding

Reading the head against CLAUDE.md, a large share of it is **work that already
shipped**: T-2721 (worktree-safe hook paths — registered in
`.vendor-divergence.yaml`, observed live this session), T-2680, T-2684, T-2686,
T-2692, T-2699, T-2691, T-2709 are all documented in CLAUDE.md as delivered, yet
all still sit `started-work` and therefore still score.

Measured across `.tasks/active/`: **52 tasks have every Agent AC ticked, zero
unticked, and are still `started-work`.** That is the T-2806 class — CLAUDE.md
records 24 of them; it is now 52. Distinct from the T-2833 latch closed
immediately before this task: those had already been *told* to complete and
half-ran, whereas these were never told at all.

The cost is not bookkeeping. A ranking computed over a backlog where ~a quarter
of the entries are finished work does not rank the remaining work correctly, so
the estimator's output degrades exactly as it is trusted more. Every one of these
also holds a slot the commit gates care about.

**Pick: T-2687** — top of the in-arc HV/LC band at 76, `owner: agent`, 6/6 Agent
ACs ticked, 3 concrete verification commands, and the defect this branch is named
for (`termlink_topics` returning a silently partial inventory, caught by the
`parity_topics` case that had been failing undetected since 2026-08-12). Landing
it is closing the branch's own subject, not opening new ground.

Deliberately NOT picked: T-2815 and T-2819 are `owner: human` and remain the
human's to close, regardless of BVP rank.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The BVP estimator runs over active tasks and populates BOTH axes — `fw bvp estimate` for value and the estimator's `cost-all` for cost, since `fw bvp` does not surface the cost subcommands and a quadrant needs both (a value-only run makes `--quadrant hv-lc` return "No tasks match" while silently meaning "the cost axis is empty")
- [x] `ruamel` is confirmed present before the estimator writes, so frontmatter comments are not silently stripped by the `yaml.safe_dump` fallback path (T-2809)
- [x] The HV/LC quadrant is produced and reported, with HV/HC named separately so the ordering is visible rather than asserted
- [x] The next unit of work is picked from that quadrant and scoped to the current arc, or — if the quadrant is empty or entirely out-of-arc — that is stated plainly rather than substituted with an unranked pick
- [x] Scores are left as `bvp_scores_proposed` (agent proposes); no confirmed `bvp_scores` are written, since confirmation is §ACD sovereignty-gated to `fw bvp confirm --i-am-human`

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

# Both BVP axes are populated on the pick, so its quadrant is real and not half-computed
python3 -c "import yaml,sys; s=open('.tasks/active/T-2687-mcp-termlinktopics-returns-a-silently-pa.md').read(); fm=yaml.safe_load(s.split('---')[1]); e=fm['bvp_scores_proposed'][-1]['scores']; assert 'D1' in e, e"
grep -q "cost_estimate" .tasks/active/T-2687-mcp-termlinktopics-returns-a-silently-pa.md
# ruamel was present, so frontmatter comments survived the estimator's rewrite (T-2809)
python3 -c "import ruamel.yaml"
grep -q "# revisit_at:" .tasks/active/T-2687-mcp-termlinktopics-returns-a-silently-pa.md
# The estimator did not confirm any scores — confirmation is human-only (ACD sovereignty)
test -z "$(grep -c '^bvp_scores:' .tasks/active/T-2687-mcp-termlinktopics-returns-a-silently-pa.md | grep -v '^0$' || true)"
# The pick is agent-owned, so completing it is not a sovereignty breach
out=$(grep -m1 '^owner:' .tasks/active/T-2687-mcp-termlinktopics-returns-a-silently-pa.md); echo "$out" | grep -q "agent"

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

### 2026-08-23T19:10:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2834-run-bvp-estimator-and-pick-the-next-arc-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c334c70c
- **Timestamp:** 2026-08-23T19:15:56Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-23T19:15:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
