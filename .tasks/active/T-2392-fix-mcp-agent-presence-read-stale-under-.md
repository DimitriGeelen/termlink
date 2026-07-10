---
id: T-2392
name: "Fix MCP agent-presence read stale under latest-per-cv-key (T-2391 MCP twin)"
description: >
  MCP tools.rs fetch_recent(agent-presence,500) + count.saturating_sub(slice_size) count-seek has the same T-2390/T-2391 staleness under latest_per_cv_key; MCP presence tools (presence_now/listeners/find_idle) read stale. Mirror the CLI cv_index fix (subscribe include_current_value) in termlink-mcp.

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
created: 2026-07-10T10:08:08Z
last_update: 2026-07-10T11:27:43Z
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

# T-2392: Fix MCP agent-presence read stale under latest-per-cv-key (T-2391 MCP twin)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

MCP twin of T-2391 (CLI) / T-2390 (shell): the `termlink-mcp` crate reimplements
the count-seek presence read (`cursor = count.saturating_sub(slice)`), which reads
LIVE peers as absent under `latest_per_cv_key` retention. The one load-bearing
agent-presence face in the MCP crate is `resolve_contact_via_fleet_mcp` (the
`agent_contact` / send-auto-discover fleet-resolution path). `find_idle` delegates
to the hub's cv_index-backed `AGENT_FIND_IDLE` RPC (T-2109 — already correct),
`presence_now` full-walks agent-chat-arc from cursor 0 (unaffected), and
`listeners` shells out to the already-fixed `agent-listeners.sh` (T-2390) — so
this task's scope is the single send-path read. See PL-250.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] `current_value_msgs_mcp` pure helper added to termlink-mcp/src/tools.rs — extracts `msg` envelopes from a `channel.subscribe` response's `current_values` array (cv_index snapshot); mirror of CLI `current_value_msgs` (T-2391, per T-2069 duplicate-don't-share convention)
- [ ] `ContactHub::fetch_presence_recent` method added — reads `agent-presence` via `channel.subscribe include_current_value:true` and returns cv snapshot; falls back to the count-seek `fetch_recent("agent-presence", …)` only when the snapshot is empty (pre-cv_index hub / non-cv-tagged producers)
- [ ] `resolve_contact_via_fleet_mcp` (the MCP `agent_contact` / send-auto-discover fleet-resolution path, line ~7005) routed through `fetch_presence_recent` — no raw count-seek `fetch_recent` remains on `agent-presence`
- [ ] Unit test `current_value_msgs_mcp_surfaces_live_agent_at_high_offset` proves the cv snapshot surfaces a LIVE agent whose offset is far past the retained `count` (the case count-seek misses under latest_per_cv_key), preserving `pty_session`; plus an empty-snapshot test
- [ ] `cargo test -p termlink-mcp current_value_msgs_mcp` passes (both tests)
- [ ] Rebuild + reinstall the binary; the MCP presence-discovery send path resolves LIVE peers under latest_per_cv_key retention

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
out=$(cargo test -p termlink-mcp current_value_msgs_mcp 2>&1); echo "$out" | grep -q "2 passed"
grep -q "fn fetch_presence_recent" crates/termlink-mcp/src/tools.rs
grep -q "fn current_value_msgs_mcp" crates/termlink-mcp/src/tools.rs

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

**Symptom:** Under `latest_per_cv_key` retention on `agent-presence`, the MCP
`agent_contact` / send-auto-discover fleet-resolution path
(`resolve_contact_via_fleet_mcp`) read LIVE peers as absent — a live agent whose
heartbeat sits at the monotonic tail offset (~33k) was invisible to the read.
On a `--require-online`-style send this would abort delivery to a reachable agent.

**Root cause:** `ContactHub::fetch_recent` (and its free-fn twin
`fetch_topic_msgs_mcp`) computes `cursor = channel-info.count.saturating_sub(slice_size)`.
Under `latest_per_cv_key`, `channel info.count` is the RETAINED count (bounded ≈
N agents), decoupled from the monotonic tail offset. Seeking to `count - slice`
lands in the oldest retained window, days behind the true tail — the identical
count-seek defect fixed for the shell in T-2390 and the CLI Rust in T-2391, here
on the MCP twin.

**Why structurally allowed:** the count-seek helper was written pre-retention
(when `count == tail offset` held), and each read face (shell `agent-listeners.sh`,
CLI `agent.rs`, MCP `tools.rs`) reimplements it independently (T-2069
duplicate-don't-cross-crate-share convention). Fixing one face did not fix the
others, and no test exercised a high-offset heartbeat under retention. T-2390 +
T-2391 closed two of the three faces; the MCP face stayed dark.

**Prevention:** read `agent-presence` via cv_index (`channel.subscribe
include_current_value:true`) — correct regardless of retention policy or sweep
cadence — with fallback to the count-seek path only when the snapshot is empty
(pre-cv_index hub). A unit test pins a LIVE agent at an offset far past the
retained count and asserts it surfaces with `pty_session` preserved; the
count-seek path fails that test. PL-250 (registered at T-2390) names the class
fleet-wide; this closes the last known code face of it.

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

### 2026-07-10T10:08:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2392-fix-mcp-agent-presence-read-stale-under-.md
- **Context:** Initial task creation

### 2026-07-10T11:27:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
