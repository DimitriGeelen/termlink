---
id: T-2401
name: "Durable launch-path-independent auto-accept for reachable agents"
description: >
  Make a reachable agent's auto-accept survive ANY relaunch (incl. plain claude --resume), not just tl-claude launches — the settings-based twin of the .mcp.json identity fix (T-2399). Closes the T-2400 residual: a manual relaunch reintroduces reachable-but-mute.

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
created: 2026-07-11T07:28:11Z
last_update: 2026-07-11T07:28:11Z
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

# T-2401: Durable launch-path-independent auto-accept for reachable agents

## Context

T-2399 made per-agent IDENTITY durable & launch-path-independent by baking
`TERMLINK_AGENT_ID` into each project's `.mcp.json` env (Claude Code injects it
into the `mcp serve` it spawns, so the session signs correctly no matter how
claude was started). T-2400 fixed AUTO-ACCEPT but only for `tl-claude`-launched
sessions (it injects `IS_SANDBOX=1` + `--dangerously-skip-permissions`). The
residual gap: a plain `claude --resume` — exactly how the original identity split
happened — comes back up MUTE (no auto-accept), because the flag lives on the
launch command, not in durable project config. So the "reachable-but-mute"
recurrence is NOT structurally closed.

This task makes auto-accept durable the SAME way identity became durable: via
project config that Claude Code reads on every session start, independent of the
launch path. Key mechanism fact (this session): the `/check-arc respond` reply
path posts via **Bash** (`termlink channel post` — scripts/agent-respond.sh:105,
117), NOT the MCP tool — so the permission gate that stalls a manual-mode agent is
the **Bash** tool. The durable allow-list must therefore cover the Bash comms
commands (and MCP equivalents for agents that post via MCP). Sibling to
[[project_comms_loud_contract]]; twin of the T-2399 `.mcp.json` fix.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The durable mechanism is confirmed: verified (via claude-code-guide + a live check) whether `.claude/settings.local.json` `permissions.allow` for the comms Bash/MCP surface (and/or `defaultMode`) is honored by Claude Code from settings ALONE — no CLI `--dangerously-skip-permissions` / `IS_SANDBOX` dependency. The confirmed mechanism + exact settings block is recorded in this task's Updates. CONFIRMED: scoped `permissions.allow` honored from settings alone; `defaultMode: bypassPermissions` IGNORED from project settings (guard since v2.1.142). Block recorded in Updates.
- [x] Each of the 4 live .107 agent projects (/opt/999 aef, /opt/832 workflow-designer, /opt/025 workshop-designer, /opt/3011 sonnenstall) has its `.claude/settings.local.json` updated (via termlink_run, T-559-safe) so the doorbell reply path (`termlink channel post`/`ack`/`subscribe`/`unread` Bash + the MCP equivalents) auto-approves without a prompt. Verify: each file parses as JSON and contains the comms-allow entries (evidence captured in Updates via termlink_run read-back). APPLIED 2026-07-11: all 4 present=4/4 (aef allow_total=350, wfd=38, workshop=85, sonnenstall=7); backed up to `*.pre-t2401`.
- [x] An operator runbook `docs/operations/durable-reachable-auto-accept.md` records the requirement + the exact settings block + the three-part deploy (binary + .mcp.json identity + settings auto-accept), so future onboarding of a reachable agent includes it. Mirrors docs/migrations/T-1700 (identity). Verify: file exists and names all three parts. WRITTEN: names all three legs (binary / identity / auto-accept) + onboarding checklist.

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

## Updates

### 2026-07-11 — mechanism confirmed + per-project audit (pre-implementation)
- **Claude Code settings mechanism (claude-code-guide + docs):** a scoped
  `permissions.allow` entry in project `.claude/settings.local.json` IS honored
  with NO CLI flag and NO `IS_SANDBOX` — auto-approves the matching tool with no
  prompt, survives `claude --resume`. Syntax: `Bash(termlink channel:*)` (CLI),
  `mcp__termlink__termlink_channel_*` (MCP — tool name wildcards after the literal
  `mcp__<server>__` prefix). **`defaultMode: "bypassPermissions"` is IGNORED from
  project settings** (guarded since v2.1.142 so a repo can't self-grant bypass) —
  only honored in user `~/.claude/settings.json`. => scoped allow-list is the only
  durable, project-portable path. Recommended block:
  `Bash(termlink channel:*)`, `Bash(termlink agent:*)`,
  `mcp__termlink__termlink_channel_*`, `mcp__termlink__termlink_agent_*`.
- **Per-project audit (settings.local.json permissions.allow):** all 4 reply via
  the MCP `termlink_channel_*` tools, but the lists are incomplete/inconsistent:
  aef = post/subscribe/reply/thread (MISSING ack, unread, list);
  workflow-designer = post/subscribe/state (MISSING ack, unread, list);
  workshop-designer = post/ack/subscribe/unread/list/thread (complete);
  sonnenstall = ONLY `agent_ask` (NO channel tools — cannot reply via MCP at all).
  None has `defaultMode`; none has `--dangerously-skip-permissions`/bypass in
  settings — today they rely purely on the launch-time blanket flag (the gap).
- **Live confirmation of the risk:** the T-2400 demo showed wfd's PTY WAS rung
  (`rang 'workflow-designer' ... offset=7`) — wake works; the stall was attention.
  But if wfd relaunched WITHOUT the blanket flag, `/check-arc`'s `channel_list`/
  `unread` (missing from wfd's allow) would prompt → stall. That is the exact
  recurrence this task closes.

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
test -f docs/operations/durable-reachable-auto-accept.md
grep -q 'permissions.allow' docs/operations/durable-reachable-auto-accept.md
grep -q 'mcp__termlink__termlink_channel_' docs/operations/durable-reachable-auto-accept.md
out=$(cat docs/operations/durable-reachable-auto-accept.md); echo "$out" | grep -q 'Binary'; echo "$out" | grep -q 'Identity'; echo "$out" | grep -q 'Auto-accept'

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

### 2026-07-11T07:28:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2401-durable-launch-path-independent-auto-acc.md
- **Context:** Initial task creation
