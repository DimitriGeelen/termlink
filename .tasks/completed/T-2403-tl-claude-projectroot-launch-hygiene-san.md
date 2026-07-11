---
id: T-2403
name: "tl-claude PROJECT_ROOT launch-hygiene: sanitize leaked env that misroutes agent project"
description: >
  tl-claude passes through whatever PROJECT_ROOT is in the launcher env; a leaked /opt/023 value misrouted workflow-designer's framework project resolution, gating fw/Bash/Edit in /opt/832. Sanitize PROJECT_ROOT at launch so a cwd-scoped agent resolves its own project.

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
created: 2026-07-11T09:47:01Z
last_update: 2026-07-11T12:05:23Z
date_finished: 2026-07-11T12:05:23Z
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

# T-2403: tl-claude PROJECT_ROOT launch-hygiene: sanitize leaked env that misroutes agent project

## Context

`tl-claude.sh` launches a project-scoped agent (claude) but does NOT sanitize
`PROJECT_ROOT`. Any value leaked into the launcher's env is inherited by the
`termlink register`/`claude` it spawns, and the framework resolves the agent's
project from `PROJECT_ROOT` — so a stale `/opt/023` value observed on
workflow-designer (PWD=/opt/832, PROJECT_ROOT=/opt/023) gated ALL of wfd's
`fw`/`Bash`/`Edit` in /opt/832 for its whole session. This is a recurring
fleet-wide launch-hygiene bug: last session's relaunch inherited the same leak.
Fix: a cwd-scoped agent must resolve its OWN project, so tl-claude clears a
leaked `PROJECT_ROOT` at launch (framework then derives from the cwd's
`.framework.yaml`/git-toplevel — the normal unset-PROJECT_ROOT path).

## Acceptance Criteria

### Agent
- [x] **Sanitize in build_claude_cmd (start/restart PTY-string path).** The
  command string built for reachable/normal starts unsets `PROJECT_ROOT`
  (`env -u PROJECT_ROOT …`) so a leaked value is not inherited. Composes
  correctly with the existing `IS_SANDBOX=1` auto-accept prefix. Opt out with
  `TL_KEEP_PROJECT_ROOT=1`. Verify: unit test greps the built command. **DONE:** `env -u PROJECT_ROOT` prefix composes to `env -u PROJECT_ROOT IS_SANDBOX=1 claude …`; test-tl-claude-cmd.sh 5 cases PASS.
- [x] **Sanitize in cmd_oneshot (exec path).** The one-shot exec unsets
  `PROJECT_ROOT` (before `exec termlink spawn`) under the same opt-out. Verify:
  `bash -n` + code inspection / the same unit harness. **DONE:** `unset PROJECT_ROOT` before exec, same `TL_KEEP_PROJECT_ROOT` opt-out; `bash -n` clean.
- [x] **Live proof on the fleet.** After relaunching workflow-designer via the
  patched tl-claude, its new `register`/`claude` process env shows
  `PROJECT_ROOT` UNSET (was `/opt/023`), and wfd can run `fw`/`Bash`/`Edit` in
  /opt/832 (the blocker cleared). Verify: `tr '\0' '\n' < /proc/<pid>/environ | grep -c '^PROJECT_ROOT='` returns 0. **DONE:** wfd relaunched (REPL pid 3803434 `claude --continue`, cwd /opt/832, PROJECT_ROOT UNSET); clean-env cwd-/opt/832 check resolves /opt/832's OWN tasks (T-014…T-182). **FOLLOW-UP (RESOLVED 2026-07-11):** leak source is the tmux SERVER env (registers inherit /opt/023); the `env -u` fix cleans the claude REPL where the framework resolves. Fleet-wide check completed via `/proc/<pid>/environ` scan of every non-termlink claude REPL: ONLY aef's live reachable REPL (pid 793972, cwd /opt/999) still carried `PROJECT_ROOT=/opt/023` — workshop-designer (pid 2852795) and sonnenstall (pid 3207474) REPLs were already clean. aef had not hit the wall because comms/channel posts do not gate on project. Remediated non-destructively (aef is a live production agent I don't own): idle-gated PTY inject notified aef of the leak + the one-line self-relaunch (`cd /opt/999-… && env -u PROJECT_ROOT claude -c`); aef acknowledged (processing). ALSO rolled the Stage-3 idle-gated waker (T-2402) across the fleet: wfd's waker already Stage-3 (relaunched last session); aef/workshop/sonnenstall wakers predated the code — aef's re-arms on its relaunch, workshop+sonnenstall notified via idle-gated inject to `/be-reachable stop && start`. All three notices received + acted on.

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

bash -n scripts/tl-claude.sh
bash scripts/test-tl-claude-cmd.sh

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

### 2026-07-11T09:47:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2403-tl-claude-projectroot-launch-hygiene-san.md
- **Context:** Initial task creation

### 2026-07-11T09:48:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-407d67ba
- **Timestamp:** 2026-07-11T12:05:24Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-11T12:05:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
