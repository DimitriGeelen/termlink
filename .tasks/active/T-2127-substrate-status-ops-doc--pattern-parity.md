---
id: T-2127
name: "substrate-status ops doc — pattern parity with other primitives (T-2018 §6 #11 doc closure)"
description: >
  substrate-status ops doc — pattern parity with other primitives (T-2018 §6 #11 doc closure)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, substrate-primitive-11, docs]
components: [docs/operations]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T15:35:24Z
last_update: 2026-06-10T15:35:24Z
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

# T-2127: substrate-status ops doc — pattern parity with other primitives (T-2018 §6 #11 doc closure)

## Context

Every shipped T-2018 §6 substrate primitive has a dedicated `docs/operations/substrate-*.md` ops doc — except #11 (substrate-status / SUBSTRATE-PULSE), which just shipped (T-2111..T-2117). Operators wanting to use `termlink substrate status --watch --notify --log`, `termlink substrate history`, `termlink_substrate_status` (MCP), or `termlink_substrate_history` (MCP) have to piece the recipe together from the CLAUDE.md row, the master orchestrator-recipe, and the closed task files.

Pattern-parity gap: 5 substrate primitives have a dedicated ops doc, 1 does not.

| Primitive | Dedicated doc | Master recipe xref |
|---|---|---|
| #1 CLAIM | substrate-claim-primitive.md | ✓ |
| #5 RESILIENCE (queue) | substrate-offline-queue-recipe.md | ✓ |
| #5 RESILIENCE (idempotency) | substrate-post-idempotency.md | ✓ |
| #9 BROADCAST-WITH-REPLAY | substrate-broadcast-with-replay.md | ✓ |
| #10 BACKPRESSURE | substrate-governor.md | ✓ |
| #11 SUBSTRATE-PULSE | **MISSING** | partial mention only |

This task ships `docs/operations/substrate-status.md` following the same template as the other 5 docs: scope, mental model, observability arc walkthrough (one-shot → watch → notify → log → history-CLI → history-MCP → status-MCP), full operator recipes per arc step, common patterns, what it does NOT do, related primitives, related tasks.

## Acceptance Criteria

### Agent
- [x] `docs/operations/substrate-status.md` created with sections: Scope | Mental model | The wire shape (`termlink substrate status` CLI + JSON envelope) | Observability arc walkthrough | Common patterns | What this does NOT do | Related
- [x] Walks all 7 surfaces: one-shot CLI, `--watch`, `--notify`, `--log`, `substrate history` CLI, `termlink_substrate_status` MCP, `termlink_substrate_history` MCP — each with at least one copy-pasteable recipe
- [x] Cross-references back to the master integration recipe (`substrate-orchestrator-recipe.md`) and forward to the four sub-verb daily docs (claims-summary, find-idle, queue-status, governor-status)
- [x] Master recipe (`substrate-orchestrator-recipe.md`) gets a bullet at the top of its Related section pointing at the new doc (pattern parity with other primitives)
- [x] CLAUDE.md Quick Reference row exists or updated — substrate status CLI arc + history CLI/MCP visible alongside `/substrate` skill row (deferred — the existing `/substrate` row already names the CLI verbs by name; a fresh row would be redundant noise; leaving as docs-only)
- [x] Word count ≥1500, ≤4000 (sized to match other substrate ops docs; T-2120 broadcast-with-replay is ~2200, governor is ~2800) — substrate-status.md is 1559 words

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

test -f docs/operations/substrate-status.md
out=$(wc -w docs/operations/substrate-status.md); n=$(echo "$out" | awk '{print $1}'); [ "$n" -ge 1500 ] && [ "$n" -le 4000 ]
grep -q "termlink substrate status" docs/operations/substrate-status.md
grep -q "termlink substrate history" docs/operations/substrate-status.md
grep -q "termlink_substrate_status" docs/operations/substrate-status.md
grep -q "termlink_substrate_history" docs/operations/substrate-status.md
grep -q "\-\-watch" docs/operations/substrate-status.md
grep -q "\-\-notify" docs/operations/substrate-status.md
grep -q "\-\-log" docs/operations/substrate-status.md
grep -q "substrate-status.md" docs/operations/substrate-orchestrator-recipe.md

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

### 2026-06-10 — filing
- **What changed:** Realized substrate-status (#11) is the lone primitive without a dedicated ops doc. The master orchestrator-recipe references it but does not walk the seven-surface arc.
- **Plan impact:** Adds one doc + one xref in the master recipe + (optionally) a CLAUDE.md row touch-up; ships single-deliverable.
- **Triggered:** No new sub-tasks expected.

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

### 2026-06-10T15:35:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2127-substrate-status-ops-doc--pattern-parity.md
- **Context:** Initial task creation
