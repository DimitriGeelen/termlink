---
id: T-2153
name: "substrate-orchestrator-recipe: fix two inline-loop jq bugs (.topics + .payload)"
description: >
  substrate-orchestrator-recipe: fix two inline-loop jq bugs (.topics + .payload)

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
created: 2026-06-11T07:04:06Z
last_update: 2026-06-11T07:06:34Z
date_finished: 2026-06-11T07:06:34Z
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

# T-2153: substrate-orchestrator-recipe: fix two inline-loop jq bugs (.topics + .payload)

## Context

The hand-rolled "Canonical work-stealing worker" loop in
`docs/operations/substrate-orchestrator-recipe.md` has two silent-failure
bugs that would prevent it from ever picking up an orchestrator dispatch:

1. **`.topics[]?` (line ~317)** — `agent inbox --json` returns an array
   directly, not an object with a `.topics` field. The recipe's jq filter
   `.topics[]? | select(...)` would silently yield empty output, leaving
   `next_dm` empty and the worker spinning in `continue`.

2. **`.payload // empty` (line ~284)** — `channel subscribe` returns
   envelopes with the payload base64-encoded in `.payload_b64`, not in
   `.payload`. The recipe would always extract empty `dm_payload`, hit
   `[ -n "$dm_payload" ] || continue`, and skip every envelope.

Both bugs were verified live (2026-06-11) against the local hub:

```
$ termlink agent inbox --json | jq -c 'keys?'  # ⇒ [0,1,2,3] (array, not object)
$ termlink channel subscribe dm:t2152-test:t2152-orch --limit 1 --json \
    | jq -c '.payload, .payload_b64'           # ⇒ null  "Y2xhaW09..."
```

T-2152 ships `substrate-worker-pickup.sh` which uses the correct jq
filters (`.[]?` and `.payload_b64` + base64 -d), so the operator-facing
ship path is clean. This task fixes the customisation reference so
operators who copy-paste the inline loop don't burn an afternoon
debugging silent failures.

## Acceptance Criteria

### Agent
- [x] Inline recipe jq selector at the inbox-poll step is corrected from `.topics[]?` to `.[]?` (the inbox returns an array of `{topic, cursor, latest, unread}` objects). Verified.
- [x] Inline recipe payload extract is corrected from `jq -r '.payload // empty'` to `jq -r '.payload_b64 // empty' | base64 -d` (matches the actual envelope shape). Verified.
- [x] A one-line comment near the corrected lines points operators at `substrate-worker-pickup.sh` as the vetted alternative. Verified — the inbox-step NOTE includes "The vetted version of this whole loop ships as `scripts/substrate-worker-pickup.sh` (T-2152) — prefer that script in production."
- [x] grep the doc to confirm the wrong selectors are removed. Verified — all 3 verification commands return 0.

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

# Confirm the corrected selectors are in place AND the executable line of
# the inline loop block no longer contains the wrong jq path.
# (The fix-note comment may still mention the wrong path in prose — that's
# intentional, so the grep is keyed to the executable code line shape.)
grep -q "jq -r '\\.\\[\\]?" docs/operations/substrate-orchestrator-recipe.md
grep -q "payload_b64 // empty' | base64 -d" docs/operations/substrate-orchestrator-recipe.md
! grep -q "jq -r '\\.topics\\[\\]?" docs/operations/substrate-orchestrator-recipe.md

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

**Symptom:** Inline "Canonical work-stealing worker" loop in the substrate-orchestrator-recipe.md doc would silently fail to pick up dispatches when copy-pasted by an operator. `next_dm` would always be empty (silently — jq `?` swallows the type error from indexing into an array as if it were an object); when staged with manual cursor bootstrap, `dm_payload` would still always be empty because `.payload` doesn't exist on the envelope shape.

**Root cause:** Doc-as-code was authored without verifying the jq selectors against live envelope shapes. Both bugs had the same anti-pattern — the author intuited a field name (`.topics` from "agent inbox shows topics"; `.payload` from "channel post --payload") rather than running a probe to confirm. The actual shapes (`.[]?` for an array-returning RPC; `.payload_b64` per T-1427 envelope canon) were never validated. Bugs survived several edits (T-2148, T-2150, T-2151) because each subsequent edit only added cross-references to other parts of the doc without re-reading the loop block.

**Why structurally allowed:** Inline shell examples in operations docs are not linted, executed, or property-tested by any framework gate. The only path to discovery is an operator copy-pasting and finding it broken. T-2152 (`substrate-worker-pickup.sh`) was authored to ship a vetted version, and during that authoring I probed the live envelope shapes and discovered the doc bugs (the find condition exists for THIS bug: writing a real, executable mirror of the inline example).

**Prevention:** Three layers:
1. **PL-206 reminder reinforced** — when authoring a doc that contains an executable example, treat the example as code: run it against the actual surface before committing.
2. **Cross-link.** The recipe now points operators at `substrate-worker-pickup.sh` first, with NOTEs at the buggy-shape boundary explaining the correct selector — so any future operator who finds yet another shape-bug has a direct comparison to the working script.
3. **Possible future lint:** an audit step that extracts ```bash blocks from docs/operations/*.md and runs shellcheck on them would catch syntax issues (not semantic ones like wrong jq paths, but a partial mitigation). Filed informally — not blocking on this task.

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

### 2026-06-11T07:04:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2153-substrate-orchestrator-recipe-fix-two-in.md
- **Context:** Initial task creation

### 2026-06-11T07:06:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
