---
id: T-2416
name: "woken-but-silent canary never self-heals — re-verify-and-clear triage"
description: >
  woken-but-silent canary never self-heals — re-verify-and-clear triage

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
created: 2026-07-18T07:58:20Z
last_update: 2026-07-18T08:11:19Z
date_finished: 2026-07-18T08:11:19Z
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

# T-2416: woken-but-silent canary never self-heals — re-verify-and-clear triage

## Context

Closes G-085: the woken-but-silent canary (G-083 detect-side) writes append-only
entries at send time and never re-evaluates them, so a false-positive (pre-T-2413
`msg_type=turn` blindness) or a late-arriving reply (pre-T-2414 tight window) keeps
`/canaries` permanently RED on stale residue. On 2026-07-18 all 5 live-log entries
re-verified as CONSUMED against the current rail — 100% false residue. Fix: a
re-verify-and-clear triage (`scripts/woken-silent-triage.sh`) that re-runs the live
matcher (`wake-confirm.sh`) over each logged entry, archives the ones now CONSUMED,
keeps only genuinely-still-silent ones, plus a daily cron so late replies self-clear
(PL-168: a manual-only tool is dormant, not prevention).

## Acceptance Criteria

### Agent
- [x] `scripts/woken-silent-triage.sh` parses each framed log entry (cid, topic, `hub=` if present, `offset=N`) and re-runs `wake-confirm.sh` per entry, partitioning into RESOLVED (exit 0 / CONSUMED) vs STILL-SILENT (exit 3)
- [x] `--apply` rewrites the live log with only STILL-SILENT entries and appends RESOLVED entries (with a `resolved-at` note + the offset the reply was found at) to `.woken-but-silent-canary.resolved.log`; default (no `--apply`) is report-only (no mutation)
- [x] `--json` emits `{ok, resolved[], still_silent[], summary}`; exit 0 when nothing still-silent, 1 when ≥1 still-silent remains, 2 on tooling error
- [x] PL-213 test seam: `WOKEN_TRIAGE_CONFIRM_CMD` overrides the matcher invocation so the triage is verified hub-independently (stub returns canned per-cid verdicts)
- [x] `tests/woken-silent-triage.sh` covers: all-resolved→green, mixed→keeps silent one, `--apply` archives + shrinks live log, report-mode is non-mutating, malformed entry skipped, `--hub` entry forwards `--hub`; ALL PASS
- [x] Daily cron (`.context/cron/woken-silent-triage.crontab`) runs `--apply --quiet` + a meta-canary aliveness companion, installed to `/etc/cron.d`
- [x] Live proof: running the triage against the real fleet clears all 5 current false-positive entries and returns the canary to green (`/canaries` no longer FIRING on woken-but-silent)

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
bash -n scripts/woken-silent-triage.sh
bash tests/woken-silent-triage.sh
# wake-confirm matcher must remain untouched (no regression to the detect side)
bash -n scripts/wake-confirm.sh

## RCA

**Symptom:** `/canaries` shows `woken-but-silent-canary` FIRING; all 5 logged
entries re-verify as CONSUMED against the live rail — the canary is red purely on
stale false-positive residue.

**Root cause:** `escalate_woken_but_silent` (agent-send.sh) writes an entry once,
at send time, and nothing ever re-evaluates it. False entries from since-fixed bugs
(T-2413 `msg_type=turn` blindness, T-2414 tight window) persist forever.

**Why structurally allowed:** the log is append-only with no re-verification or ack
path. G-083 built the detect side but not the resolve side — the framework can see a
silent send but is blind to that send later being answered (the mirror gap, G-085).

**Prevention:** `scripts/woken-silent-triage.sh` re-runs the live matcher over each
entry and archives the now-CONSUMED ones; a daily cron (`--apply --quiet`) makes late
replies self-clear within a day (PL-168 — a manual-only tool is dormant, not
prevention). Distinct from the fix: the fix clears today's residue; the cron stops it
recurring.

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

### 2026-07-18T07:58:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2416-woken-but-silent-canary-never-self-heals.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-529279cd
- **Timestamp:** 2026-07-18T08:11:21Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#6 (Agent)** — Daily cron (`.context/cron/woken-silent-triage.crontab`) runs `--apply --quiet` + a meta-canary aliveness companion, installed to `/etc/cron.d`
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: Daily cron (`.context/cron/woken-silent-triage.crontab`) runs `--apply --quiet` + a meta-canary aliveness companion, installed to `/etc/cron.d``

### 2026-07-18T08:11:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
