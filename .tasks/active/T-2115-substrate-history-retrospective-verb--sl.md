---
id: T-2115
name: "substrate history retrospective verb — Slice 5 (T-2111 arc, T-2018 §6)"
description: >
  substrate history retrospective verb — Slice 5 (T-2111 arc, T-2018 §6)

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
created: 2026-06-10T08:16:16Z
last_update: 2026-06-10T08:16:16Z
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

# T-2115: substrate history retrospective verb — Slice 5 (T-2111 arc, T-2018 §6)

## Context

Slice 5 of T-2111 substrate-status observability roll-up arc under
T-2018 §6 — the retrospective read-side companion to T-2114's
`--watch --log` audit trail.

Closes the substrate-status arc's write-then-read symmetry:
- Write surface (already shipped): T-2113 `--notify` event hook +
  T-2114 `--log <PATH>` NDJSON audit trail
- Read surface (this slice): `substrate history` retrospective verb

Pattern parity (same shape across all 4 substrate-primitive arcs):
- T-2068 `fleet governor-history` (mirror of T-2066 governor `--log`)
- T-2081 `agent find-idle-history` (mirror of T-2080 find-idle `--log`)
- T-2074 `channel claims-history` (mirror of T-2073 claims `--log`)
- T-2086 `channel queue-history` (mirror of T-2085 queue `--log`)

Answers the operator question "when did substrate health flip?"
without needing the watch terminal still attached. Forensic
retrospective in a JSON-friendly aggregate.

Design (mirror T-2086 queue-history):
- New subcommand `termlink substrate history [--since DAYS] [--field NAME]
  [--log PATH] [--json]`
- Defaults: `--since 7`, log path `~/.termlink/substrate.log`
- `--since DAYS` clamped 1..=365
- `--field` exact-match filter on `field` column (per-field history)
- Pure helpers in substrate.rs:
  - `parse_substrate_log(text, cutoff_secs, field_filter) -> (Vec<Value>, malformed_count)`
  - `aggregate_substrate_entries(entries) -> BTreeMap<String, u64>` (per-field counts)
  - `render_substrate_history_line(entry) -> String`
  - `default_substrate_log_path() -> PathBuf` → `~/.termlink/substrate.log`
  - `rfc3339_to_unix_secs_substrate(ts) -> i64` (duplicate per T-2069 convention)
- Read-only: no auth, no network, no log mutation
- Missing log → operator hint pointing back at `substrate status --watch --log`
- JSON envelope:
  `{ok, entries, summary{total, per_field:{<f>:{count}}, since_days,
    field_filter, malformed_lines_skipped, log_path}}`

Right-sized — ~200 LOC + 3-4 unit tests (parse, aggregate, render,
filter). Closes the write-then-read symmetry for substrate-status
observability. Slice 6 (MCP parity for `substrate status`) and Slice 7
(MCP parity for `substrate history`) follow.

## Acceptance Criteria

### Agent
- [x] New `SubstrateAction::History` clap variant with flags `--since
      DAYS` (default 7), `--field NAME`, `--log PATH`, `--json`. Wired
      through `main.rs` dispatch.
- [x] Pure helper `parse_substrate_log(text, cutoff_secs, field_filter)
      -> (Vec<Value>, usize)`: skips malformed lines (count returned),
      filters by ts cutoff + field exact-match. Unit test covers
      malformed-skip + filter + cutoff behavior.
- [x] Pure helper `aggregate_substrate_entries(entries) -> BTreeMap<String, u64>`:
      groups by `field` column into per-field event counts. Unit test
      asserts counts roll up correctly.
- [x] Pure `render_substrate_history_line(entry) -> String`: emits one
      human-format line `<ts>  <field>  <old>→<new>`. Unit test covers
      the rendered shape.
- [x] `--since DAYS` clamped to 1..=365 (mirror prior history verbs).
- [x] Missing log file → operator hint pointing at `substrate status
      --watch --log <PATH>` writer. JSON mode returns
      `{ok:true, entries:[], summary{...}, note:"log file does not exist yet"}`.
