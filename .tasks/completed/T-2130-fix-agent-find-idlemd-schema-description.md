---
id: T-2130
name: "fix agent-find-idle.md schema description: claims.claimed_by → claims.claimer (PL-206 class instance)"
description: >
  fix agent-find-idle.md schema description: claims.claimed_by → claims.claimer (PL-206 class instance)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-parallel-substrate, doc, bug]
components: []
related_tasks: [T-2129, T-2124]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T16:29:28Z
last_update: 2026-06-10T16:30:41Z
date_finished: 2026-06-10T16:30:41Z
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

# T-2130: fix agent-find-idle.md schema description: claims.claimed_by → claims.claimer (PL-206 class instance)

## Context

While sweeping `docs/operations/` for other instances of T-2129's class
(PL-206 — doc-CLI drift), found two more in `agent-find-idle.md` that
describe the find-idle derivation in pseudo-relational syntax:

```
L23: idle_agents = LIVE(agent-presence) \ DISTINCT(claims.claimed_by)
L29: - `DISTINCT(claims.claimed_by)` = every agent that currently holds at least
```

The hub's `claims` table column / `ClaimInfo` field is `claimer`, NOT
`claimed_by` (`crates/termlink-bus/src/claim.rs:22`,
`crates/termlink-hub/src/channel.rs:1377`). A reader of agent-find-idle.md
who searches the source for `claimed_by` finds nothing; a reader inspired
to write a `jq` query against `channel.claims --json` based on this
pseudo-syntax hits the exact same null-extraction bug T-2129 just fixed.

Same root-cause class as T-2129 (PL-206); separate file → separate task
per "one bug = one task" (CLAUDE.md §Task Sizing Rules).

## Acceptance Criteria

### Agent
- [x] L23: replace `claims.claimed_by` with `claims.claimer` in the derivation pseudo-code
- [x] L29: replace `claims.claimed_by` with `claims.claimer` in the prose bullet
- [x] No other `claimed_by` references in `docs/operations/agent-find-idle.md` after the fix

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

# No claims.claimed_by references remain in agent-find-idle.md.
out=$(grep -nE 'claims\.claimed_by' docs/operations/agent-find-idle.md 2>&1); [ -z "$out" ]
# claims.claimer references are present.
grep -q 'claims\.claimer' docs/operations/agent-find-idle.md
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

**Symptom:** `docs/operations/agent-find-idle.md` lines 23 and 29 described the
find-idle anti-join as `claims.claimed_by`, but the hub's `ClaimInfo` struct
and serialized JSON envelope both use `claimer`. A reader searching the
source for `claimed_by` finds nothing; a reader writing their own derivation
based on this pseudo-syntax hits the same null-extraction bug T-2129 just
patched in the orchestrator recipe.

**Root cause:** Same as T-2129 — doc-prose authored against an assumed
schema rather than the live serialization. Specific to this doc: the
pseudo-relational syntax (`claims.claimed_by`) reads like a real database
column reference, so a reader is more likely to believe and propagate it
than they would a free-form description.

**Why structurally allowed:** No lint asserts that field/column names in
doc pseudo-code match the live serialization in Rust source. The
markdown-as-inert-text pattern from PL-206 (T-2129 RCA) applies equally
to schema descriptions and CLI invocations.

**Prevention:** Same as T-2129's PL-206 — the lint script proposed there
would catch CLI verbs against `termlink help` but NOT schema-field
references in prose. A complementary lint or doc-test would extract every
`field-name` pattern in pseudo-code blocks under `## Derivation` headings
and grep the `crates/` source for it. Filed as conceptual follow-up;
this task's prevention is "same class as T-2129 → PL-206 already logged."

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

### 2026-06-10 — Class instance, not surprise
- **What changed:** T-2129 surfaced PL-206 (doc-CLI drift); the post-fix sweep of `docs/operations/` for the two specific patterns (`claimed_by`, `agent dms --watch`) found two more `claimed_by` references in `agent-find-idle.md`. Not a new class — same instance, separate file.
- **Plan impact:** None — scope held to "fix the two lines, run the same verification grep, close."
- **Triggered:** No new tasks. The sweep also confirmed (`substrate primitive #11`, `agent dms --watch`, watch+jq pipes) are otherwise clean across all 7 substrate ops docs.

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

### 2026-06-10T16:29:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2130-fix-agent-find-idlemd-schema-description.md
- **Context:** Initial task creation

### 2026-06-10T16:30:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
