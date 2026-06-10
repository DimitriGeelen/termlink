---
id: T-2134
name: "Fix canonical orchestrator pattern — next_free_offset is fictional"
description: >
  Fix canonical orchestrator pattern — next_free_offset is fictional

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [T-2018, substrate, pl-206, docs-fix]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T17:22:46Z
last_update: 2026-06-10T17:26:30Z
date_finished: 2026-06-10T17:26:30Z
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

# T-2134: Fix canonical orchestrator pattern — next_free_offset is fictional

## Context

T-2129 fixed the canonical WORKER pattern in `docs/operations/substrate-orchestrator-recipe.md`
(L208 broken DM-poll, L219 `.claimed_by` → `.claimer`). T-2133's layer-c
lint then validated all fenced-bash `termlink` verb invocations clean.

Scoping the master recipe more closely (T-2134) surfaced a **third**
class of doc-CLI drift in the **canonical ORCHESTRATOR pattern**: Step 2
references `claims-summary.next_free_offset` — a JSON field that does
NOT exist in `ClaimsSummary` (verified at `crates/termlink-bus/src/claim.rs:104`).
The actual fields are `active_count`, `expired_count`, `oldest_active_at_ms`,
`oldest_active_age_ms`, `next_active_expiry_ms`. There is no derived
"next free offset" surface; orchestrators must use a different pattern.

This is the same drift class PL-206 covers but in JSON field names
rather than CLI verb names — layer-c lint catches verbs, not field
references. T-2129 surfaced the worker variant (`.claimed_by`); this
task is the orchestrator variant.

The fix replaces the broken Step-2 derivation with the substrate's
canonical "process each envelope once" primitive: `channel subscribe
--resume`. The orchestrator streams the work topic and tries to claim
each envelope as it arrives. CLAIM_CONFLICT (-32015) means already-claimed
OR previously completed — both are skip-and-move-on. Cleaner mental
model AND a correct implementation against shipped substrate.

## Acceptance Criteria

### Agent
- [x] `next_free_offset` no longer appears anywhere in `docs/operations/substrate-orchestrator-recipe.md`
- [x] Canonical orchestrator pattern rewritten to use `channel subscribe --resume` stream-based dispatch
- [x] Rewritten pattern uses ONLY verbs verified against source: `channel subscribe`, `channel claim`, `channel claim-transfer`, `agent find-idle`, `agent contact`
- [x] "Why this pattern is correct" prose updated to reflect the stream-based correctness argument
- [x] PL-206 layer-c lint reports clean on the modified doc
- [x] PL-206 layer-b lint reports clean on the modified doc
- [x] Grep `next_free_offset` across all docs/ returns no matches

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

out=$(grep -n next_free_offset docs/operations/substrate-orchestrator-recipe.md 2>&1); test -z "$out"
out=$(grep -rn next_free_offset docs/ 2>&1); test -z "$out"
out=$(grep -n "channel subscribe.*--resume" docs/operations/substrate-orchestrator-recipe.md 2>&1); test -n "$out"
out=$(TERMLINK_BIN=target/debug/termlink bash scripts/lint-doc-fenced-bash.sh 2>&1); echo "$out" | grep -qE 'Status: clean|no drift found'
out=$(bash scripts/lint-doc-cli-references.sh 2>&1); echo "$out" | grep -qE 'Status: clean|no drift found'

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

**Symptom:** The canonical orchestrator pattern in the master
substrate AEF integration recipe referenced
`claims-summary.next_free_offset` — a JSON field that doesn't exist
in the response envelope. Every AEF integration developer copy-pasting
the orchestrator pattern would get a script that silently does nothing:
`next_offset` evaluates to `null`, the `if [ "$next_offset" = "null" ]`
branch always fires, the loop sleeps forever. No work is dispatched.

**Root cause:** The doc was written aspirationally — the author reached
for a hub-derived "next free offset" surface that would have been
convenient but was never built. The actual `ClaimsSummary` (verified at
`crates/termlink-bus/src/claim.rs:104`) exposes aggregate counters
(`active_count`, `expired_count`, `oldest_active_age_ms`, etc.) but no
specific-offset hint. Computing "next free offset" generically requires
either client-side bookkeeping (watermark + state file) or a stream-based
approach (`channel subscribe --resume`). The doc reached past the
shipped substrate's actual surface.

**Why structurally allowed:** PL-206 layer-c lint (T-2133) validates
CLI VERB existence in fenced bash but not JSON FIELD references. The
`next_free_offset` reference is in a jq expression (`jq -r '.next_free_offset'`)
that the lint sees only as "jq" — it has no awareness of which fields
the upstream `termlink ... --json` envelope actually contains.

**Prevention:** Three layers, only one applicable today:
- **(applicable now)** Stream-based pattern (`channel subscribe
  --resume`) avoids the "compute next free" trap entirely — the
  substrate's own primitive gives "what's new" for free, so the
  orchestrator never has to derive it. The rewrite makes this the
  canonical pattern, removing the temptation to invent more derived
  fields.
- **(future layer-c2 lint, deferred)** Extend the doc-CLI lint to also
  validate JSON field references against the actual response struct.
  For `termlink <verb> --json | jq '.<field>'`, run a small jq query
  against a known-good envelope and assert the field exists. Would
  catch this class structurally.
- **(future doc-smoke harness, T-2133 follow-up)** Layer (c2) of the
  PL-206 arc: actually execute fenced bash blocks against a throwaway
  hub. Would have caught this instantly — the pattern would have
  never dispatched work in the harness.

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

### 2026-06-10T17:22:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2134-fix-canonical-orchestrator-pattern--next.md
- **Context:** Initial task creation

### 2026-06-10T17:26:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
