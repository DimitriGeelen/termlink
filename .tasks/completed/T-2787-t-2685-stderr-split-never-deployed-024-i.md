---
id: T-2787
name: "T-2685 stderr-split never deployed: 0/24 installed crontabs carry it, and the drift check calls them 'not scheduled'"
description: >
  T-2685 stderr-split never deployed: 0/24 installed crontabs carry it, and the drift check calls them 'not scheduled'

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/check-cron-install-drift.sh, tests/cron-install-drift-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-17T15:11:14Z
last_update: 2026-08-17T15:18:52Z
date_finished: 2026-08-17T15:18:52Z
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

# T-2787: T-2685 stderr-split never deployed: 0/24 installed crontabs carry it, and the drift check calls them 'not scheduled'

## Context

Two findings, one root cause, from running `check-cron-install-drift.sh` on this host.

**Finding A — real, and human-actionable.** T-2685 replaced `>> <log> 2>&1` with
`>> <log> 2>> <log>.stderr` across all canary crontabs, because merging stderr into the
findings log destroys the "empty log = healthy" one-bit channel: a tooling error dirties
the log, and every subsequent genuine finding appends to an already-non-empty file and
changes nothing an operator can see. That fix is in git and **was never deployed**:

    grep -l "2>&1"      /etc/cron.d/termlink-*  →  23
    grep -l "log.stderr" /etc/cron.d/termlink-* →   0   (of 24 installed)

So the defect T-2685 exists to prevent is still live on this host. Deploying needs root
(`sudo cp`), so it is surfaced, not performed.

**Finding B — a false claim in our own guard.** The check reports those 21 crontabs as:

    21 installed crontab(s) are MISSING JOB LINES that git declares — SHIPPED BUT DARK (G-069)
      The scheduled work below exists in git but is NOT scheduled on this host:

That is **false**. The work *is* scheduled — every one of those jobs runs on its cron
schedule. What differs is the redirect. `job_lines()` compares whole lines with `grep -Fxv`
(`:139`), so any command change reads as "line absent", and absence is then reported as
"not scheduled".

The effect is inverted severity: an operator reading "21 canaries SHIPPED BUT DARK" would
conclude the entire canary layer is dead and go re-install 21 crontabs, when the true
finding is "one redirect fix is undeployed". A guard that misreports *which* thing is
broken costs more than one that stays quiet — it sends the operator at the wrong target.

**This does not weaken the guard.** Redirect drift is a genuine deployment gap and must
still FIRE — T-2685's whole point is that the redirect is load-bearing. Only the claim
changes: from "not scheduled" to "scheduled, but the installed command differs".

## Acceptance Criteria

### Agent
- [x] New `JOB_DRIFT` class: a git job line whose redirect-stripped form matches an
      installed job line's, but which differs textually (the T-2685 shape)
- [x] `JOB_DRIFT` still FIRES (exit 1) — the deployment gap is real; only the claim changes
- [x] The `JOB_DRIFT` message does NOT assert "NOT scheduled on this host", and shows both
      the git form and the installed form so the operator can see what actually differs
- [x] A genuinely-absent job line (no schedule match at all) still classifies as
      `UNINSTALLED_JOBS` with its original "NOT scheduled on this host" wording
- [x] `--json` carries the new class (`job_drift_count` + `job_drift[]`)
- [x] `bash tests/cron-install-drift-fixtures.sh` passes, with new assertions covering
      redirect-drift-is-JOB_DRIFT and genuinely-absent-is-still-UNINSTALLED_JOBS
- [x] Real-tree run recorded in Evidence: the 21 reclassify from UNINSTALLED_JOBS to
      JOB_DRIFT, and the check still exits non-zero
- [x] Finding A surfaced to the human as a single copy-pasteable install command (T-609),
      since deploying to `/etc/cron.d` needs root

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
       Conversion: this AC should be moved to ### Agent and this line added to
       ## Verification (herestring, not a pipeline — see the L-387 hint below):
         out=$(bin/fw reviewer T-XXX 2>&1 || true); grep -q "Overall:.*PASS" <<< "$out"
-->

## Evidence

### The reclassification, and what it un-buried

Real tree, `bash scripts/check-cron-install-drift.sh --json`, exit **1**:

| class | before T-2787 | after |
|---|---|---|
| `missing_count` | 0 | 0 |
| `uninstalled_jobs_count` | **21** | **2** |
| `job_drift_count` | — | **21** |
| `drift_count` | 0 | 0 |
| `ok_count` | 3 | 3 |

