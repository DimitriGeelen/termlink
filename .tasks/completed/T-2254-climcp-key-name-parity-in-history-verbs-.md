---
id: T-2254
name: "CLI↔MCP key-name parity in history verbs — align find-idle/claims-history summary keys to *_events"
description: >
  CLI↔MCP key-name parity in history verbs — align find-idle/claims-history summary keys to *_events

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-substrate-fitness]
components: [crates/termlink-cli/src/commands/agent_find_idle.rs, crates/termlink-cli/src/commands/channel.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-23T13:09:37Z
last_update: 2026-06-23T13:13:12Z
date_finished: 2026-06-23T13:13:12Z
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

# T-2254: CLI↔MCP key-name parity in history verbs — align find-idle/claims-history summary keys to *_events

## Context

arc-substrate-fitness coordination-truth (w6) consistency fix, surfaced by the T-2253
CLI↔MCP parity audit. Two retrospective verbs emit their `summary` aggregate map under
DIFFERENT key names on the CLI vs MCP side — same data, divergent keys, so an agent and an
operator parsing the same logical counter disagree:
- `find-idle-history`: CLI `summary.per_agent.<id>` = `{new, removed}` (`agent_find_idle.rs:684-685`)
  vs MCP `{new_events, removed_events}` (`tools.rs:28912-28913`).
- `claims-history`: CLI `summary.per_topic.<t>` = `{transitions, new, removed}` (`channel.rs:10752-10753`)
  vs MCP `{transitions, new_events, removed_events}` (`tools.rs:16614-16616`).
The `*_events` naming is canonical on three independent signals: (1) the sibling `queue-history`
verb already emits `pending_events`/`drained_events` on BOTH sides; (2) the MCP tool descriptions
document `new_events`/`removed_events` (`tools.rs:16551, 28849`); (3) the CLI's own internal aggregate
structs are named `new_events`/`removed_events` — the CLI was renaming them to `new`/`removed` only at
the JSON boundary. Fix = rename the CLI JSON summary keys to `*_events`. JSON-output-only; the
human-readable text footer and the per-entry `kind` enum values (`"new"`/`"removed"` — correct,
unchanged) are untouched. Breaking note: alters `--json` output of two recent low-adoption verbs
(T-2074 claims-history, T-2081 find-idle-history); acceptable to converge on the canonical shape.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `find-idle-history` CLI `--json` emits `summary.per_agent.<id>` as `{new_events, removed_events}` (was `{new, removed}`) — matches the MCP `termlink_agent_find_idle_history` shape (`agent_find_idle.rs:686-687`)
- [x] `claims-history` CLI `--json` emits `summary.per_topic.<t>` as `{transitions, new_events, removed_events}` (was `{transitions, new, removed}`) — matches the MCP `termlink_channel_claims_history` shape (`channel.rs:10754-10755`)
- [x] Per-entry `kind` values (`"new"`/`"removed"` `ClaimChangeKind` strings) and the human-readable (non-`--json`) footers are UNCHANGED — the rename is confined to the JSON summary-aggregate map keys (grep confirmed: 0 old summary-key emit sites remain; `ClaimChangeKind` string sites untouched)
- [x] `cargo build -p termlink` compiles clean; no CLI test asserted the old summary keys (existing helper tests assert the `new_events`/`removed_events` struct fields, not rendered JSON keys); `cargo test -p termlink claims_history` (4) + `find_idle` (15) pass

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
# T-2254 — CLI↔MCP key-name parity. Each line self-contained (P-011: separate set -u shell).
out=$(cargo build -p termlink 2>&1); echo "$out" | grep -qE "Finished|Compiling" && ! echo "$out" | grep -q "error\["
grep -q '"new_events": a.new_events' crates/termlink-cli/src/commands/agent_find_idle.rs
grep -q '"new_events": a.new_events' crates/termlink-cli/src/commands/channel.rs

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

### 2026-06-23 — found by the T-2253 systematic parity audit, not the original arc map
- **What changed:** After fixing the T-2253 dead-letter STRIP, I ran a systematic CLI↔MCP field-parity audit across the substrate read verbs (the w5/w6 surface). It found NO further silent-strips (queue_status was the only true strip) but two key-NAME divergences in the `*-history` verbs. So the substrate observability surface had a second, lower-severity parity-bug class (divergent key names, not missing data) the original arc map didn't enumerate.
- **Plan impact:** None to the arc shape — this is consistency hardening of already-shipped substrate verbs, in service of the AS_COORDINATION_TRUTH (w6) driver (CLI and MCP callers must agree on the contract). The audit also gives positive confirmation the rest of the surface is parity-clean.
- **Triggered:** T-2254 (this task). Decision: align CLI→`*_events` (not MCP→`new`/`removed`) because `*_events` is canonical on three signals — sibling queue-history, the MCP tool descriptions, and the CLI's own struct field names.

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

### 2026-06-23T13:09:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2254-climcp-key-name-parity-in-history-verbs-.md
- **Context:** Initial task creation

### 2026-06-23T13:13:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
