---
id: T-2764
name: "Triage the stuck-claims canary firing that the worktree false all-clear was hiding"
description: >
  T-2763 fixed canary-status to read the main checkout, which immediately surfaced stuck-claims-canary as FIRING. It had been invisible from the worktree. Triage what is actually stuck, decide whether it is real (a dead claim holder) or canary noise, and either clear it or record why it is expected. This is the first finding the T-2763 fix produced, so it also serves as evidence the fix earns its keep.

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
created: 2026-08-16T13:35:12Z
last_update: 2026-08-16T13:38:55Z
date_finished: 2026-08-16T13:38:55Z
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

# T-2764: Triage the stuck-claims canary firing that the worktree false all-clear was hiding

## Context

T-2763 taught `/canaries` to read the main checkout instead of this worktree's
empty copy. The first thing it surfaced was `stuck-claims-canary` FIRING. This
task triaged it — and the answer turned out to be much larger than one canary.

## Verdict: CANARY DEFECT — fixed on this branch, NOT LIVE in cron

**Reproduced on demand:** `bash scripts/check-stuck-claims-freshness.sh` →
`healthy (772 topics, 0 stuck)`, exit 0. The canary does **not** fire when run
from this branch.

**But the log was appended today.** That contradiction is the finding.

The logged entries name 11 topics, and **every single one has `active=0`** —
nothing is held, so nothing can be stuck. `substrate-drain-demo` accumulates 81
then 90 expired rows across successive runs:

| topic | active | expired |
|---|---|---|
| substrate-drain-demo | 0 | 81 → 90 |
| substrate-drain-demo-1438568 / -1474600 / -1483085 | 0 | 15 each |
| substrate-drain-demo-1442195 / -1486453 | 0 | 12 each |
| substrate-drain-demo-1436872 / -1481552 | 0 | 9 each |
| drain-fix-verify-2169932 | 0 | 4 |
| drain-probe-1425555 / work-queue | 0 | 1 each |

That is the **exact signature CLAUDE.md already documents** for the pre-T-2709
monotonic latch: *"11 topics latched true, every one with `active_count: 0`
(nothing held, nothing that could be stuck), one carrying 81 expired rows."*
Expired rows are reaped lazily and only when the same `(topic, offset)` is
re-claimed, so on a topic nobody re-claims the row persists for the life of the
hub's SQLite and the verdict never clears.

**Which arm fired:** the old `expired_count > 0` arm. T-2709 replaced it with a
self-clearing `newest_expired_at_ms` recency test.

**Why it still fires despite T-2709.** Measured, not inferred:

```
git show main:scripts/check-stuck-claims-freshness.sh | grep -c newest_expired_at_ms   → 0
grep -c newest_expired_at_ms scripts/check-stuck-claims-freshness.sh                   → 3
```

`main` is at `19ba70a33` (2026-08-13). **This worktree branch is 225 commits
ahead of it.** Host cron runs from the MAIN checkout, so it executes `main`'s
copy of every canary script — the pre-T-2709 latching predicate included.

## The larger finding: the guard layer is shipped-but-not-live

This is not one stale canary. **Every canary and guard-layer fix made on this
branch — 225 commits — is dark in cron**, because cron executes `main`'s scripts
and `main` has not moved since 2026-08-13. The daily protection running on this
host is the 2026-08-13 version.

That is the **G-069 "shipped ≠ live"** class, which this repo has an entire
CLAUDE.md section and a dedicated gate (T-2480 `arc-live-probe.sh`) about — landed
here on the guard layer itself, which is the worst place for it: the layer whose
whole job is noticing that other things are dark had nobody watching whether IT
was live. The fleet-binary canary (T-2359) exists to catch exactly this shape for
hub BINARIES; nothing performs the equivalent check for the canary SCRIPTS.

It also explains the second `/canaries` observation — 4 STALE static-check
canaries (`alloc-sink`, `busy-spin`, `drain-sink`, `silent-exit`, heartbeats
2026-08-12/13). Their heartbeats stop right around `main`'s last commit.

## What was deliberately NOT done

- **No claim was force-released.** Nothing is actually stuck: every flagged topic
  has `active=0`. Tier-0 `claim-force-release` against debris would mutate live
  hub state to silence a false positive — the wrong direction entirely.
- **The canary log was NOT truncated.** It lives in the main checkout, which the
  T-559 boundary protects from this session. That gate is correct and I did not
  work around it. Its full contents are preserved verbatim above.
- **No merge to main.** Merging is outside what an agent may do here, and this
  one is 225 commits — a human decision, not a cleanup step.

## Operator actions

