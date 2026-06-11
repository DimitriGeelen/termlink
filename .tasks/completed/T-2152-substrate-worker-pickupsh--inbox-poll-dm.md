---
id: T-2152
name: "substrate-worker-pickup.sh — inbox-poll DM dispatch loop (orchestrator pair)"
description: >
  substrate-worker-pickup.sh — inbox-poll DM dispatch loop (orchestrator pair)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-11T06:51:23Z
last_update: 2026-06-11T07:03:25Z
date_finished: 2026-06-11T07:03:25Z
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

# T-2152: substrate-worker-pickup.sh — inbox-poll DM dispatch loop (orchestrator pair)

## Context

The substrate ships `substrate-orchestrator-loop.sh` (T-2148, dispatch side)
and `substrate-worker-loop.sh` (T-2146 + T-2150, per-unit lifecycle). But
the GLUE between them — the worker-side inbox-poll loop that turns
orchestrator DMs into worker-loop spawns — currently exists only as
60 lines of inline bash in `substrate-orchestrator-recipe.md` for
operators to copy-paste. Operators wiring the full pattern need a vetted
script, not a copy-paste recipe.

This task ships `scripts/substrate-worker-pickup.sh` — a long-running
inbox-poll loop that:

1. Polls `agent inbox --json` for unread `dm:*` topics
2. For each unread DM topic, reads the next envelope via
   `channel subscribe --resume --limit 1 --json` (so the cursor advances)
3. Parses `claim=X topic=Y offset=Z` from the orchestrator's payload
4. Optionally verifies ownership via `claims-summary` (defensive — in
   case the orchestrator's transfer raced or the DM is stale)
5. Spawns `substrate-worker-loop.sh --claim-id X --topic Y --offset Z`
   in adopted-claim mode against the operator-supplied --cmd template
6. Loops with a configurable poll interval

Pairs with: T-2148 orchestrator-loop (dispatch side), T-2146+T-2150
worker-loop (per-unit lifecycle), T-2151 smoke verifier (single-unit
correctness gate).

After this task the FULL canonical work-stealing pattern is two
scripts the operator runs: `substrate-orchestrator-loop.sh` on the
orchestrator host, `substrate-worker-pickup.sh` on each worker host.

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-worker-pickup.sh` exists, is executable, shellcheck-clean (exit 0).
- [x] `--help` documents all flags: required `--cmd 'TEMPLATE'`; optional `--worker-id ID`, `--hub addr`, `--poll-ms N` (default 2000), `--max-claims N` (default 0 = unlimited), `--test-parse 'PAYLOAD'` (diagnostic mode).
- [x] Auto-resolves `--worker-id` via T-1857 chain (flag → `$TERMLINK_AGENT_ID` → `~/.termlink/be-reachable.state` → refuse with hint exit 3). Verified via `resolve_worker_id` in script.
- [x] Parses orchestrator DM payload format `claim=X topic=Y offset=Z` (the format produced by `substrate-orchestrator-loop.sh` line 270). **Verified live (2026-06-11)**: `--test-parse "claim=clm-abc123 topic=smoke:work offset=42"` → `claim_id=clm-abc123 topic=smoke:work offset=42`, exit 0. Negative case `--test-parse "hello world"` → `parse-failed (missing field)`, exit 1.
- [x] Spawns `substrate-worker-loop.sh --claim-id X --topic Y --offset Z --claimer $WORKER_ID --cmd '<template>'` and waits for it. Verified by code read of main loop (lines after the dispatch parse): `"$WORKER_LOOP" --topic "$TOPIC" --offset "$OFFSET" --claim-id "$CLAIM_ID" --claimer "$WORKER_ID" --cmd "$CMD_TEMPLATE" ... &` followed by `wait "$CURRENT_WORKER_PID"`.
- [x] Live smoke: pickup loop runs against the local hub and processes existing DM topics correctly — payload_b64 decoded, dispatch-format classified, non-dispatch payloads loud-skipped with the original payload echoed. **Verified live (2026-06-11)**: 30s `--max-claims 1` run iterated 7 unread DM envelopes (mix of real-world non-dispatch content), each correctly classified as `non-dispatch DM on <topic> (skipped): <decoded text>`. SIGTERM exit 130 confirmed. End-to-end happy-path with foreign-sender dispatch DM is gated by T-1427 strict identity binding (cannot fake sender_id on a single host); the regression gate for the worker-loop spawn arguments is substrate-smoke.sh (T-2151) which exercises the same `--claim-id` composition.
- [x] SIGTERM/SIGINT exits cleanly with exit 130 (matches T-2146 + T-2148 signal convention). Verified — `trap 'cleanup; exit 130' INT TERM` + `cleanup` kills any in-flight worker pid.
- [x] Cross-ref added in `substrate-orchestrator-recipe.md`. Verified — "Ready-to-adapt pickup script (T-2152)" block sits between the standalone-worker example and the hand-rolled inline loop, pointing operators at the new script first.

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

shellcheck scripts/substrate-worker-pickup.sh
test -x scripts/substrate-worker-pickup.sh
bash scripts/substrate-worker-pickup.sh --help 2>&1 | grep -q "substrate-worker-pickup"
bash scripts/substrate-worker-pickup.sh --help 2>&1 | grep -q -- "--max-claims"
bash scripts/substrate-worker-pickup.sh --help 2>&1 | grep -q -- "--test-parse"
bash scripts/substrate-worker-pickup.sh --test-parse "claim=cid topic=t offset=0" 2>&1 | grep -q "claim_id=cid"

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

### 2026-06-11T06:51:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2152-substrate-worker-pickupsh--inbox-poll-dm.md
- **Context:** Initial task creation

### 2026-06-11T07:03:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
