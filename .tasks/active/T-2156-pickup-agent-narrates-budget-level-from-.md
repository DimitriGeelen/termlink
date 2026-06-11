---
id: T-2156
name: "Pickup: Agent narrates budget level from historical tool-result JSON in system-reminders, not .context/working/.budget-status canonical cache — reproduced this turn (claimed 273K when actual was 159K) (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-2155. Type: bug-report.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [pickup, bug-report]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-11T10:34:27Z
last_update: 2026-06-11T19:53:42Z
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
source_task_id_in_origin: T-2155
source_project_in_origin: "termlink"
---

# T-2156: Pickup: Agent narrates budget level from historical tool-result JSON in system-reminders, not .context/working/.budget-status canonical cache — reproduced this turn (claimed 273K when actual was 159K) (from termlink)

## Context

T-2155 RCA GO outcome — ship Option A (defence-in-depth): extend the
`/resume` skill to read `.context/working/.budget-status` (the canonical
budget cache) so the agent grounds budget claims against current state,
not against historical tool-result JSON re-injected as system-reminders.

Two surfaces drift on `/resume`:
- Live `/root/.claude/commands/resume.md` (userSettings, what actually runs)
- Vendored `/opt/termlink/.claude/commands/resume.md` (source-of-truth
  for vendor/install — currently more advanced re: watchtower URL)

Both must converge. Option B (SessionStart:compact hook) is framework-side
and stays open for framework-agent pickup — out of scope here.

## Acceptance Criteria

### Agent
- [x] Live userSettings `/resume` skill (`/root/.claude/commands/resume.md`) reads `.context/working/.budget-status` as part of Step 1 gather phase.
- [x] Live userSettings `/resume` skill Step 2 summary template includes a Budget line sourced from the cache (level + tokens).
- [x] Vendored `/opt/termlink/.claude/commands/resume.md` carries the same budget-cache read + summary line so future installs propagate the fix.
- [x] Both files name `.context/working/.budget-status` literally so a future audit (grep) can detect drift.
- [x] Note in task body that Option B (SessionStart:compact hook prepend) remains framework-agent pickup.

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
grep -q "budget-status" /root/.claude/commands/resume.md
grep -q "budget-status" /opt/termlink/.claude/commands/resume.md
grep -q "Budget:" /root/.claude/commands/resume.md
grep -q "Budget:" /opt/termlink/.claude/commands/resume.md

## RCA

**Symptom:** Agent reads budget level/tokens from system-reminder JSON echoing a prior session's Read of an ephemeral Task tool output file (e.g. `/tmp/claude-0/.../tasks/<id>.output`). Same key names (`level`/`tokens`/`timestamp`/`source`) as the canonical cache make it indistinguishable from a current read. One reproduced instance: agent narrated `level=urgent, tokens=273016` for an entire session when actual cache was `level=ok, tokens=159350` — ~140K of real headroom unused.

**Root cause:** The `/resume` skill's Step 1 gather phase reads handover + git + tasks + tool counter + web server — NOT `.context/working/.budget-status`. Nothing in the skill forces a grounded read of the canonical cache, so the agent satisfies "report budget" from whatever budget-shaped JSON it finds in context.

**Why structurally allowed:** Hook-side budget enforcement (PreToolUse `budget-gate.sh`) is pull-only — it writes the cache but never re-asserts level into agent context. The /resume skill is the next-best surface (runs at every resume) and missed naming the cache file. CLAUDE.md "After context compaction" names `fw resume status` + `fw resume sync` but not the cache path.

**Prevention:** This task (Option A) — extend /resume to read `.context/working/.budget-status` literally + render it in summary. Option B (SessionStart:compact hook prepends current budget) remains framework-agent pickup — would force-feed ground truth at the system-reminder layer, defeating the misread at the source.

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

### 2026-06-11T10:34:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2156-pickup-agent-narrates-budget-level-from-.md
- **Context:** Initial task creation

### 2026-06-11T19:53:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