1. **Merge the branch to `main`** (the real fix — makes 225 commits of guard work
   live, T-2709's predicate among them).
2. **Then truncate the saturated log** so the one-bit channel is usable again:
   `: > /opt/termlink/.context/working/.stuck-claims-canary.log`
   Order matters: truncating first just lets tomorrow's cron re-fill it from the
   old predicate.
3. Consider a guard for the guard layer — nothing currently detects that cron is
   executing canary scripts N commits behind HEAD.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The canary's firing condition is reproduced on demand and the specific stuck topic(s), claim holder(s), lease age, and `active`/`expired` counts are recorded — named values, not "it fires"
- [x] Determined which arm of the T-2556 heuristic fired: the `newest_expired_at_ms` recency arm (T-2709's self-clearing replacement for the old monotonic `expired_count > 0` latch) or `oldest_active_age_ms > 60_000` — **the old `expired_count > 0` latch, running from `main`**
- [x] A verdict is recorded and justified: REAL vs EXPECTED vs CANARY DEFECT — **CANARY DEFECT, already fixed on this branch by T-2709 but not live because cron runs `main`, which is 225 commits behind**
- [x] If REAL: N/A — not real. Every flagged topic has `active=0`; nothing is held, so nothing could be stuck.
- [x] If EXPECTED or CANARY DEFECT: no state was mutated, and the reason is written down in full above
- [x] The post-triage canary state is re-measured and reported: run from this branch it exits **0** (`772 topics, 0 stuck`); run from `main` it will keep firing until the branch is merged, and the record says exactly why that is the correct outcome rather than something to silence
- [x] No claim belonging to a LIVE peer is force-released — nothing was released at all

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

# Run from this branch the canary is clean — the predicate fix works.
bash scripts/check-stuck-claims-freshness.sh > /dev/null 2>&1

# The branch carries T-2709's self-clearing predicate...
out=$(grep -c 'newest_expired_at_ms' scripts/check-stuck-claims-freshness.sh); [ "$out" -gt 0 ]

# ...and `main` — the tree cron actually executes — does NOT. This is the whole
# finding, so it is asserted mechanically rather than left as prose. When the
# branch is merged this leg flips and the task's conclusion is correctly stale.
p=$(mktemp); git show main:scripts/check-stuck-claims-freshness.sh > "$p" 2>/dev/null; out=$(grep -c 'newest_expired_at_ms' "$p" || true); rm -f "$p"; [ "$out" = "0" ]

# The branch is materially ahead of main (the shipped-but-not-live condition).
out=$(git rev-list --count main..HEAD); [ "$out" -gt 100 ]

## RCA

**Symptom:** `stuck-claims-canary` fires every day naming 11 topics as having
stuck claims. Run by hand from this branch the same check reports
`healthy (772 topics, 0 stuck)`.

**Root cause:** cron executes the MAIN checkout's copy of the script, and `main`
is 225 commits behind this branch. `main` still carries the pre-T-2709
`expired_count > 0` predicate — a monotonic latch that can never clear, because
expired claim rows are reaped only when the same `(topic, offset)` is re-claimed,
which on an abandoned demo topic never happens. Every flagged topic has
`active=0`: the canary is reporting on permanent debris.

**Why structurally allowed:** two distinct blindnesses compounded.

1. *Nothing checks that cron runs current canary code.* The repo has a canary for
   stale hub BINARIES (T-2359, with declared version floors), a canary for wakers
   running old code (T-2405), and a deploy-time gate for shipped≠live (T-2480).
   None of them look at the canary scripts themselves. The guard layer had no
   guard.
2. *The failure is self-concealing.* A canary firing daily on debris trains its
   operator to ignore it — and because "empty log = healthy" is a one-bit channel,
   the saturated log means a GENUINE stuck claim appended tomorrow would change
   nothing an operator can see. The canary is not merely noisy; it is **deaf**.
   That is precisely the harm T-2685 documents, arrived at by a different route:
   T-2685 was about stderr dirtying the log, this is about a stale predicate doing
   it.

**Why it took a fleet restart to find:** it didn't, quite — it took T-2763. This
was invisible from the worktree (`/canaries` read the empty local copy and
reported 0 firing) and would have stayed invisible. The chain was: restart the
fleet → check canaries → notice `/canaries` itself was lying → fix that → the
first thing it surfaced was this.

**Prevention:** the fix is a merge, which is a human action. The durable
prevention — something that detects cron executing canary scripts N commits behind
HEAD, the script-level analogue of the T-2359 binary floor — does not exist yet
and is NOT closed by this task. Recorded here rather than silently left out:
mitigation (explain the false positive) is not prevention (make it impossible to
recur), and this task delivers only the former.

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

### 2026-08-16T13:35:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2764-triage-the-stuck-claims-canary-firing-th.md
- **Context:** Initial task creation

### 2026-08-16T13:38:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
