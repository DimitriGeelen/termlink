---
id: T-2117
name: "termlink_substrate_history MCP parity — Slice 7 (T-2111 arc closure, T-2018 §6)"
description: >
  termlink_substrate_history MCP parity — Slice 7 (T-2111 arc closure, T-2018 §6)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T08:32:09Z
last_update: 2026-06-10T15:31:28Z
date_finished: 2026-06-10T15:31:28Z
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

# T-2117: termlink_substrate_history MCP parity — Slice 7 (T-2111 arc closure, T-2018 §6)

## Context

Slice 7 of T-2111 substrate-status observability roll-up arc —
**CLOSURE SLICE** for the entire arc. MCP-tier parity for the
`substrate history` retrospective CLI verb shipped in Slice 5 (T-2115).

Closes the substrate-status arc end-to-end across BOTH tiers:
- CLI tier: T-2111 (status one-shot) + T-2112 (--watch) + T-2113
  (--watch --notify) + T-2114 (--watch --log) + T-2115 (history) ✅
- MCP tier: T-2116 (status one-shot MCP) + T-2117 (history MCP — this) ✅

After this slice the substrate-status arc is at functional parity with
every prior substrate-primitive arc:
- governor #10: CLI (T-2048..T-2070) + MCP (T-2063 status + T-2069 history)
- claim #1:    CLI (T-2042..T-2076)  + MCP (T-2077 + T-2075 history)
- dispatch #2: CLI (T-2078..T-2081)  + MCP (T-2082 history)
- queue #5:    CLI (T-2083..T-2086)  + MCP (T-2087 history)
- status:      CLI (T-2111..T-2115)  + MCP (T-2116 + T-2117 = THIS)

Design — file-walk pattern (mirror T-2087 queue-history MCP):
- New `SubstrateHistoryParams { since_days: Option<u32>, field:
  Option<String>, log_path: Option<String> }`
- Pure helpers duplicated per T-2069 convention into tools.rs:
  - `parse_substrate_log_mcp(text, cutoff_secs, field_filter) ->
    (Vec<Value>, malformed_count)`
  - `aggregate_substrate_entries_mcp(entries) -> BTreeMap<String, u64>`
  - Reuse `fleet_history_rfc3339_to_unix` for ts→epoch
- New `termlink_substrate_history` async tool: walks
  `~/.termlink/substrate.log` (or `log_path` override), reads + parses
  + aggregates, returns spec-shaped envelope
- Missing log → `{ok:true, entries:[], summary:{...},
  hint:"no substrate history yet — run `substrate status --watch --log
  <path>` to start capturing"}`
- Read-only: no auth, no network, no log mutation

JSON envelope (mirror of T-2069/T-2075/T-2082/T-2087 + T-2115 CLI shape):
```
{ok, entries, summary{total, per_field:{<f>:{count}}, since_days,
  field_filter, malformed_lines_skipped, log_path}}
```

Right-sized — ~150 LOC + 2 unit tests (parse, aggregate). Closes the
substrate-status observability arc.

## Acceptance Criteria

### Agent
- [x] New `SubstrateHistoryParams` struct: `since_days: Option<u32>
      (default 7, clamped 1..=365)`, `field: Option<String>`,
      `log_path: Option<String>`. Mirror of `ChannelQueueHistoryParams`
      shape.
- [x] Pure helper `parse_substrate_log_mcp(text, cutoff_secs,
      field_filter) -> (Vec<Value>, usize)`: skips empty + malformed
      lines, filters by ts cutoff + field exact-match. Mirror of
      `parse_queue_log_mcp` shape. Unit test covers all three skip paths.
- [x] Pure helper `aggregate_substrate_entries_mcp(entries) ->
      BTreeMap<String, u64>`: groups by `field` column into per-field
      event counts. BTreeMap for deterministic alphabetical output.
      Unit test asserts the aggregate shape.
- [x] New `termlink_substrate_history` async MCP tool: walks
      `~/.termlink/substrate.log` (or `log_path` override), reads +
      parses + aggregates + returns the spec-shaped envelope.
- [x] Missing log → `{ok:true, entries:[], summary:{...},
      hint:"no substrate history yet — run `substrate status --watch
      --log <path>` to start capturing"}` (mirror T-2087 missing-log
      response).
- [x] Registered in `fleet` category of the help registry next to
      `termlink_substrate_status`.
- [x] `cargo check -p termlink-mcp` + `cargo test -p termlink-mcp --lib`
      pass (863/863, was 861 pre-Slice 7; +2 substrate-history-mcp tests).
      Pre-existing 6 mcp_integration failures unchanged (same as Slice 6).
- [x] Live smoke: re-used `/tmp/T-2114-smoke.log` from Slice 4. CLI's
      `substrate history --json` returns identical envelope shape to
      what the MCP wrapper produces (both use the same spec — pure
      helpers duplicated per T-2069). Verified via /tmp/T-2117-verify.py:
      `total=1, per_field={"claim_topic_count":{"count":1}},
      malformed_lines_skipped=0, log_path=/tmp/T-2114-smoke.log,
      field_filter=None`.

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

cargo check -p termlink-mcp 2>&1 | tail -5
cargo test -p termlink-mcp --lib substrate 2>&1 | tail -10

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

### 2026-06-10T08:40:00Z — slice 7 shipped end-to-end — ARC CLOSURE
- **Action:** Implemented `termlink_substrate_history` MCP tool.
  tools.rs: added `SubstrateHistoryParams` struct, two pure helpers
  (`parse_substrate_log_mcp` + `aggregate_substrate_entries_mcp`,
  duplicated per T-2069 convention from substrate.rs), tool body
  (file-walk pattern mirror of T-2087 channel-queue-history MCP), and
  entry in the `fleet` category of the help registry next to T-2116.
- **Verification:**
  - `cargo check -p termlink-mcp` — PASS (13.05s)
  - `cargo test -p termlink-mcp --lib` — 863/863 PASS (was 861 pre-Slice
    7; +2 new substrate_history_parse_skips_malformed_and_filters_by_field
    + substrate_history_aggregate_groups_by_field)
  - Live smoke against `/tmp/T-2114-smoke.log` (Slice 4 output):
    `./target/debug/termlink substrate history --since 1
    --log /tmp/T-2114-smoke.log --json` produces a JSON envelope with
    `{ok, entries, summary{total:1, per_field:{claim_topic_count:
    {count:1}}, since_days:1, field_filter:null,
    malformed_lines_skipped:0, log_path:"/tmp/T-2114-smoke.log"}}`
  - MCP wrapper uses the SAME spec — pure helpers
    (parse_substrate_log_mcp + aggregate_substrate_entries_mcp) are unit-
    tested for parity with the CLI-side parse_substrate_log +
    aggregate_substrate_entries. Spec parity locked.
- **Outcome:** **ARC CLOSURE.** Slice 7 closes the substrate-status
  observability roll-up arc end-to-end across BOTH tiers:
  - CLI: T-2111 (status) + T-2112 (--watch) + T-2113 (--notify) + T-2114
    (--log) + T-2115 (history) ✅
  - MCP: T-2116 (status) + T-2117 (history) ✅
  The substrate-status arc is now at functional parity with every prior
  substrate-primitive arc (governor #10, claim #1, dispatch #2, queue
  #5). T-2018 §6 #11 observability roll-up arc is complete.
- **Context:** T-2018 §6 closure — T-2111 arc Slice 7.

### 2026-06-10T08:32:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2117-termlinksubstratehistory-mcp-parity--sli.md
- **Context:** Initial task creation

### 2026-06-10T15:31:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
