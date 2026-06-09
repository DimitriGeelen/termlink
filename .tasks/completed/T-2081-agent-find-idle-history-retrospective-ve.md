---
id: T-2081
name: "agent find-idle-history retrospective verb (substrate primitive #2 obs arc Slice 4, T-2074 mirror)"
description: >
  agent find-idle-history retrospective verb (substrate primitive #2 obs arc Slice 4, T-2074 mirror)

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
created: 2026-06-09T11:24:11Z
last_update: 2026-06-09T11:30:37Z
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

# T-2081: agent find-idle-history retrospective verb (substrate primitive #2 obs arc Slice 4, T-2074 mirror)

## Context

T-2080 (Slice 3 of substrate primitive #2 obs arc) shipped `agent find-idle
--watch --log <PATH>` writing NDJSON one-line-per-event audit trail to
`~/.termlink/find-idle.log` with schema
`{ts, agent_id, kind, role, capabilities, last_heartbeat_ms}`. There is
no read-side companion: an operator investigating "did claude-alpha go
busy in the last hour?" or "is this worker flapping?" today has to
hand-roll `jq` over the log file.

This task ships the retrospective verb — `agent find-idle-history` —
mirroring T-2074's `channel claims-history` shape exactly: read-only,
default 7-day window (clamped 1..=365), optional `--agent-id` exact
filter, optional `--log` path override, `--json` envelope, per-agent
aggregate footer counting new vs removed events.

Symmetry table (governor arc → claim arc → find-idle arc):

| Slice | Governor (T-2028 #10) | Claim (T-2042+) | Find-idle (T-2045+) |
|-------|-----------------------|-----------------|---------------------|
| Watch | T-2064 | T-2041 | T-2078 |
| Notify | T-2065 | T-2072 | T-2079 |
| Log | T-2066 | T-2073 | T-2080 |
| History CLI | T-2068 | T-2074 | **T-2081 (this task)** |

This closes the read-side observability for substrate primitive #2.

## Acceptance Criteria

### Agent
- [x] `agent find-idle-history` clap variant added to `AgentAction` enum in cli.rs with `--since`, `--agent-id`, `--log`, `--json` (mirror T-2074 `ClaimsHistory` shape, params renamed for the find-idle domain)
- [x] `main.rs` dispatches the variant to `commands::agent_find_idle::cmd_agent_find_idle_history`
- [x] `cmd_agent_find_idle_history` implemented in agent_find_idle.rs with: read log file (default `~/.termlink/find-idle.log`), clamp `since` to 1..=365, apply cutoff filter, apply agent-id exact-match filter, render human or JSON, missing-log emits operator hint pointing at the writer
- [x] Pure helpers `parse_find_idle_log` + `aggregate_find_idle_entries` (+ `FindIdleHistoryAgg` struct) extracted — pattern mirror of T-2074's `parse_claims_log` + `aggregate_claims_entries` + `ClaimsHistoryAgg`
- [x] `find_idle_log_path()` helper resolves `~/.termlink/find-idle.log` with the same `$HOME`-unset fallback as `claim_log_path` (T-2074)
- [x] Per-entry render line + per-agent aggregate footer + JSON envelope shape `{ok, entries, summary{total, per_agent:{<id>:{new, removed}}, since_days, agent_id_filter, malformed_lines_skipped, log_path}}` (mirror T-2074 envelope)
- [x] At least 4 unit tests: `parse_skips_malformed_and_counts`, `parse_applies_cutoff`, `parse_applies_agent_id_filter`, `aggregate_counts_kinds`
- [x] `cargo build -p termlink-cli` clean
- [x] `cargo test -p termlink-cli --lib agent_find_idle::tests::find_idle_history` passes
- [x] CLAUDE.md DISPATCH row extended with T-2081 + retrospective recipe

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

out=$(cargo build -p termlink 2>&1 | tail -3); echo "$out" | grep -qE 'Compiling|Finished|warning'
out=$(cargo test -p termlink --bin termlink commands::agent_find_idle::tests::find_idle_history -- --nocapture 2>&1 | tail -5); echo "$out" | grep -qE 'test result: ok'
out=$(./target/debug/termlink agent find-idle-history --since 7 --json 2>&1); echo "$out" | grep -qE '"ok": true|"ok":true'

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

### 2026-06-09T11:24:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2081-agent-find-idle-history-retrospective-ve.md
- **Context:** Initial task creation
