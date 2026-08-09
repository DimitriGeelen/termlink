---
id: T-2556
name: "stuck-claims detection canary — verb-3 claim-work daily-detection gap"
description: >
  stuck-claims detection canary — verb-3 claim-work daily-detection gap

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
created: 2026-08-09T07:41:27Z
last_update: 2026-08-09T07:41:27Z
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

# T-2556: stuck-claims detection canary — verb-3 claim-work daily-detection gap

## Context

Charter-verb completeness review (T-2468 purpose campaign) found the substrate's
four core verbs each have an affirmative on-demand PROVER, but the daily-detection
CANARY axis covers only verbs 1 (discover) and 2 (exchange messages). **Verb 3
(claim work) has the RICHEST prover (`substrate-smoke.sh` — full claim→renew→
release with ownership enforcement) yet ZERO passive detection**: a claim can sit
expired, or a work-topic can accumulate stuck claims, for days with nothing firing.
The substrate already ships the detector primitive (`channel claims-summary --all
--only-stuck --json`, T-2076, using the T-2042 stuck heuristic: `expired_count>0
OR oldest_active_age_ms>60_000`). This task wraps it in the standard canary shape
(empty-log = healthy) — the same pattern as the 13 existing canaries.

## Acceptance Criteria

### Agent
- [x] `scripts/check-stuck-claims-freshness.sh` exists, is executable, and FIRES (exit 1) when a watched topic has `potentially_stuck` claims; exits 0 (healthy) when `stuck_count==0`; exit 2 on tooling error (hub unreachable / malformed JSON)
- [x] Firing gate is `stuck_count > 0` from the `claims-summary --all --only-stuck --json` envelope; per-topic fetch errors (`ok:false` entries) are surfaced but do not themselves fire (parity with the "fetch errors could mask a stuck topic" note)
- [x] `--json`, `--quiet`, `--no-heartbeat` flags present; heartbeat touched FIRST (before the check) at `.context/working/.stuck-claims-canary.heartbeat` so `/canaries` can prove it ran on a healthy cycle
- [x] Test hook `TERMLINK_STUCK_CLAIMS_TEST_JSON=<file>` feeds canned `claims-summary` JSON for hub-independent verification (PL-213)
- [x] `.context/cron/stuck-claims-canary.crontab` exists, runs the script `--quiet` daily, appends to `.context/working/.stuck-claims-canary.log`
- [x] Load-bearing PROVEN: a fixture with a stuck topic makes the script exit 1; a clean fixture exits 0 (temp-revert the firing gate → the stuck fixture stops firing)
- [x] CLAUDE.md documents the canary in a new "### Stuck-claims canary" section matching the existing canary-section convention

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
test -x scripts/check-stuck-claims-freshness.sh
# stuck fixture fires (exit 1)
sp=$(mktemp -d); printf '{"ok":true,"topic_count":3,"stuck_count":1,"shown":1,"only_stuck":true,"topics":[{"ok":true,"topic":"work-queue","active_count":2,"expired_count":1,"oldest_active_age_ms":95000,"potentially_stuck":true}]}' > "$sp/stuck.json"; TERMLINK_STUCK_CLAIMS_TEST_JSON="$sp/stuck.json" bash scripts/check-stuck-claims-freshness.sh --no-heartbeat >/dev/null; test $? -eq 1
# clean fixture healthy (exit 0)
printf '{"ok":true,"topic_count":3,"stuck_count":0,"shown":0,"only_stuck":true,"topics":[]}' > "$sp/clean.json"; TERMLINK_STUCK_CLAIMS_TEST_JSON="$sp/clean.json" bash scripts/check-stuck-claims-freshness.sh --no-heartbeat --quiet; test $? -eq 0
test -f .context/cron/stuck-claims-canary.crontab
out=$(cat CLAUDE.md); echo "$out" | grep -q "Stuck-claims canary (T-2556"

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

**Symptom:** Verb 3 ("claim work") — the substrate's core work-coordination
primitive — could have expired/stuck claims accumulate on any work-topic for days
with nothing firing. An idle worker's slot never reopens; a crashed worker's lease
is never surfaced.

**Root cause (blindness class, not a code bug):** the project built an affirmative
PROVER for every charter verb but a detection CANARY for only two of four. Verb 3
had the richest prover (substrate-smoke full lifecycle) yet zero passive/retrospective
detection — the "prover ≠ canary" asymmetry.

**Why structurally allowed:** canaries were added reactively (one per incident/gap),
so coverage tracked incidents rather than the charter surface. Nothing audited
"does each core verb have BOTH a prover and a canary?" — the charter-verb-completeness
review (T-2468) is what surfaced it.

**Prevention:** this canary closes the verb-3 cell; the sibling T-2557 closes verb-4.
The detector reuses the substrate's own `claims-summary --only-stuck` heuristic
(T-2042/T-2076) so detection cannot drift from the primitive's definition of "stuck".

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

### 2026-08-09T07:41:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2556-stuck-claims-detection-canary--verb-3-cl.md
- **Context:** Initial task creation