- [x] JSON envelope shape per spec: `{ok, entries, summary{total,
      per_field:{<f>:{count}}, since_days, field_filter,
      malformed_lines_skipped, log_path}}`. Unit test parses one
      synthetic log + asserts envelope fields.
- [x] `cargo check -p termlink` + `cargo test -p termlink --bin termlink
      substrate` pass. Full regression `cargo test -p termlink --bin
      termlink` passes (≥913 baseline after Slice 4).
- [x] Live smoke: re-use `/tmp/T-2114-smoke.log` from Slice 4 if it has
      entries (induce one if not). Run `termlink substrate history
      --since 1 --log /tmp/T-2114-smoke.log`. Assert human output shows
      the recorded `claim_topic_count: 1338→1339` line + aggregate
      footer. `--json` returns a `{summary.total >= 1}` envelope.
      Append timestamped Updates evidence.

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

cargo check -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink substrate 2>&1 | tail -10
help_out=$(./target/debug/termlink substrate history --help 2>&1); echo "$help_out" | grep -q "since"

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

### 2026-06-10T08:16:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2115-substrate-history-retrospective-verb--sl.md
- **Context:** Initial task creation

### 2026-06-10T08:22:00Z — slice 5 shipped end-to-end
- **Action:** Implemented `substrate history` retrospective read verb.
  cli.rs: added `SubstrateAction::History` variant with --since/--field
  /--log/--json flags. main.rs: wired through dispatch. substrate.rs:
  added 4 pure helpers (default_substrate_log_path,
  rfc3339_to_unix_secs_substrate, parse_substrate_log,
  aggregate_substrate_entries, render_substrate_history_line) plus
  cmd_substrate_history handler.
- **Verification:**
  - `cargo check -p termlink` — PASS (10.31s)
  - `cargo test -p termlink --bin termlink substrate` — 20/20 PASS
    (3 new: parse_substrate_log_skips_malformed_and_filters,
    aggregate_substrate_entries_groups_by_field,
    render_substrate_history_line_shape)
  - `cargo test -p termlink --bin termlink` — 916/916 PASS (was 913
    baseline pre-Slice 5)
  - Live smoke 1 (read Slice 4's log): `substrate history --since 1
    --log /tmp/T-2114-smoke.log` renders exact human-format line
    `2026-06-10T08:13:36Z  claim_topic_count  1338→1339` + aggregate
    footer `claim_topic_count  1`
  - Live smoke 2 (JSON envelope): `--json` returns
    `{ok:true, entries:[1 entry], summary:{total:1, per_field:
    {claim_topic_count:{count:1}}, since_days:1, field_filter:null,
    malformed_lines_skipped:0, log_path:"/tmp/T-2114-smoke.log"}}` —
    exact spec match
  - Live smoke 3 (missing log): `--log /tmp/nonexistent-T-2115.log`
    renders operator hint `(no log file at ... — write events first
    with \`substrate status --watch --log ...\`)`
  - Live smoke 4 (field filter match): `--field claim_topic_count`
    returns the matching entry
  - Live smoke 5 (field filter no match): `--field dispatch_idle_count`
    renders `(no entries in last 1 day(s) field="dispatch_idle_count")`
    — affirmative empty path
- **Outcome:** Slice 5 closes the substrate-status arc's write-then-read
  symmetry — write surface (T-2113 --notify + T-2114 --log) AND read
  surface (this slice) now both shipped. The CLI tier of the substrate-
  status observability arc is now end-to-end at functional parity with
  every prior substrate-primitive arc (governor #10, claim #1, dispatch
  #2, queue #5). Slice 6 (MCP parity for `substrate status` one-shot)
  and Slice 7 (MCP parity for `substrate history`) follow.
- **Context:** T-2018 §6 observability roll-up arc — T-2111 Slice 5
  (CLI-tier closure).
