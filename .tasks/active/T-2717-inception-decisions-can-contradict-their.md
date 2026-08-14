---
id: T-2717
name: "Inception decisions can contradict their own recorded rationale"
description: >
  Two of six completed inceptions record Decision GO with a rationale arguing DEFER or NO-GO verbatim; nothing detects a verdict that contradicts its attached reasoning

status: captured
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
created: 2026-08-14T20:30:13Z
last_update: 2026-08-14T20:30:13Z
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

# T-2717: Inception decisions can contradict their own recorded rationale

## Context

Found while consolidating research artifacts under T-2716. Of the six completed
inceptions reviewed, **two record a `## Decision` of GO whose attached rationale
argues the opposite, verbatim**:

| Task | `## Recommendation` | `## Decision` | Rationale actually recorded |
|---|---|---|---|
| T-1793 | **DEFER** | **GO** | the DEFER text, ending *"Parked at horizon=later because..."* |
| T-2288 | **NO-GO** | **GO** | the NO-GO text, including *"a termlink-driven build now would violate the gate"* |

The other four (T-1692, T-1693, T-1830, T-2054) are consistent — and in every one
of those the recommendation was already GO. So the divergence appears exactly and
only where the recommendation was *not* GO, across two tasks five weeks apart
(2026-05-25 and 2026-06-26). Two independent instances sharing one precondition is
a mechanism, not a slip.

**This is not an auto-fill bug.** `lib/inception.sh:421-423` makes `--rationale`
mandatory for `decide` and errors without it, so in both cases someone passed the
recommendation body explicitly while typing `go` as the verdict.

**Why it matters more than a cosmetic inconsistency.** A recorded inception
decision is a human-sovereignty artifact and the thing every downstream gate
trusts: the commit-msg inception gate unblocks on `**Decision**: GO`, and
`fw inception decide` is precisely the authority an agent may not exercise. When
the verdict field and its own reasoning disagree, the record cannot serve as
evidence in either direction — a reader who trusts the field concludes the
opposite of a reader who reads the paragraph beneath it.

T-2288 is the sharper case. Its whole finding is *"a build cannot proceed without a
recorded GO, and an agent cannot record one."* It is now itself recorded as GO,
with that sentence attached.

**What is genuinely unknown from here:** whether each instance was a deliberate
human override (legitimate — sovereignty includes overriding a recommendation) or a
mis-typed verdict. That cannot be recovered from the artifacts, and this task must
not assume either. The defect being filed is the *undetectability*, not the
verdicts: nothing in the framework notices that a decision contradicts the
reasoning stored beside it, so a genuine override and a typo are indistinguishable
afterwards. G-019 — the instance is unrecoverable; the blindness is fixable.

**Cross-repo.** `lib/inception.sh` is vendored; a local edit is erased on
re-vendor. Deliverable is an upstream report, not an edit under
`.agentic-framework/`.

## Acceptance Criteria

### Agent
- [ ] The report tabulates both instances with task id, date, recommendation, decision, and the contradicting rationale excerpt
- [ ] It shows the four consistent cases and that all four had a GO recommendation, establishing the precondition rather than asserting it
- [ ] It rules out tool auto-fill by citing `lib/inception.sh:421-423` (`--rationale` is mandatory for `decide`)
- [ ] It states plainly that whether each was an override or a typo is unrecoverable, and that the filing targets the undetectability
- [ ] A concrete remedy is proposed: on `fw inception decide`, when the rationale contains a `Recommendation: <verdict>` line disagreeing with the verdict being recorded, require explicit confirmation (or `--override-recommendation`) so a deliberate override is recorded AS an override
- [ ] Filed to `framework:pickup` and the post confirmed present
- [ ] No file under `.agentic-framework/` is edited by this task
- [ ] No decision field on T-1793 or T-2288 is altered by this task

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

### 2026-08-14T20:30:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2717-inception-decisions-can-contradict-their.md
- **Context:** Initial task creation
