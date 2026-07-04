---
id: T-2345
name: "Activate R2 presence retention on live hub — topic-growth canary firing"
description: >
  Activate R2 presence retention on live hub — topic-growth canary firing

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
created: 2026-07-04T08:57:59Z
last_update: 2026-07-04T08:57:59Z
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

# T-2345: Activate R2 presence retention on live hub — topic-growth canary firing

## Context

The T-2252 topic-growth canary FIRED on 2026-07-04: `agent-presence` at 30,804
records with `retention: forever` on the live :9100 hub (threshold 5000). Root
cause: arc-002 R2 (T-2244/T-2245 set-retention + sweep + `latest-per-cv-key`
compaction) shipped the VERBS but the live-hub ACTIVATION was left as a pending
operator step (arc-002 closeout note) and never ran — the exact "framework
relies on out-of-band hygiene that may never run" class the canary was built to
catch (T-1991 recurrence). The compaction policy itself was human-decided in the
T-2242 arc walkthrough (Q1, 2026-06-22): `latest-per-cv-key` is the R2 target
for presence. This task executes runbook
`docs/operations/agent-presence-retention-reset.md` §3 (set-retention + sweep)
and §4 (daily sweep cron — the bus runs no background sweep thread, so without
the cron the topic regrows and the canary re-fires).

## Acceptance Criteria

### Agent
- [x] `agent-presence` on the live hub has `retention.kind == latest_per_cv_key` (runbook §3 step 2 applied)
- [x] Post-sweep `agent-presence` count is under the 5000 canary threshold (runbook §3 step 3 enforced; pre-cv_key residue handled if present) — 30,804 → 4
- [x] Daily sweep cron installed to /etc/cron.d + tracked copy in `.context/cron/` (runbook §4 — prevents regrowth between sweeps)
- [x] `scripts/check-topic-growth-freshness.sh` exits 0 (canary no longer firing) — "healthy — no watched topic over 5000 records"

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
out=$(termlink channel list --json | jq -r '.topics[] | select(.name=="agent-presence") | .retention.kind'); echo "$out" | grep -q "latest_per_cv_key"
cnt=$(termlink channel list --json | jq -r '.topics[] | select(.name=="agent-presence") | .count'); [ "$cnt" -lt 5000 ]
test -f /etc/cron.d/termlink-presence-sweep
test -f .context/cron/presence-sweep.crontab
bash scripts/check-topic-growth-freshness.sh --quiet

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

### 2026-07-04 — pre-cv_key residue handling (two-pass sweep)
- **Chose:** After the first `latest-per-cv-key` sweep left 13,461 records (heartbeats
  predating T-2107 cv_key wiring — exempt from per-key compaction by design), applied an
  interim `days:2` pass (set-retention + sweep, pruned 13,457) and then restored
  `latest-per-cv-key` as the standing policy. Final: 4 records, all carrying cv_key.
- **Why:** Per-key compaction never drops non-cv_key records (runbook §2 — deliberate
  no-silent-drop), so the residue would have kept the canary firing forever. `days:2` was
  already the human-decided interim policy from the arc-002 Q1 walkthrough — reusing it
  for a one-time residue clear stays inside decided policy space.
- **Rejected:** `messages:N` pass (guessing N; prunes by position not age); leaving the
  residue and raising the canary threshold (defeats the canary); sqlite surgery (§6 legacy
  path — runbook explicitly deprecates it).

### 2026-07-04 — sweep cron scheduled at 04:17, before the 05:11 canary
- **Chose:** Daily sweep at 04:17 UTC in /etc/cron.d/termlink-presence-sweep.
- **Why:** The topic-growth canary runs 05:11; sweeping first means the canary always
  evaluates post-sweep state — a canary firing then genuinely means the sweep is broken,
  not that it just hasn't run yet today.
- **Rejected:** Runbook's verbatim 04:17-with-user-crontab example (house convention is
  /etc/cron.d with USER field + git-tracked copy in .context/cron/).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-04T08:57:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2345-activate-r2-presence-retention-on-live-h.md
- **Context:** Initial task creation
