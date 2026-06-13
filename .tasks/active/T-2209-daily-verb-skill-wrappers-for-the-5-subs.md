---
id: T-2209
name: "Daily-verb skill wrappers for the 5 substrate history/retrospective verbs"
description: >
  Add .claude/commands/ skill wrappers for claims-history, find-idle-history, queue-history, governor-history, substrate-history — completing the daily-verb skill layer (base verbs shipped as /claims, /find-idle, /queue-status, /governor, /substrate under T-2092..T-2096). CLI+MCP tiers already exist (T-2074/2081/2086/2068/2111); only the skill tier is missing, breaking the established pattern for operators investigating flaps.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [substrate, skill, observability]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T10:34:40Z
last_update: 2026-06-13T10:46:41Z
date_finished: 2026-06-13T10:46:41Z
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

# T-2209: Daily-verb skill wrappers for the 5 substrate history/retrospective verbs

## Context

The substrate observability arc shipped each read-side primitive across three
tiers — CLI, MCP, and a `.claude/commands/*.md` daily-verb skill wrapper. The
base read verbs all have skills: `/find-idle` (T-2092), `/claims` (T-2093),
`/queue-status` (T-2094), `/governor` (T-2095), `/cv-keys` (T-2121), and the
composite `/substrate` (T-2096). Each arc also has a **retrospective/history**
verb (`channel claims-history` T-2074, `agent find-idle-history` T-2081,
`channel queue-history` T-2086, `fleet governor-history` T-2068,
`substrate history` T-2111) with CLI + MCP tiers — but **no skill wrapper**.

This breaks the established pattern: an operator who finds a wedge with
`/claims --all --only-stuck` has no `/claims-history` to answer "first time or
Nth?" without dropping to raw CLI. This task closes that asymmetry by adding the
five missing skill wrappers, mirroring the structure of the base-verb skills.

Surface confirmed present (binary 0.11.1230): all five `*-history` CLI verbs
respond to `--help` with the documented `--since`/`--json` + per-verb filter
flags. Work is confined to `.claude/commands/` (read-only wrappers; no source
or hub changes).

## Acceptance Criteria

### Agent
- [x] `.claude/commands/claims-history.md` exists, wraps `termlink channel claims-history`, documents `--since`/`--topic`/`--log`/`--json`, and follows the base-verb skill structure (pre-flight, parse, run, render, empty-result hint, rules, related).
- [x] `.claude/commands/find-idle-history.md` exists, wraps `termlink agent find-idle-history` (`--since`/`--agent-id`/`--log`/`--json`).
- [x] `.claude/commands/queue-history.md` exists, wraps `termlink channel queue-history` (`--since`/`--kind`/`--log`/`--json`).
- [x] `.claude/commands/governor-history.md` exists, wraps `termlink fleet governor-history` (`--since`/`--hub`/`--log`/`--json`).
- [x] `.claude/commands/substrate-history.md` exists, wraps `termlink substrate history` (`--since`/`--field`/`--log`/`--json`).
- [x] Each skill cross-references its base-verb sibling skill and its CLI/MCP task IDs in a `## Related` section.
- [x] Each skill's documented command string matches the verb's real `--help` flags (no invented flags) — verified against the live binary. Smoke-tested 2026-06-13: all five verbs exit 0 and emit the documented missing-log hint, matching each skill's Step 5.

### Human
- [ ] [REVIEW] The five history skills read naturally and are discoverable alongside their base-verb siblings.
  **Steps:**
  1. In a Claude Code session on this host, type `/` and confirm `claims-history`, `find-idle-history`, `queue-history`, `governor-history`, `substrate-history` appear in the skill list.
  2. Open one (e.g. `.claude/commands/claims-history.md`) and read the Invocation table + Step 5 empty-result hints.
  3. Optionally invoke `/claims-history` and confirm the rendered output + missing-log hint reads sensibly.
  **Expected:** Each skill is listed, the pairing with its live sibling (`/claims` ↔ `/claims-history`) is obvious, and the empty-result hint correctly points back at the `--watch --log` writer.
  **If not:** Note which skill reads awkwardly or has a wrong pointer; the fix is a wording edit in the corresponding `.claude/commands/*.md` (no code change).

## Recommendation

**Recommendation:** GO

**Rationale:** This is a pattern-completion task with zero code/hub risk —
five read-only `.claude/commands/*.md` skill wrappers that close a tier
asymmetry (the base read verbs all had skills; their `*-history` retrospective
siblings did not). Each skill mirrors the established base-verb skill structure
(pre-flight → parse → run → render → empty-result hint → rules → related) and
was verified against the live binary. The only open item is a non-blocking
human taste check on wording/discoverability.

**Evidence:**
- 5 skill files created: `.claude/commands/{claims,find-idle,queue,governor,substrate}-history.md`.
- 10/10 verification commands PASS (file-exists + correct-verb-name for each).
- Smoke-test 2026-06-13: all five underlying CLI verbs (`channel claims-history`,
  `agent find-idle-history`, `channel queue-history`, `fleet governor-history`,
  `substrate history`) exit 0 and emit the documented missing-log hint — exactly
  the empty-result behavior each skill's Step 5 describes.
- No source or hub changes; binary unaffected.
- Sibling base-verb skills shipped under T-2092..T-2096 / T-2121; CLI+MCP
  history tiers under T-2074/2081/2086/2068/2111 + their MCP twins.

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

test -f .claude/commands/claims-history.md
test -f .claude/commands/find-idle-history.md
test -f .claude/commands/queue-history.md
test -f .claude/commands/governor-history.md
test -f .claude/commands/substrate-history.md
# Each skill names the exact CLI verb it wraps (no invented verb)
grep -q "termlink channel claims-history" .claude/commands/claims-history.md
grep -q "termlink agent find-idle-history" .claude/commands/find-idle-history.md
grep -q "termlink channel queue-history" .claude/commands/queue-history.md
grep -q "termlink fleet governor-history" .claude/commands/governor-history.md
grep -q "termlink substrate history" .claude/commands/substrate-history.md

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

### 2026-06-13T10:34:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/T-2209-history-skills/.tasks/active/T-2209-daily-verb-skill-wrappers-for-the-5-subs.md
- **Context:** Initial task creation

### 2026-06-13T10:46:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
