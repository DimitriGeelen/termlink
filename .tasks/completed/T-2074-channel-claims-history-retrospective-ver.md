---
id: T-2074
name: "channel claims-history retrospective verb (T-2073 read-side, T-2068 mirror)"
description: >
  channel claims-history retrospective verb (T-2073 read-side, T-2068 mirror)

status: work-completed
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
created: 2026-06-09T08:47:01Z
last_update: 2026-06-09T08:47:01Z
date_finished: 2026-06-09T08:57:57Z
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

# T-2074: channel claims-history retrospective verb (T-2073 read-side, T-2068 mirror)

## Context

T-2073 shipped `--log <PATH>` (NDJSON audit trail) for
`channel claims-summary --watch`. T-2074 ships the read-side companion:
a `channel claims-history` retrospective verb that walks the log file,
filters by window + topic, renders one human-format line per matching
entry, and prints a per-topic aggregate footer.

Mirror of T-2068 `fleet governor-history`. Read-only; no auth; no
network; pure file scan of `~/.termlink/claims.log` (or any `--log`
override matching the watch loop's destination).

Answers operator questions:
- "Has this topic been stuck repeatedly or was that a one-off?"
- "How many transitions across the fleet this week?"
- "Which topics flapped the most in the last 24h?"

Without these answers, the audit trail T-2073 wrote is unstructured —
the operator would have to `jq` it manually. This verb codifies the
common operator query patterns.

Filters:
- `--since DAYS` — window from now (default 7, clamped 1..=365)
- `--topic NAME` — exact-match topic filter
- `--log PATH` — override log file location (default `~/.termlink/claims.log`)
- `--json` — machine-readable envelope

Output schema (human):
```
<ts>  <topic>  <kind>  old=<stuck>→<stuck>  counters: active=N expired=N oldest_age=Nms

Aggregate (since N days, M entries, K malformed lines skipped):
  <topic>  <transition_count> transition(s)  <new_count> new  <removed_count> removed
  ...
```

Pure helpers (mirror T-2068):
- `parse_claims_log(text, cutoff_secs, hub_filter, topic_filter) -> (Vec<Value>, usize)`
- `aggregate_claims_entries(entries) -> BTreeMap<String, ClaimsHistoryAgg>`

## Acceptance Criteria

### Agent
- [x] `cli.rs` `ClaimsHistory` variant added under ChannelAction (positional: none; flags: `--since DAYS`, `--topic NAME`, `--log PATH`, `--json`)
- [x] `main.rs` dispatch wires `ClaimsHistory` to a new `cmd_channel_claims_history`
- [x] Default log path: `$HOME/.termlink/claims.log`; helper `claim_log_path()` extracted
- [x] Missing log path prints a one-line hint pointing back at `claims-summary --watch --log` (exit 0)
- [x] `--since DAYS` clamped to 1..=365 (mirror T-2068's range)
- [x] `--topic NAME` filters to exact-match entries
- [x] Pure helper `parse_claims_log(text, cutoff_secs, topic_filter) -> (Vec<Value>, usize)` extracted; returns `(entries, malformed_count)`
- [x] Pure helper `aggregate_claims_entries(entries) -> BTreeMap<String, ClaimsHistoryAgg>` extracted; counts per-topic `transition` / `new` / `removed`
- [x] Human-format output renders one line per entry + per-topic aggregate footer + total line + malformed-skip note
- [x] `--json` emits `{ok, entries, summary{total, per_topic:{<topic>:{transitions, new, removed}}, since_days, topic_filter, malformed_lines_skipped, log_path}}` envelope
- [x] At least 4 unit tests on the pure helpers: (a) malformed-line skip + count, (b) cutoff filter excludes old entries, (c) topic filter excludes non-matching, (d) aggregate counts kinds correctly
- [x] `cargo check -p termlink` builds clean
- [x] `cargo test --bin termlink claims_history` passes
- [x] CLAUDE.md CLAIM-OBSERVABILITY row extended with the `claims-history` verb

### Human
<!-- All ACs above are agent-verifiable; no human review needed. -->

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

cargo check -p termlink 2>&1 | tail -5 | grep -qv "error\["
out=$(cargo test --bin termlink --release claims_history 2>&1); echo "$out" | grep -q "test result: ok"

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

### 2026-06-09T08:47:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2074-channel-claims-history-retrospective-ver.md
- **Context:** Initial task creation
