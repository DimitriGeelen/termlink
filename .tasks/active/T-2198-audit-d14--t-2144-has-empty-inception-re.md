---
id: T-2198
name: "Audit D14 — T-2144 has empty inception Recommendation (false-positive or finalize
  gap)"
description: >
  Audit D14 WARN: T-2144 'Empty inception Recommendation'. Scout: T-2144 ACTUALLY
  has a populated Recommendation section ('Substrate is at clean ship state. Forward
  motion needs human input on...'). Two possible RCAs: (a) auditor pattern looks for
  specific anchor that doesn't match the existing format; (b) T-2144 needs to be finalized
  via fw inception decide to record the conclusion structurally. Both — fix the auditor
  false-positive AND structurally close T-2144.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-12T10:21:07Z
last_update: '2026-08-20T15:21:21Z'
date_finished:
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
  - ts: '2026-08-20T15:20:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2198: Audit D14 — T-2144 has empty inception Recommendation (false-positive or finalize gap)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Read D14 check at `.agentic-framework/agents/audit/audit.sh` (Python heredoc). Pattern: requires `**Recommendation:**` (bold prefix) at line-start of Recommendation section body. T-1510 added bulleted variants. T-2144's prose-only format failed the regex
- [x] Updated T-2144's `## Recommendation` to canonical format: `**Recommendation:** NO-GO on additional substrate-arc primitives...` + rationale. D14 pattern will match next audit run
- [ ] **(HUMAN)** Finalize T-2144 structurally: `cd /opt/termlink && .agentic-framework/bin/fw inception decide T-2144 no-go --rationale "Substrate at clean ship state — no ship-ready slice without human input on T-2090/T-2025/T-2022/T-2024/T-2026"` (Tier-0 blocks agent execution; human authority required per autonomous-mode boundary)
- [ ] Re-run audit after human decide, confirm D14 no longer fires on T-2144 (covered automatically next audit cycle)

### Human
- [ ] [REVIEWER] Approve the T-2144 NO-GO decision. **Steps:** 1) read T-2144's updated Recommendation; 2) if you agree with the NO-GO conclusion (substrate at clean ship state, deferred primitives need human GO/NO-GO), run `cd /opt/termlink && .agentic-framework/bin/fw inception decide T-2144 no-go --rationale "..."` — copy rationale from the T-2144 file. **Expected:** T-2144 transitions to work-completed and moves to .tasks/completed/. **If not:** if you disagree (substrate has open slice you want shipped), decide go + file a build task per the ship-ready slice you want

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

## Recommendation

**Recommendation:** CLOSE — the decision this task asks you to make was already
made and recorded on 2026-06-25, and D14 has been passing since.

**Rationale:** Both halves of this task's stated scope are satisfied in the tree.
T-2144's NO-GO is not pending: it is written into T-2144's `## Decision` section
in `fw inception decide` form, the task is finalized and archived, and the D14
auditor no longer names it. The Human `[REVIEWER]` box asks you to run a command
whose effect is already present. What is left is a tick, not a decision — with one
caveat below that is worth reading before you tick it.

**Evidence:** `.tasks/completed/T-2144-substrate-primitive-review--identify-nex.md`
— `status: work-completed`, `date_finished: 2026-06-25T06:31:59Z`, and a populated
`## Decision` reading `**Decision**: NO-GO` with the substrate-at-clean-ship-state
rationale carried over verbatim from its Recommendation. `.context/audits/2026-08-20.yaml:894`
records `D14: Empty inception Recommendation — none (PASS no_empty_recommendations)`,
which is the "re-run audit, confirm D14 no longer fires" Agent AC, met.

**The caveat — the false positive was not fixed, it was outrun.** This task's
`description` proposes two RCAs and says to do both: fix the auditor false positive
AND close T-2144. Only the second happened. The D14 check at
`.agentic-framework/agents/audit/audit.sh:5958` still requires
`^\s*[-*]?\s*\*\*Recommendation:\*\*\s*\S`; nothing in it changed (last touches
were re-vendors and T-2721's worktree fixes, none to this predicate). T-2144 stopped
firing because AC 2 reformatted *the task* to match the pattern — and then because
T-2144 left `active/`, which is the only directory D14 scans. So the next inception
that writes a prose-only Recommendation will fire D14 exactly as before. It is also
vendored code (G-062), so a local fix would be erased by the next re-vendor.

**What you are actually deciding.**

| Option | Behaviour | Cost |
|---|---|---|
| CLOSE (recommended) | tick the `[REVIEWER]` box; T-2144's NO-GO stands as recorded | the strict D14 pattern stays; the class recurs on the next prose-only Recommendation, with nothing tracking it |
| CLOSE + file the pattern upstream | same, plus the checker half goes to `framework:pickup` | one filing's effort; the honest close for a task whose description promised both halves |
| KEEP-OPEN | hold this task until the auditor half is done | a task pinned open on vendored code we cannot fix here anyway |

**Why I should not decide this alone.** Two things. Reversing or confirming your own
prior authorization is a `[REVIEWER]` tick under the Authority Model. And whether a
checker that only stopped firing because its one subject moved directories counts as
"fixed" is a standards call about what this project will accept as closed — the same
judgement that separates mitigation from prevention under G-019, and not mine to make.

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

### 2026-06-12T10:21:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2198-audit-d14--t-2144-has-empty-inception-re.md
- **Context:** Initial task creation

### 2026-06-12T10:52:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-12T10:52:15Z — status-update [task-update-agent]
- **Change:** owner: agent → human
