---
id: T-2421
name: "channel.delete primitive — hub can forget a topic (T-2419 GAP-2)"
description: >
  channel.delete primitive — hub can forget a topic (T-2419 GAP-2)

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
created: 2026-07-19T20:51:04Z
last_update: 2026-07-19T21:05:24Z
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

# T-2421: channel.delete primitive — hub can forget a topic (T-2419 GAP-2)

## Context

T-2419 GAP-2: the hub cannot forget. No topic-deletion primitive exists — only
retention trim (`channel sweep`), which empties a topic but leaves it in the topic
list forever. Field evidence: the .107 production hub carries 1,581 topics of which
~60% are test debris (617 t-*/T-XXXX smoke, 107 xhub-*, 96 stress-*); walk verbs
(`claims-summary --all`) now self-DoS on the debris (-32008 mid-walk). Scope: a new
`channel.delete` RPC (Execute scope, exact-name only, NO wildcards — destructive verbs
never glob), bus-side `delete_topic` (removes log storage + topic registry entry +
claims + cv_index entries), CLI `termlink channel delete <TOPIC> --yes`, tests for
delete/nonexistent/recreate-fresh semantics. Operator directive 2026-07-19
("identify gaps … build these") + T-2419 §6 authorize the build. Wildcard/bulk sweep
tooling is a FOLLOW-UP (operator script walking list+delete), not this primitive.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Bus: `delete_topic(name)` removes the topic's log storage, registry/meta entry,
      claims, and cv_index entries; returns whether the topic existed; a subsequent
      post to the same name creates a FRESH topic starting at offset 0
- [x] Hub: `channel.delete` RPC routed with Execute (highest) scope; exact-name only —
      a name containing `*` or `?` is rejected with a clear error; unknown topic
      returns a structured not-found error (no stealth success)
- [x] CLI: `termlink channel delete <TOPIC>` requires `--yes` (refuses without it,
      printing what would be deleted); supports `--json`; success output includes the
      deleted record count
- [x] Tests: bus unit tests (delete existing, delete nonexistent, recreate-fresh at
      offset 0, claims/cv_index cleared) + wildcard-rejection test; full workspace
      test suite still green (all crates 0 failed; termlink-mcp parity showed 2
      timeouts under full-workspace load contention, standalone rerun 24/24 green —
      pre-existing flake class, not a T-2421 surface)

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

grep -q "CHANNEL_DELETE" crates/termlink-protocol/src/control.rs
grep -q "handle_channel_delete_with" crates/termlink-hub/src/channel.rs
grep -q "cmd_channel_delete" crates/termlink-cli/src/commands/channel.rs
out=$(cargo test -p termlink-bus --lib delete_topic 2>&1); echo "$out" | grep -q "3 passed"
out=$(cargo test -p termlink-hub --lib delete_ 2>&1); echo "$out" | grep -q "4 passed"

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

### 2026-07-19 — Execute scope, not Control
- **Chose:** `channel.delete` requires Execute (highest) scope.
- **Why:** trim/sweep (Control) empty a topic under a policy the topic keeps; delete erases the topic's existence, cursors, claims, cv_index for every subscriber, unrecoverably. One notch above is right.
- **Rejected:** Control (same as trim) — under-weights irreversibility; deny-by-default fallthrough (implicit Execute) — unlisted methods read as accidents, this one must be explicit + tested.

### 2026-07-19 — No MCP parity slice
- **Chose:** CLI + RPC only; no `termlink_channel_delete` MCP tool.
- **Why:** T-2419 GAP-6 policy — new verbs ship core+JSON only until field demand exists; also a destructive verb behind an agent-callable tool wants its own safety review first.
- **Rejected:** reflex 5-slice treatment (the sprawl engine T-2419 §5.6 names).

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

### 2026-07-19T20:51:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2421-channeldelete-primitive--hub-can-forget-.md
- **Context:** Initial task creation
