---
id: T-2131
name: "fix CLAUDE.md claimed_by → claimer (PL-206 class instance, L1168 and L1169)"
description: >
  fix CLAUDE.md claimed_by → claimer (PL-206 class instance, L1168 and L1169)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, doc, bug]
components: []
related_tasks: [T-2129, T-2130]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T16:31:40Z
last_update: 2026-06-10T16:31:40Z
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

# T-2131: fix CLAUDE.md claimed_by → claimer (PL-206 class instance, L1168 and L1169)

## Context

Third file with the same PL-206 drift class T-2129 + T-2130 just fixed.
CLAUDE.md L1168 and L1169 both reference the non-existent field `claimed_by`
in the substrate primitives quick-reference table:

- L1168 (find-idle row): `LIVE(agent-presence) \ DISTINCT(claimed_by)` —
  pseudo-relational anti-join, same form as T-2130 fixed in
  `docs/operations/agent-find-idle.md`.
- L1169 (claim-transfer row): "`by` MUST equal current `claimed_by`
  (returns CLAIM_NOT_OWNED -32017 otherwise)" — API contract prose;
  the hub-side check is against the `claimer` field per
  `crates/termlink-bus/src/claim.rs:22` + `claim.rs:67` (TransferInfo).

CLAUDE.md is auto-loaded into every agent session — drift here propagates
to every fresh context. One bug = one task per CLAUDE.md (separate from
the two ops docs already fixed).

## Acceptance Criteria

### Agent
- [x] L1168: `DISTINCT(claimed_by)` → `DISTINCT(claimer)` in find-idle row of substrate primitives table
- [x] L1169: `current \`claimed_by\`` → `current \`claimer\`` in claim-transfer row API-contract prose
- [x] Zero remaining `claimed_by` references in CLAUDE.md (sweep confirms only L1168 and L1169 instances at task-creation time)

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

# Zero claimed_by references remaining in CLAUDE.md.
out=$(grep -nE 'claimed_by' CLAUDE.md 2>&1); [ -z "$out" ]
# At least one new claimer reference present in the substrate table region.
grep -qE 'DISTINCT\(claimer\)|current \`claimer\`' CLAUDE.md
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

**Symptom:** CLAUDE.md L1168 and L1169 both reference `claimed_by` as the
field/column name in the substrate-primitives quick-reference table. The
hub uses `claimer` end-to-end (`crates/termlink-bus/src/claim.rs:22`,
`crates/termlink-hub/src/channel.rs:1377`, `:1379`). Every fresh agent
session auto-loads CLAUDE.md → every fresh agent inherits the wrong
field name as part of their system prompt.

**Root cause:** Identical to T-2129/T-2130 — doc-prose authored against
an assumed schema, no lint asserting field names in doc-prose match
live serialization. Specific to CLAUDE.md: the quick-reference table is
generated by hand-curation during task closures (e.g. T-2046 added the
claim-transfer row), and the original author imported "claimed_by" from
their mental model rather than verifying against the struct.

**Why structurally allowed:** Same as PL-206. CLAUDE.md is plain markdown
with no schema validation; the quick-reference table cells routinely
embed example JSON shapes, API-contract prose, and CLI usage strings
without compile-time verification. The first observable failure surfaces
when an agent tries to use the misremembered field in a jq selector
(exactly T-2129's L219 bug).

**Prevention:** Same PL-206 mitigation layers apply. CLAUDE.md is in
scope for any "doc-prose field-name lint" tooling — the sweep grep
(`grep -rnE 'claimed_by' CLAUDE.md docs/operations/`) was sufficient to
catch all three instances this session; a recurring lint cron would
catch the NEXT addition before it spreads further. Filed as conceptual
follow-up; this task's prevention is "PL-206 already logged, sweep
confirmed all three instances fixed."

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

### 2026-06-10 — Third instance, same session, single-pass sweep
- **What changed:** After T-2129 fixed the substrate-orchestrator-recipe and T-2130 fixed agent-find-idle.md, a sweep of CLAUDE.md found two more instances of the same drift. No new class, no surprise. The sweep also confirmed the skill set (`.claude/commands/`) and other ops docs are clean.
- **Plan impact:** Same scope as T-2130 — fix the two lines, run the verification grep, close.
- **Triggered:** Filed conceptual follow-up in RCA prevention block: a doc-prose field-name lint cron is the right level of prevention since this is the third instance in one session. Not filed as a task — the lint script would itself be PL-206 mitigation layer (a), which the PL-206 entry already names. If a fourth instance appears, file the lint task immediately.

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

### 2026-06-10T16:31:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2131-fix-claudemd-claimedby--claimer-pl-206-c.md
- **Context:** Initial task creation