The headline number barely moved, but the *meaning* changed completely — and the fix
recovered a signal the old output was concealing. The 2 genuinely-unscheduled job lines
are:

- `fleet-doorbell-mail-canary.crontab` — meta-canary aliveness line never scheduled
- `substrate-preflight-canary.crontab` — same

Those are **exactly the pair T-2682 originally found** (CLAUDE.md records them as T-2175 /
T-2176). They had regressed to invisible: buried inside a list of 21, 19 of which were
false. That is the real cost of the misclassification — not the wrong wording, but that a
true finding was indistinguishable from noise the guard itself generated.

### Finding A — T-2685 is undeployed (human/root)

```
grep -l "2>&1"       /etc/cron.d/termlink-*  →  23
grep -l "log.stderr" /etc/cron.d/termlink-*  →   0   (of 24 installed)
```

Every canary on this host still merges stderr into its findings log. Per T-2685 that
means a single tooling error permanently dirties the "empty log = healthy" channel, after
which a genuine finding appends to an already-non-empty file and changes nothing an
operator can see — the canary is not merely noisy but *deaf* until someone truncates it by
hand. Deploying needs root, so it is surfaced rather than performed (see the Human note
below).

### Not weakening the guard

`JOB_DRIFT` fires unconditionally, exactly as `UNINSTALLED_JOBS` does — `--strict` is
irrelevant to both. Fixture 12 is the explicit control: a genuinely-absent job line must
still classify as `UNINSTALLED_JOBS` with its original "NOT scheduled on this host"
wording, and must NOT be swallowed by the new class. Fixture 13 proves one crontab can
carry both classes at once, which the real `fleet-doorbell-mail` does.

Fixtures: **37 passed, 0 failed** (was 24). Guard layer: **46/46 PASS**.

## Verification

bash tests/cron-install-drift-fixtures.sh
# The new class exists and is wired into the JSON envelope.
out=$(bash scripts/check-cron-install-drift.sh --json 2>&1 || true); grep -q "job_drift_count" <<< "$out"
# The genuinely-absent class survives — the control that this did not weaken the guard.
out=$(bash tests/cron-install-drift-fixtures.sh 2>&1 || true); grep -q "genuinely-absent job is still UNINSTALLED_JOBS" <<< "$out"
out=$(bash tests/cron-install-drift-fixtures.sh 2>&1 || true); grep -q "0 failed" <<< "$out"
# Redirect drift no longer claims the job is unscheduled.
out=$(bash tests/cron-install-drift-fixtures.sh 2>&1 || true); grep -q "job-drift does not claim the job is unscheduled" <<< "$out"
# Docs record the new class.
out=$(cat CLAUDE.md); grep -q "Why JOB_DRIFT is its own class" <<< "$out"
# Guard layer still clean.
out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -q "guard layer: PASS" <<< "$out"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387, corrected by T-2775): P-011 runs each command
# under `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141
# (SIGPIPE) when grep matches and closes stdin while the upstream is still
# writing — verification then "fails" BECAUSE the check succeeded, and the
# earlier the match, the more reliably it fails.
#
# USE ONE OF THESE — both measured rc=0 at 3M lines:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# The herestring is preferred: a herestring spawns no producer process, so there
# is nothing to SIGPIPE and it cannot regress as output grows. In the second form
# the pipeline sits inside a command substitution, whose status is discarded — the
# OUTER `test` decides.
#
# DO NOT capture-then-pipe. This template previously prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# and it is size-dependent, not safe: `echo`/`printf` is a producer like any
# other, so once $out exceeds the pipe buffer it is still writing when `grep -q`
# exits and pipefail propagates 141. The capture bounds the DATA but does not
# remove the PRODUCER. Anything wrapping `cargo test`, `fleet doctor --json`, or a
# full log is already in that size range. (T-2775 measured this; 999-AEF L-613 and
# 050-email-archive PL-161 published the capture-then-pipe form before the
# correction — both have since adopted the herestring.)
#
# Corollary (T-2090): intermediate stages are just as fatal — `... | tail -3 |
# grep -q PAT` re-introduces the same risk. With a herestring the question does
# not arise; grep scans the whole captured string anyway.
#
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before the hint;
# T-2775 then measured 1490 exposed lines across 802 tasks despite the hint, which
# is why `scripts/check-verification-pipefail.sh` now enforces it structurally.
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

### 2026-08-17T15:11:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2787-t-2685-stderr-split-never-deployed-024-i.md
- **Context:** Initial task creation

### 2026-08-17T15:18:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
