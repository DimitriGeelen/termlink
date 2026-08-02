---
id: T-2494
name: "unread count blind to never-acked offset-0 message (up_to=0 sentinel collision)"
description: >
  count_unread / count_unread_mcp use up_to:u64 with a 0 default sentinel that collides with acked-through-offset-0; inclusive off<=up_to then hides a never-acked FIRST DM (offset 0) as unread=0 on the RECEIVE surface (/check-arc, channel unread, agent_dms). Fix: thread Option<u64> (None=never-acked=>count all, Some(b)=>skip off<=b). Silent durable-message loss, directive-#2.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T09:37:40Z
last_update: 2026-08-02T09:45:28Z
date_finished: 2026-08-02T09:45:28Z
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

# T-2494: unread count blind to never-acked offset-0 message (up_to=0 sentinel collision)

## Context

The unread/RECEIVE surface (`/check-arc` DM inbox, `channel unread`, MCP `agent_dms`)
under-reports by exactly one message on a **never-acked** topic whose first message
sits at offset 0. Root: `count_unread` (CLI `channel.rs:4208`) and its MCP mirror
`count_unread_mcp` (`tools.rs:2714`) take `up_to: u64` and skip `off <= up_to`; the
two callers default `up_to = 0` and overwrite it only when a matching receipt row is
found. Because `record_append` (`termlink-bus/src/meta.rs:116`) assigns the first
message offset 0, and `channel ack` of one message writes `up_to = count-1 = 0`, the
`0` value is overloaded — "never acked" is indistinguishable from "acked through
offset 0", and the inclusive `<=` wrongly hides the never-acked first message.

## Acceptance Criteria

### Agent
- [x] `count_unread` (CLI) takes `up_to: Option<u64>`; `None` counts every content envelope (never-acked), `Some(b)` skips `off <= b` (wire inclusive semantics unchanged)
- [x] `count_unread_mcp` (MCP mirror) takes the same `Option<u64>` and behaves identically
- [x] Both CLI callers (`compute_dm_inbox_row`, `cmd_channel_unread`) and the MCP `agent_dms` caller initialize `up_to = None` and set `Some(n)` only when a matching receipt row with a numeric `up_to` is found (malformed `up_to` → `None` → conservative count-all)
- [x] New regression test: a single `{offset:0, msg_type:"chat"}` envelope with `up_to=None` returns `(1, Some(0))`; with `up_to=Some(0)` returns `(0, None)` — asserted in both CLI and MCP test modules
- [x] Existing `count_unread` / `count_unread_mcp` tests updated to the `Option` signature and still pass

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

cargo test -p termlink --bin termlink count_unread
cargo test -p termlink-mcp count_unread
cargo check -p termlink
cargo check -p termlink-mcp

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

**Symptom:** A recipient who has never acked a DM topic sees `unread=0` for a durably
stored first message (offset 0) on `/check-arc`, `channel unread`, and MCP `agent_dms`.
The message only becomes visible once a *second* message (offset 1) arrives. The
`compute_dm_inbox_row` comment (`:2980`) even asserts the opposite of the real behavior
("fallback to up_to=0 … ALL content counts as unread").

**Root cause:** `up_to: u64` with a `0` default is an overloaded sentinel. `record_append`
starts offsets at 0, and acking one message writes `up_to = count-1 = 0`, so `0` legitimately
means "acked through offset 0". The never-acked default *also* used `0`, and `count_unread`'s
inclusive `off <= up_to` skip then dropped the offset-0 message in both cases. The two
distinct states ("never acked" vs "acked through 0") were compressed into one value.

**Why structurally allowed:** the pure `count_unread` unit tests only exercised `up_to` >= 0
with content at higher offsets (bounds of 0,1,4 with messages above) — none covered the
never-acked-offset-0 case, so the sentinel collision was invisible. A `u64` type cannot
express "no bound", so the absence of a receipt was silently coerced to a real bound of 0.

**Prevention:** the type now distinguishes the states (`Option<u64>`), making the illegal
"never acked == acked-through-0" collapse unrepresentable. A new regression test pins both
directions (`None → (1, Some(0))`, `Some(0) → (0, None)`) in both the CLI and MCP mirrors,
so a future refactor that re-flattens to a sentinel fails the suite.

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

### 2026-08-02T09:37:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2494-unread-count-blind-to-never-acked-offset.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-f20cff95
- **Timestamp:** 2026-08-02T09:46:51Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T09:45:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
