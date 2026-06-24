---
id: T-2279
name: "Fix agent contact no-match hint references non-existent agent listeners subcommand"
description: >
  Fix agent contact no-match hint references non-existent agent listeners subcommand

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/agent.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-24T20:48:34Z
last_update: 2026-06-24T21:01:24Z
date_finished: 2026-06-24T21:01:24Z
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

# T-2279: Fix agent contact no-match hint references non-existent agent listeners subcommand

## Context

T-2275 added a fleet no-match error in `cmd_agent_contact` (agent.rs:904) whose
hint tells the user to "Run `termlink agent listeners --fleet`" — but there is
**no `agent listeners` subcommand** (the real verbs are `contact` / `find-idle` /
`who` / `dms` / `listen`; clap rejects `listeners` with "unrecognized
subcommand"). The hint points at a command that does not exist, violating the
Usability directive (actionable errors). Correct pointers: `termlink agent
find-idle` (real CLI verb for the LIVE roster) and `/peers` (skill listing LIVE
peers). The MCP-side sibling hint (`termlink_agent_listeners_fleet`) is a REAL
MCP tool — only the CLI string is broken.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] agent.rs:904 no-match hint no longer references the non-existent `agent listeners` subcommand; it points at `termlink agent find-idle` (real CLI verb) and `/peers` (skill), plus the existing `--target-fp` fallback.
- [x] No runnable-command source string references `agent listeners` (space-form) — the only remaining `agent-listeners` reference is the hyphenated script name `agent-listeners-fleet.sh` in a comment (legitimate).
- [x] `cargo check -p termlink` clean; binary rebuilt + installed; `termlink --version` matches VERSION; `strings` on the installed binary shows the corrected hint and NOT `agent listeners --fleet`.

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

cargo check -p termlink
out=$(grep -rn 'agent listeners' crates/termlink-cli/src/commands/agent.rs); [ -z "$out" ]

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

**Symptom:** `termlink agent contact <unknown-peer>` (no local + no fleet match)
emits a hint telling the user to "Run `termlink agent listeners --fleet`" — a
subcommand that does not exist; clap rejects it with "unrecognized subcommand
'listeners'".

**Root cause:** When authoring the T-2275 no-match branch the recovery hint was
written from memory of the shell helper name (`agent-listeners-fleet.sh`),
assuming a parallel CLI verb existed. It does not — the CLI exposes the
LIVE/idle roster via `agent find-idle`, and the all-LIVE-peers list lives only
in the `/peers` skill (which wraps the shell script).

**Why structurally allowed:** error-string hints naming other commands are not
validated against the actual clap subcommand set — nothing greps shipped hint
strings for runnable-command accuracy, so a hint can name any command and still
compile.

**Prevention:** AC2 + the Verification `grep` lock the space-form `agent
listeners` out of this file; the corrected hint names only a verb proven to
exist (`agent find-idle`) and a skill (`/peers`). A broader lint (extract
back-ticked `termlink <verb>` strings and check them against the clap command
tree) is noted as a follow-up idea, out of scope for this one-line fix.

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

### 2026-06-24T20:48:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2279-fix-agent-contact-no-match-hint-referenc.md
- **Context:** Initial task creation

### 2026-06-24 — fixed + deployed + runtime-proven
- One-line hint fix at agent.rs:904 (commit `8046ad2d`): `agent listeners
  --fleet` → `agent find-idle` (real verb) + `/peers` skill. `cargo check`
  clean; space-form `agent listeners` grep empty; hyphenated script ref kept.
- Release rebuilt (0.11.23) + installed to `~/.cargo/bin/termlink`. `strings`:
  corrected hint present (1), broken hint absent (0).
- **Runtime-proven:** `termlink agent contact __nomatch_smoke_xyz__ --dry-run`
  on the installed binary now emits the corrected hint — which ALSO confirms the
  T-2275 fleet-walk no-match path executes end-to-end in the shipped binary
  (upgrades T-2275 from compile-verified to runtime-verified for that path).

### 2026-06-24T21:01:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
