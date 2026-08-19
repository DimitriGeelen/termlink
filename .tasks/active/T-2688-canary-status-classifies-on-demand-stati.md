---
id: T-2688
name: "canary-status classifies on-demand static checks as STALE forever, so /canaries can never report healthy"
description: >
  canary-status classifies on-demand static checks as STALE forever, so /canaries can never report healthy

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
created: 2026-08-19T23:39:52Z
last_update: 2026-08-19T23:39:52Z
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

# T-2688: canary-status classifies on-demand static checks as STALE forever, so /canaries can never report healthy

## Context

`/canaries` (T-2172, `scripts/canary-status.sh`) exists to give the operator one
green/red read on the cron-tier protection set. It cannot currently do that: it reports
`4 canary(ies) need attention (0 firing, 4 stale)` and exits 1 on a completely healthy
tree, and it has done so continuously since the checks were written.

The four are the **source-level static checks** — `alloc-sink-canary` (T-2527),
`drain-sink-canary` (T-2531), `silent-exit-canary` (T-2666), `busy-spin-canary`
(T-2672). CLAUDE.md documents each of them as, verbatim, "a source-level static check"
and "**NOT a runtime cron canary**"; they are meant to be run ad-hoc (and the T-2561
cron-install-drift check confirms none of them has, or should have, a crontab). But
`discover_canaries()` synthesizes a `.log` path from *any* `.*-canary.heartbeat`, so
their heartbeats pull them into the canary set, and `classify()` then measures them on
the one axis that is meaningless for an on-demand check: heartbeat age. Their
heartbeats are 2026-08-04, 08-12, 08-12, 08-13 — all past the 48h threshold, and always
will be, because nothing is supposed to run them daily.

Net effect: `PROBLEMS = FIRING + STALE` is permanently ≥ 4, so `/canaries` never exits
0, and four immovable yellow rows sit above any genuine STALE. That is precisely the
alarm-fatigue failure `/canaries` was built to remove — a green/red signal that is
always red carries no information, and the next real stale canary lands in a list the
operator has already learned to ignore.

Fix: an explicit, git-tracked registry of on-demand checks
(`.context/cron/ondemand-checks.conf`), and an `ON_DEMAND` classification that
suppresses **staleness only**. A registered check with findings still reports FIRING —
an on-demand check that found something is exactly as actionable as a cron one, so
suppression must never extend to its log content.

## Acceptance Criteria

### Agent
- [x] `.context/cron/ondemand-checks.conf` exists, is git-tracked, supports `#` comments,
      and registers the four static checks with a cited reason each
- [x] `canary-status.sh` classifies a registered check with an empty log as `ON_DEMAND`
      rather than `STALE`, regardless of heartbeat age
- [x] A registered check whose log has findings still classifies as `FIRING` —
      registration suppresses staleness, never findings
- [x] `ON_DEMAND` is excluded from the `PROBLEMS` count, so a healthy tree exits 0
- [x] An unregistered canary is still classified `STALE` on an aged heartbeat
      (no blanket suppression)
- [x] `--json` output carries the `on_demand` count in `summary` and `ON_DEMAND` as a
      per-canary `status`
- [x] `--quiet` does not render `ON_DEMAND` rows (they are not problems)
- [x] A fixture test drives all of the above against a scratch working-dir and passes
- [x] The suppression is proven registry-driven: the same scratch tree exits 1 when the
      registry is absent and 0 when it is present

**Not verifiable in this worktree (deferred to the main checkout):** end-to-end
`bash scripts/canary-status.sh` against the real `.context/working/`. The canary
heartbeats/logs are untracked, so they do not exist on this branch; and a canary cron
firing with cwd inside a worktree reports `release-mirror-canary: error: origin HEAD
empty` (the worktree branch has no upstream), which is a worktree artefact rather than
real mirror drift. The fixtures reproduce the exact real conditions instead — the same
four names, empty logs, heartbeats aged past the threshold.

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

bash -n scripts/canary-status.sh
bash tests/canary-status-ondemand-fixtures.sh

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

**Symptom:** `/canaries` reports `4 canary(ies) need attention (0 firing, 4 stale)` and
exits 1 on a healthy tree, permanently. The four named are `alloc-sink-canary`,
`busy-spin-canary`, `drain-sink-canary`, `silent-exit-canary`.

**Root cause:** Those four are source-level static checks, documented in CLAUDE.md as
"**NOT a runtime cron canary**" and deliberately not scheduled. They nonetheless write
`.context/working/.<name>-canary.heartbeat` when run. `discover_canaries()` synthesizes
a `.log` path from *any* `.*-canary.heartbeat` (T-2178, so that healthy never-fired
canaries are still surfaced), which pulls them into the canary set; `classify()` then
grades every set member on heartbeat freshness. For a check nothing runs daily, that age
only ever grows.

**Why structurally allowed:** the canary set is defined by a *filename convention*
(`.*-canary.*`) rather than by "is there a cron for this". Nothing connects the
discovery rule to the scheduling reality, so a check could adopt the naming convention
— reasonably, since it is conceptually a canary — and silently acquire a grading axis
that does not apply to it. `check-cron-install-drift.sh` (T-2561) knows which checks
have crontabs, but its knowledge was never wired to `canary-status.sh`.

**Prevention:** `.context/cron/ondemand-checks.conf` makes "this check is deliberately
not cron-backed" an explicit, git-tracked, reviewable declaration rather than an
implicit property of a heartbeat file, and `ON_DEMAND` grades those on findings alone.
`tests/canary-status-ondemand-fixtures.sh` pins the asymmetry that matters — suppression
covers staleness only, never findings — and proves it registry-driven by showing the
identical scratch tree exits 1 without the registry and 0 with it, so a future blanket
"ignore stale" shortcut fails the suite.

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

### 2026-08-19T23:39:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2688-canary-status-classifies-on-demand-stati.md
- **Context:** Initial task creation
