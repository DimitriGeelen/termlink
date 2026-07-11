---
id: T-2405
name: "Stale-waker-code detection canary — surface agents on pre-current push-waker code (G-019 detection for T-2404)"
description: >
  Detection counterpart to T-2404 fleet-rearm-wakers.sh (remediation). The framework is currently BLIND to agents running stale push-waker code — this session found the fleet on pre-Stage-3 wakers only via manual /proc mtime comparison. Build a check-waker-code-freshness.sh canary (same empty-log=healthy + cron pattern as the 9 existing canaries; sibling to T-2359 fleet-binary but at the waker-process layer): walk be-reachable-*.state, compare each running pushwaker_pid /proc mtime vs the live be-reachable-pushwaker.sh mtime, FIRE on any stale waker. Auto-discovered by /canaries. Remediation on fire: bash scripts/fleet-rearm-wakers.sh --all.

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
created: 2026-07-11T13:30:25Z
last_update: 2026-07-11T21:45:52Z
date_finished: 2026-07-11T21:45:52Z
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

# T-2405: Stale-waker-code detection canary — surface agents on pre-current push-waker code (G-019 detection for T-2404)

## Context

T-2404 shipped the REMEDIATION (`scripts/fleet-rearm-wakers.sh`) for agents running
stale push-waker code, but the framework has no DETECTION — the stale-waker-code class was
found only by a manual `/proc/<pid>` mtime compare. G-019 says: fix the symptom (T-2404), then
ask "why was the framework blind?" and close the blindness. This task adds the read-only canary
sibling: it walks the same per-agent `be-reachable-<id>.state` files T-2404 re-arms, classifies
each live push-waker by process-start-time vs the current waker-script mtime, and FIRES
(empty-log = healthy) when any running waker predates the current code. Sibling to T-2359
(fleet-binary-freshness, at the hub-binary layer) and T-2387 (waker-liveness, at the
waker-*running* layer) — this one is at the waker-*code-version* layer. Remediation-on-fire is
exactly `fleet-rearm-wakers.sh --all`. Reuses T-2404's staleness primitives verbatim
(`code_mtime` / `proc_start_mtime` / `is_stale`) so detection and remediation cannot drift apart.

## Acceptance Criteria

### Agent
- [x] **Canary script.** `scripts/check-stale-waker-code-freshness.sh` walks
  `$STATE_DIR/be-reachable-*.state` (STATE_DIR = `$HOME/.termlink`, override
  `STALE_WAKER_STATE_DIR`), and for each state file with a `pushwaker_pid`, classifies the
  running waker: **STALE** (pid alive AND `/proc/<pid>` start-mtime < current waker-script
  mtime — the firing class), **current** (alive AND not older — healthy), **not-running**
  (dead/absent pid — informational cleanup class, non-firing, mirrors T-2387/T-2239 dead-pid
  handling). FIRES (exit 1) on ANY stale waker; exit 0 when none stale; exit 2 on tooling error.
  Waker-script ref path overridable via `STALE_WAKER_PW_SCRIPT` (mirrors T-2404's
  `FLEET_REARM_PW_SCRIPT`). Each firing line names the agent + running-pid + proc-mtime vs
  code-mtime + the `fleet-rearm-wakers.sh <agent>` remediation.
- [x] **Canary conventions.** `--quiet` (print only on firing, cron form), `--json` (envelope
  `{ok, stale[], current[], not_running[], summary}`), `--no-heartbeat`, and a `.heartbeat`
  touch at `.context/working/.stale-waker-code-canary.heartbeat` — all matching the nine
  existing empty-log=healthy canaries so `/canaries` auto-discovers it. Pure helpers factored
  for unit test via a `STALE_WAKER_LIB=1` source-without-run guard.
- [x] **Cron + registry.** `.context/cron/stale-waker-code-canary.crontab` (daily, `--quiet`,
  USER-field syntax) authored AND installed to `/etc/cron.d/termlink-stale-waker-code-canary`
  (pre-push audit FAILs on an uninstalled `*-canary.crontab`); cron registry in sync (audit
  PASS for the new cron entry).
- [x] **Tests + doc.** Hermetic test `tests/stale-waker-code-canary.sh` — fixture state dir +
  fake waker script with controlled mtimes proving STALE fires, current is healthy, not-running
  is informational, and the JSON envelope shape. `cargo` untouched (pure shell). Doc
  `docs/operations/stale-waker-code-canary.md` + a CLAUDE.md canary §. Live `--json` on the
  real .107 fleet reported (expected: all current after T-2404 convergence).

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

### 2026-07-11T13:30:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2405-stale-waker-code-detection-canary--surfa.md
- **Context:** Initial task creation

### 2026-07-11T21:38:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-34338a72
- **Timestamp:** 2026-07-11T21:45:53Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#1 (Agent)** — **Canary script.** `scripts/check-stale-waker-code-freshness.sh` walks
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/check-stale-waker-code-freshness.sh in: **Canary script.** `scripts/check-stale-waker-code-freshness.sh` walks`
- **AC#4 (Agent)** — **Tests + doc.** Hermetic test `tests/stale-waker-code-canary.sh` — fixture state dir +
  - **AC-verify-mismatch** (narrow, heuristic) — `path=tests/stale-waker-code-canary.sh in: **Tests + doc.** Hermetic test `tests/stale-waker-code-canary.sh` — fixture state dir +`

### 2026-07-11T21:45:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
