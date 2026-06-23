---
id: T-2253
name: "MCP queue-status dead-letter parity — surface R4 poison drops to agent callers"
description: >
  MCP queue-status dead-letter parity — surface R4 poison drops to agent callers

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-substrate-fitness]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-23T11:17:58Z
last_update: 2026-06-23T11:17:58Z
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

# T-2253: MCP queue-status dead-letter parity — surface R4 poison drops to agent callers

## Context

arc-substrate-fitness follow-through on R4 (T-2243). R4 replaced the silent poison-drop in
the offline queue with a durable dead-letter store and surfaced it at the CLI:
`channel queue-status` prints `dead_letters` (count) + `dead_letter_rows` (capped at 50) in
both human and JSON modes (`channel.rs:8573-8639`). But the MCP twin
`termlink_channel_queue_status` (`tools.rs:29029-29073`) returns only
`{queue_path, exists, cap, pending, oldest}` — it never reads `dead_letter_count()` /
`list_dead_letters()`. So an agent calling via MCP is blind to the poison backlog a human
sees at the CLI: R4's "a poison-drop must not be silent" guarantee is violated for MCP
callers. This is the PL-167 / PL-172 class (MCP wrapper silently hides a field the CLI shows)
and directly undercuts the arc's AS_FAILURE_OBSERVABILITY (w5) driver. Fix = mirror the CLI's
two fields into the MCP response, same shape, same 50-row cap.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink_channel_queue_status` MCP response includes `dead_letters` (exact count via `dead_letter_count()`) and `dead_letter_rows` (capped at 50 via `list_dead_letters()`), with the same per-row field shape as the CLI (`id, topic, msg_type, sender_id, reason, attempts, enqueued_ms, dead_lettered_ms`) — built by the shared pure helper `build_queue_status_exists_value`
- [x] The MCP tool description is updated to mention the dead-letter fields (no longer claims to surface only count/lag/last-offset) — both the registry one-liner (`tools.rs:889`) and the `#[tool(...)]` description
- [x] Regression test `mcp_queue_status_surfaces_dead_letters`: an in-memory queue with 1 dead-lettered row yields a response whose `dead_letters` count is 1 and whose `dead_letter_rows[0].reason` carries the forensic reason — proves an MCP caller is no longer blind to the poison backlog (PL-213: asserts the claimed property)
- [x] `cargo test -p termlink-mcp` lib suite passes (869 + 2 new); empty *existing* queue returns `dead_letters: 0` / `[]` (`mcp_queue_status_empty_queue_has_zero_dead_letters`); the non-existent-queue path is unchanged from the CLI (minimal `exists:false` object). NOTE: 6 pre-existing hub-dependent integration failures in `tests/mcp_integration.rs` + a `tests/parity.rs` hub-hang are environmental (confirmed present on clean tree before this change) — NOT a regression

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
# T-2253 — MCP dead-letter parity. Each line self-contained (P-011 runs each as a separate set -u shell).
out=$(cargo test -p termlink-mcp dead_letter 2>&1); echo "$out" | grep -q "test result: ok"
grep -q '"dead_letters"' crates/termlink-mcp/src/tools.rs
grep -q '"dead_letter_rows"' crates/termlink-mcp/src/tools.rs

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

### 2026-06-23 — gap surfaced during arc closure pass, not at arc filing
- **What changed:** R4 (T-2243) was treated as fully shipped — its AC #2 claimed dead-letter rows are "readable via queue-status or a read verb" and was ticked. True at the CLI tier, but the MCP twin `termlink_channel_queue_status` never read the dead-letter store. A poison-drop was therefore still *silent* to every agent calling through MCP, undercutting the arc's AS_FAILURE_OBSERVABILITY (w5) driver. This is the PL-167/PL-172 class (MCP wrapper silently hides a CLI field).
- **Plan impact:** R4 wasn't actually closed for agent callers; the arc's observability driver needed this one-field-pair parity fix to be honestly satisfied. No change to the arc shape — this is R4 follow-through, not a new node.
- **Triggered:** T-2253 (this task). Also extracted `build_queue_status_exists_value` as a pure helper so CLI↔MCP field-shape drift is now unit-testable in lockstep (defends against the next PL-167 recurrence on this verb).

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

### 2026-06-23T11:17:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2253-mcp-queue-status-dead-letter-parity--sur.md
- **Context:** Initial task creation
