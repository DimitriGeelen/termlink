---
id: T-2132
name: "ship PL-206 mitigation layer (b): scripts/lint-doc-cli-references.sh — catch doc-CLI drift before commit"
description: >
  ship PL-206 mitigation layer (b): scripts/lint-doc-cli-references.sh — catch doc-CLI drift before commit

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-parallel-substrate, doc, lint, prevention]
components: []
related_tasks: [T-2129, T-2130, T-2131]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T16:36:12Z
last_update: 2026-06-10T16:38:59Z
date_finished: 2026-06-10T16:38:59Z
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

# T-2132: ship PL-206 mitigation layer (b): scripts/lint-doc-cli-references.sh — catch doc-CLI drift before commit

## Context

This session shipped T-2129/T-2130/T-2131 fixing the same doc-CLI drift
class in three files (recipe doc, agent-find-idle.md, CLAUDE.md). PL-206
captured the class. The PL-206 entry names three prevention layers:
(a) author docs against `--help`, (b) lint script, (c) doc-smoke harness.
Layer (a) is behavioral; (b) is the next concrete deliverable.

This task ships layer (b): `scripts/lint-doc-cli-references.sh` that
sweeps `docs/operations/*.md`, `.claude/commands/*.md`, and `CLAUDE.md`
for the three known instances of the drift class. Non-zero exit when
any hit. Operators run it manually; future work may wire to CI or
`fw doctor`.

**Why this scope, not bigger:** A full CLI-verb-existence lint
(compare `termlink X Y` invocations in docs against `termlink help`)
requires walking the clap structure — substantial. A pattern-grep
script catches the specific instances we've observed and is
immediately useful. Bigger lint deferred until a fourth instance
appears or someone scopes it formally.

## Acceptance Criteria

### Agent
- [x] `scripts/lint-doc-cli-references.sh` exists, is executable, has clear `--help` output naming PL-206
- [x] Script scans `docs/operations/*.md` + `.claude/commands/*.md` + `CLAUDE.md` for the three drift patterns: (i) `claimed_by` as field name, (ii) `agent dms --watch` (incompatible with `--json` per T-1559), (iii) `substrate primitive #11` (SUBSTRATE-PULSE is a composition, not §6 manifest)
- [x] Script exits 0 with affirmative message when clean (after T-2129/T-2130/T-2131 fixes, current repo state IS clean) — confirmed after fixing fourth instance caught by lint itself in substrate-broadcast-with-replay.md:207
- [x] Script exits 1 with per-pattern findings (file:line + which task fixed the precedent) when any pattern matches
- [x] Script handles the legitimate explanatory comment in `substrate-orchestrator-recipe.md:232` (which references `.claimed_by` to document the prior bug) — scope the `claimed_by` regex to executable-position (not arbitrary mention), mirroring T-2129's verification refinement

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

# Script exists and is executable.
test -x scripts/lint-doc-cli-references.sh
# Help text mentions PL-206 explicitly.
out=$(bash scripts/lint-doc-cli-references.sh --help 2>&1); echo "$out" | grep -q "PL-206"
# Clean-repo run exits 0 (after T-2129/T-2130/T-2131 fixes the repo IS clean).
bash scripts/lint-doc-cli-references.sh >/dev/null
# Affirmative-clean message is rendered when no hits.
out=$(bash scripts/lint-doc-cli-references.sh 2>&1); echo "$out" | grep -qE "clean|no findings|PASS"
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

### 2026-06-10 — Lint validated itself on first run
- **What changed:** Built the lint script. First clean-repo run reported `Status: DRIFT FOUND` on `docs/operations/substrate-broadcast-with-replay.md:207` — a fourth `DISTINCT(claimed_by)` instance my earlier T-2131 sweep had missed (different surrounding syntax: `∖` Unicode set-minus vs `\` plain backslash in the other instances).
- **Plan impact:** None on lint design — exactly the prevention behavior intended. Fixed the fourth instance inline as part of T-2132's "clean run" AC, since the lint's exit-0 AC requires a clean repo.
- **Triggered:** Demonstrated layer (b)'s actual value on day one. Captured below; no new task needed (one-line fix, same class, already covered by PL-206).

### 2026-06-10 — Scope refinement: regex anchored to executable positions
- **What changed:** Original draft regex was bare `claimed_by` which would have false-positive on the legitimate explanatory comment in substrate-orchestrator-recipe.md:232 (T-2129's "uses .claimer ... NOT .claimed_by" doc).
- **Plan impact:** Tightened regex to known executable-position patterns: jq selectors (`jq -r ".*select.*claimed_by`), pseudo-relational anti-joins (`DISTINCT(claimed_by`), API-contract prose (`current \`claimed_by\``), schema descriptions (`claims.claimed_by`).
- **Triggered:** No new tasks. If a fifth instance appears with a NEW surrounding syntax, extend the regex; otherwise the four-pattern union covers all currently-observed forms.

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

### 2026-06-10T16:36:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2132-ship-pl-206-mitigation-layer-b-scriptsli.md
- **Context:** Initial task creation

### 2026-06-10T16:38:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
