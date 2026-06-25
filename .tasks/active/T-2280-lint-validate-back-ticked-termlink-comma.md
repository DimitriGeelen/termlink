---
id: T-2280
name: "Lint: validate back-ticked termlink command hints against the clap command tree"
description: >
  Prevention for T-2279/PL-230: a build-time check that extracts back-ticked `termlink <group> <verb>` strings from source and verifies each names a real clap subcommand. Catches hints that point users at non-existent commands (e.g. the `agent listeners --fleet` bug). Backlog — the agent group is currently audited clean.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-24T21:06:21Z
last_update: 2026-06-25T06:51:57Z
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

# T-2280: Lint: validate back-ticked termlink command hints against the clap command tree

## Context

Prevention for T-2279 / PL-230: the T-2275 no-match error hint pointed users at
`termlink agent listeners --fleet`, which is not a real clap subcommand. CLI hint
strings naming `termlink <group> <verb>` commands are never validated against the
actual command tree, so a typo'd or stale hint ships silently and sends users to
an "unrecognized subcommand" dead end. This task adds a static lint that extracts
back-ticked `termlink <group> <verb>` strings from source and verifies each names
a real command, catching the bug class at build/audit time.

## Acceptance Criteria

### Agent
- [x] `scripts/lint-command-hints.sh` exists: walks the live clap command tree
      (`termlink --help` + per-group `termlink <group> --help`) to build the set
      of valid `group verb` paths, extracts back-ticked `termlink <group> <verb>`
      hints from `crates/termlink-cli/src` + `crates/termlink-mcp/src`, and reports
      every hint whose group is real but verb is NOT a subcommand of that group.
- [x] Exit codes: 0 = no bad hints, 1 = one or more invalid hints found (with
      file:line + the offending hint + nearest-match suggestion), 2 = tooling
      error (binary not found / help unparseable). `--json` emits a scriptable
      envelope.
- [x] The lint passes clean on the current tree — after fixing the 3 *additional*
      offenders it found (see Evolution). Comment lines are excluded and
      `termlink help <cmd>` is special-cased so no false positives remain.
- [x] A self-test proves the lint CATCHES a known-bad hint: a fixture line
      containing `` `termlink agent listeners` `` is flagged (proves no false
      negatives).
- [x] Wired into CI (`.github/workflows/install-check.yml`, the job that already
      builds + installs the binary the lint needs) so it is not dormant tooling
      (PL-168).

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
test -x scripts/lint-command-hints.sh
bash scripts/lint-command-hints.sh
bash scripts/lint-command-hints.sh --self-test

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

### 2026-06-25 — the lint immediately found 3 more live offenders

- **What changed:** filing assumed the tree was already clean (T-2279 removed the
  one *known* bad hint). On first run the lint flagged 3 additional genuine
  user-facing offenders that had shipped silently:
  1. `crates/termlink-cli/src/commands/channel.rs:273` — an error message told
     users to `termlink fleet profile add`; `fleet` has no `profile` subcommand.
     Fixed → `termlink remote profile add` (the real command).
  2. `crates/termlink-mcp/src/tools.rs:20140` — MCP tool description cited
     `termlink channel typing-list <topic>`; the real verb is `channel typing`.
  3. `crates/termlink-mcp/src/tools.rs:20180` — cited `termlink channel
     typing-emit <topic>`; real verb is `channel typing --emit` (plus a second
     `channel typing-list` prose reference on the same line).
  This is the T-2279/PL-230 class recurring exactly as predicted — the lint paid
  for itself on first run.
- **Plan impact:** added a 5th AC (CI wiring) — a lint nobody runs is dormant
  tooling (PL-168). Scoped the lint to user-facing strings (excludes `//`/`///`
  comment lines, which carry deliberate typo-examples and `help <cmd>` notes) and
  special-cased `termlink help <cmd>` (valid for any real command). These removed
  4 comment-only false positives, leaving only the 3 real bugs.
- **Triggered:** no new tasks; the 3 fixes landed under this task.
- **Gotcha captured:** the validator first mis-classified every group as unknown
  because the build array was named `GROUPS` — a *reserved bash variable* (the
  user's supplementary GIDs). Assigning to it is coerced, so `$GROUPS` expanded to
  a gid. Renamed `TL_GROUPS`. (Candidate learning if it recurs.)

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

### 2026-06-24T21:06:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2280-lint-validate-back-ticked-termlink-comma.md
- **Context:** Initial task creation

### 2026-06-25T06:51:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
