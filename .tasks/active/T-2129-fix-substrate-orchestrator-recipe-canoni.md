---
id: T-2129
name: "fix substrate-orchestrator-recipe canonical worker pattern bugs"
description: >
  fix substrate-orchestrator-recipe canonical worker pattern bugs

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, doc, bug]
components: []
related_tasks: [T-2124, T-2127]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T16:21:39Z
last_update: 2026-06-10T16:21:39Z
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

# T-2129: fix substrate-orchestrator-recipe canonical worker pattern bugs

## Context

`docs/operations/substrate-orchestrator-recipe.md` (T-2124, the AEF integration
master walkthrough) has three concrete drifts in the **Canonical worker
pattern** code block that silently break the pattern for any developer
copy-pasting it. Found while auditing T-2018 closure work in S-2026-0610-1804+1.

### Bugs

1. **L208** — `dm_payload="$(termlink agent dms --watch --limit 1 | jq -r '.payload')"`
   - `agent dms --watch` is documented "Incompatible with `--json`" (CLI
     help, T-1559); using it in a pipeline-to-jq fails immediately.
   - No `--limit` flag exists on `agent dms` (verified via `--help`).
   - **Impact:** Step 2 of the worker loop fails on first iteration; the
     worker never receives an assigned DM.

2. **L219** — `.claims[] | select(.claim_id==…) | .claimed_by`
   - The hub's `channel.claims` JSON envelope uses field `claimer`, NOT
     `claimed_by` (verified at `crates/termlink-hub/src/channel.rs:1377`
     and `crates/termlink-bus/src/claim.rs:22`).
   - **Impact:** Step 4 (ownership defence-in-depth) silently extracts
     null; the comparison `[ "$current_owner" != "$WORKER_ID" ]` always
     evaluates true → worker `continue`s on every iteration → no work
     is ever processed.

3. **L571** — "T-2111..T-2117 — substrate status (substrate primitive #11) build chain"
   - T-2127 fix-up corrected line 546 but missed line 571. SUBSTRATE-PULSE
     is a COMPOSITION; T-2026 reserves `#11` for typed agent-launch
     (un-partitionable-file handling per the inception report).
   - **Impact:** Doc-internal inconsistency confuses readers about the
     §6 manifest's actual slot count.

### Why these matter

The canonical worker pattern is the SHIPPED contract that an AEF
integration developer copy-pastes to start their worker loop. Bugs 1
and 2 are not cosmetic — they make the worker silently fail (bug 1
crashes; bug 2 silently no-ops). This is direct user-value damage on
T-2018's headline deliverable.

## Acceptance Criteria

### Agent
- [x] L208 fix: replace broken `agent dms --watch --limit 1` with a `--json`-compatible poll pattern that surfaces an unread DM payload (e.g. `agent inbox --json` to discover unread `dm:*` topics, then `channel subscribe --json --resume --limit 1` to read the next message)
- [x] L219 fix: replace `.claimed_by` with `.claimer` so the ownership defence-in-depth actually compares the correct field
- [x] L571 fix: relabel "T-2111..T-2117 — substrate status (substrate primitive #11) build chain" to match the corrected line 546 framing ("composition, not §6 manifest primitive")
- [x] No regression to other code-block snippets (L147-L153 orchestrator pattern uses `channel claims-summary` + `channel claim` correctly — leave alone)
- [x] Recipe still renders sensibly as a self-contained worker pattern after the L208 rewrite (no dangling references)

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

# Verify the L208 fix: no `agent dms --watch` invocation in worker pattern body.
out=$(grep -nE 'agent dms --watch' docs/operations/substrate-orchestrator-recipe.md 2>&1); [ -z "$out" ]
# Verify the L219 fix: the jq selector uses .claimer (not .claimed_by) for the
# extraction. The explanatory comment may still reference .claimed_by to document
# the prior bug — that's intentional; scope the check to executable jq lines.
out=$(grep -nE 'jq -r ".*select.*claim_id.*claimed_by' docs/operations/substrate-orchestrator-recipe.md 2>&1); [ -z "$out" ]
# Verify the L571 fix: no remaining "substrate primitive #11" framing.
out=$(grep -nE 'substrate primitive #11' docs/operations/substrate-orchestrator-recipe.md 2>&1); [ -z "$out" ]
# Verify the canonical worker pattern still references `.claimer` after the fix.
grep -q '\.claimer' docs/operations/substrate-orchestrator-recipe.md
# Verify the L208 replacement uses a real CLI verb.
grep -qE 'channel subscribe.*--json|agent inbox.*--json' docs/operations/substrate-orchestrator-recipe.md
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

### 2026-06-10T16:21:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2129-fix-substrate-orchestrator-recipe-canoni.md
- **Context:** Initial task creation
