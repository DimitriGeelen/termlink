---
id: T-2148
name: "Canonical substrate-orchestrator-loop.sh recipe script (T-2146 sibling)"
description: >
  Canonical substrate-orchestrator-loop.sh recipe script (T-2146 sibling)

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
created: 2026-06-10T22:03:01Z
last_update: 2026-06-10T22:03:01Z
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

# T-2148: Canonical substrate-orchestrator-loop.sh recipe script (T-2146 sibling)

## Context

T-2146 shipped `scripts/substrate-worker-loop.sh` — the "hello world"
for substrate WORKER users. Operators wiring the orchestrator side still
have only the prose pattern in T-2124 to crib from. Drop the matching
companion — `scripts/substrate-orchestrator-loop.sh` — that runs the
canonical 4-step dispatch loop (subscribe-stream → find-idle → claim
→ claim-transfer → DM) with proper signal handling and rollback when
mid-step failures leave a claim orphaned.

Composition of existing substrate verbs:
- `termlink channel subscribe TOPIC --resume` (work-queue stream)
- `termlink agent find-idle --capability X` (substrate primitive #2)
- `termlink channel claim TOPIC OFFSET` (substrate primitive #1)
- `termlink channel claim-transfer` (substrate primitive #3, T-2046)
- `termlink agent contact WORKER` (doorbell, T-1429)

The script is the dispatch counterpart to T-2146 — together they
cover both halves of the canonical work-stealing pattern.

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-orchestrator-loop.sh` exists, executable, shebang `#!/usr/bin/env bash`
- [x] `bash -n` passes
- [x] `--help` exits 0 and documents `--work-topic`, `--capability`, `--orchestrator-id`, `--ttl-ms`, `--idle-poll-ms`, `--hub`, `--max-envelopes` flags + exit-code table
- [x] Missing `--work-topic` exits 2 with stderr line `Usage: --work-topic required (see --help)`
- [x] Stream-driven loop: subscribes to work-topic with `--resume`, jq-parses one envelope at a time, dispatches per `.offset`
- [x] Find-idle backoff: when no idle worker matches `--capability`, sleeps `idle_poll_ms` then retries — never crashes the loop (wait_for_idle_worker helper)
- [x] Claim → claim-transfer → DM sequence per envelope. If `claim-transfer` fails after a successful `claim`, the claim is released (without --ack) so the slot reopens — no claim leak (cleanup path proven by IN_FLIGHT_CLAIM_ID state machine)
- [x] Signal handling: trap INT TERM → cleanup() releases IN_FLIGHT_CLAIM_ID + kills subscribe child + exits 130
- [x] `shellcheck` clean — `shellcheck scripts/substrate-orchestrator-loop.sh` produces no output
- [x] Master recipe doc gains a "Ready-to-adapt script (T-2148)" block at the top of the "Canonical orchestrator pattern" section
- [x] Smoke test on a real hub: orchestrator dispatched 1 envelope at offset 0 on smoke:t2146 → claim (clm-1781129246844580644...) → claim-transfer to root-claude-dimitrimintdev → DM attempted → exit clean at --max-envelopes=1. Worker-side release as root-claude-dimitrimintdev returned `{ack:false, ok:true}` proving the ownership transfer worked end-to-end. Active-count went 0 → 1 → 0 across the lifecycle.

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

test -x scripts/substrate-orchestrator-loop.sh
bash -n scripts/substrate-orchestrator-loop.sh
scripts/substrate-orchestrator-loop.sh --help > /tmp/.t2148.out 2>&1 && grep -q -- "--work-topic" /tmp/.t2148.out
out=$(scripts/substrate-orchestrator-loop.sh 2>&1 || true); echo "$out" | grep -q "Usage\|required"
grep -q "scripts/substrate-orchestrator-loop.sh" docs/operations/substrate-orchestrator-recipe.md

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

### 2026-06-10T22:03:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2148-canonical-substrate-orchestrator-loopsh-.md
- **Context:** Initial task creation
