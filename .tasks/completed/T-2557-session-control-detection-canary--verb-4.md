---
id: T-2557
name: "session-control detection canary — verb-4 daily-detection gap"
description: >
  session-control detection canary — verb-4 daily-detection gap

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T07:46:50Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-09T07:50:04Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:51Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=2 (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2557: session-control detection canary — verb-4 daily-detection gap

## Context

Sibling of T-2556. The T-2468 charter-verb-completeness review found the canary
axis covered only verbs 1-2. **Verb 4 — "control terminal sessions"** (the founding
verb: spawn a PTY, inject a command, capture output) has an affirmative prover
(`scripts/session-selftest.sh`, T-2485: SPAWN + EXEC sentinel round-trip + CLEANUP)
but ZERO passive daily detection. Session spawn/exec could break and only a manual
`session-selftest.sh` run would catch it. The prover is deterministic + local (uses
`exec --json`, no live peer needed) and self-reaps, making it ideal canary substrate.
This canary wraps it: selftest exit 0 → healthy, exit 1 → FIRE (verb-4 genuinely
broken, broken_stage named), exit 2 → tooling (hub down / no tmux — NOT a verb-4
regression; that is preflight territory, non-firing).

## Acceptance Criteria

### Agent
- [x] `scripts/check-session-control-freshness.sh` exists, is executable, runs `session-selftest.sh --json`, and maps: selftest exit 0 → canary exit 0 (healthy); selftest exit 1 → canary exit 1 (FIRE, naming `broken_stage`); selftest exit 2 → canary exit 2 (tooling, non-firing — hub-down/dep-missing is preflight territory, not a verb-4 regression)
- [x] `--json`, `--quiet`, `--no-heartbeat` flags present; heartbeat touched FIRST at `.context/working/.session-control-canary.heartbeat`
- [x] Test hook feeds a canned selftest result (JSON + rc) so all three branches are verifiable without a hub/tmux (`TERMLINK_SESSION_CANARY_TEST_JSON` + `TERMLINK_SESSION_CANARY_TEST_RC`)
- [x] `.context/cron/session-control-canary.crontab` exists, runs the script `--quiet` daily, appends to `.context/working/.session-control-canary.log`
- [x] Load-bearing PROVEN: a canned "broken" selftest result makes the canary exit 1; a "proven" result exits 0; a "tooling" result exits 2 (temp-revert the firing map → the broken fixture stops firing)
- [x] CLAUDE.md documents the canary in a new "### Session-control canary" section matching the existing convention

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
test -x scripts/check-session-control-freshness.sh
# broken selftest verdict fires (exit 1)
f=$(mktemp); printf '{"ok":false,"proven":false,"broken_stage":"exec"}' > "$f"; TERMLINK_SESSION_CANARY_TEST_JSON="$f" TERMLINK_SESSION_CANARY_TEST_RC=1 bash scripts/check-session-control-freshness.sh --no-heartbeat >/dev/null 2>&1; rc=$?; rm -f "$f"; test "$rc" -eq 1
# proven verdict healthy (exit 0)
f=$(mktemp); printf '{"ok":true,"proven":true,"broken_stage":null}' > "$f"; TERMLINK_SESSION_CANARY_TEST_JSON="$f" TERMLINK_SESSION_CANARY_TEST_RC=0 bash scripts/check-session-control-freshness.sh --no-heartbeat --quiet >/dev/null 2>&1; rc=$?; rm -f "$f"; test "$rc" -eq 0
# tooling verdict does NOT fire (exit 2, not 1)
f=$(mktemp); printf '{"ok":false,"proven":false,"broken_stage":"tooling"}' > "$f"; TERMLINK_SESSION_CANARY_TEST_JSON="$f" TERMLINK_SESSION_CANARY_TEST_RC=2 bash scripts/check-session-control-freshness.sh --no-heartbeat >/dev/null 2>&1; rc=$?; rm -f "$f"; test "$rc" -eq 2
test -f .context/cron/session-control-canary.crontab
grep -q "Session-control canary (T-2557" CLAUDE.md

## RCA

**Symptom:** Verb 4 ("control terminal sessions") — the founding verb — could
regress in spawn/exec and only a manual `session-selftest.sh` run would catch it;
nothing fired passively.

**Root cause (blindness class):** same "prover ≠ canary" asymmetry as T-2556. The
project built an affirmative prover for every charter verb but a detection canary
for only two of four. Verb 4 had a deterministic, self-reaping prover already —
ideal canary substrate — but it was never wired to a cron.

**Why structurally allowed:** canaries accreted per-incident, not per-charter-surface;
nothing audited "does each core verb have BOTH a prover and a canary?" until the
T-2468 charter-verb-completeness review.

**Prevention:** this canary closes the verb-4 cell (T-2556 closed verb-3); all four
charter verbs now have BOTH a prover and a canary. The canary reuses the existing
prover verbatim, so detection cannot drift from the prover's definition of "works".
The tooling/broken split (selftest exit 2 → non-firing) keeps the firing log
meaningful — it fills only on a genuine verb-4 regression, never on a transient
hub-down.

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

### 2026-08-09T07:46:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2557-session-control-detection-canary--verb-4.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-cda5d9f8
- **Timestamp:** 2026-08-09T07:50:05Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -f`

### 2026-08-09T07:50:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
