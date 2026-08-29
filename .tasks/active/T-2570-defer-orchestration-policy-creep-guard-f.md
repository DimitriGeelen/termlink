---
id: T-2570
name: "DEFER: orchestration-policy-creep guard for substrate crates (charter non-goal #4)"
description: >
  Filed from T-2468 purpose-review round (2026-08-09)

status: captured
workflow_type: build
owner: human
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T11:24:24Z
last_update: 2026-08-09T11:24:24Z
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

# T-2570: DEFER: orchestration-policy-creep guard for substrate crates (charter non-goal #4)

## Context

Filed from T-2468 non-goal-guard review (non-goal #4 "NOT a workflow/orchestration
engine" — the substrate stays mechanism, AEF builds policy on top). The only current
"guard" is convention plus one doc-comment marker (termlink-session/src/ack_retry.rs:24
self-documents "auto-resume is orchestration policy that belongs to the AEF layer").
No static check, no canary, no test enforces mechanism-vs-policy.

DEFER rationale: "policy creep into the substrate" is a SEMANTIC distinction (a
scheduler loop, a retry-policy default, a priority heuristic) with no reliable
syntactic signature. A grep check would be high-false-positive and low-recall.
Precise enforcement needs human architectural review, not a cheap automated detector
— unlike non-goals #2 (T-2562, shipped) and #1 (T-2569, cheap unit test). Parked
until a human decides whether periodic architectural review is worth formalising.

## Acceptance Criteria

### Human
- [ ] [REVIEW] Decide whether non-goal #4 warrants any structural guard at all, given it is
      semantic (no cheap syntactic signature) — vs accepting periodic human
      architectural review as the guard.
- [ ] [REVIEW] If a guard is wanted, scope what it would check (e.g. a review checklist item,
      an ADR gate on new substrate-crate public APIs, or a curated denylist of
      policy-shaped constructs) — NOT a naive grep.
- [ ] [RUBBER-STAMP] Record the decision; if "no automated guard", close with that rationale so the
      gap is knowingly-accepted, not silently open.

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

**Recommendation:** CLOSE — record non-goal #4 as a knowingly-accepted gap guarded
by human architectural review, not by an automated check.

**Rationale:** The task's third Human AC already names this as a valid outcome
("if 'no automated guard', close with that rationale so the gap is
knowingly-accepted, not silently open"), and the repo's own guard-design experience
argues for it. This codebase has repeatedly found that a detector with no reliable
signature is worse than none: T-2818 documents 150 findings teaching an operator to
force past a gate, and T-2833's first draft produced 58 findings that were all
legitimate and had to be narrowed before the check was usable. "A scheduler loop, a
retry-policy default, a priority heuristic" have no syntactic form to anchor on, so
a grep-shaped guard lands squarely in that class. Closing with the reason recorded
converts an open question into a documented decision; leaving it open converts it
into a recurring re-litigation.

**Evidence (measured 2026-08-27, this repo):**
- `non-goal #4` appears in **exactly one file**,
  `crates/termlink-session/src/ack_retry.rs` (2 lines), confirming the task's claim
  that the only guard is one self-documenting doc-comment. Read in place: it states
  auto-resume "is orchestration policy that belongs to the AEF layer, not the hub".
- No static check, canary, or fixture references non-goal #4 or mechanism-vs-policy.
- Whether policy has ALREADY crept into the substrate crates: **not measured**. I
  did not audit the crates against the non-goal. That matters — see below.

**What you are actually deciding.** Whether periodic human architectural review is
an acceptable guard for a semantic non-goal:

| Option | Cost |
|---|---|
| **Close as knowingly-accepted** (recommended) | Non-goal #4 has no daily detection. Real, but it never did, and a bad detector would not give it one. |
| Curated denylist of policy-shaped constructs | Someone must maintain the list; it catches only shapes already thought of, and reads as full coverage when it is not — the T-2680 failure mode. |
| ADR gate on new substrate-crate public APIs | The most defensible option and the only one that scopes to a real decision point. Costs process on every new public API, forever. |

**The gap in my own recommendation, stated plainly.** I am recommending you close a
guard task without anyone having checked whether the thing it guards is currently
violated. A one-off audit of the substrate crates against non-goal #4 is a different
and cheaper question than a standing guard, and it would tell you whether you are
accepting a clean boundary or an already-blurred one. If that answer would change
your decision, get it before closing.

**Why I should not decide this alone.** Whether the mechanism/policy boundary is
worth process cost is an architectural judgement about how you intend the substrate
and AEF to evolve. I can show that a cheap detector would be bad; I cannot tell you
whether an ADR gate is worth its friction.

**If you prefer to keep it open:** the ADR-gate option is the one to pursue —
narrow it to "new public API on a substrate crate requires a one-line
mechanism-vs-policy justification", which is a review checklist item rather than a
static check.

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

### 2026-08-09T11:24:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2570-defer-orchestration-policy-creep-guard-f.md
- **Context:** Initial task creation
