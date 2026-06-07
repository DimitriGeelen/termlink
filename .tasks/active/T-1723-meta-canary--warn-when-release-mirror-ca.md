---
id: T-1723
name: "Meta-canary — warn when release-mirror-canary log is stale despite drift"
description: >
  Add a self-check to scripts/check-mirror-freshness.sh (or a sibling cron) so that if .context/working/.release-mirror-canary.log mtime is older than 2× cron-interval (≥48h for the daily canary) AND OneDev→GitHub HEAD divergence is non-zero, an alert fires. T-1721 surfaced the failure mode: the canary itself can be silently broken (wrong install location, bad PATH, parse error in /etc/cron.d/) and produce zero log entries even when drift is present — replicating the exact G-058 silent-failure pattern the canary was built to prevent. The meta-canary closes the recursion: 'the watcher is being watched'. Implementation options: (a) prepend a self-check to check-mirror-freshness.sh that stats its own log; (b) a separate scripts/check-canary-aliveness.sh; (c) a Watchtower panel that surfaces 'canary log mtime' next to 'drift status' so an operator sees both in one glance. Choice between (a)/(b)/(c) is the first design decision in the task.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [canary, meta, release, G-058, prevention, observability]
components: [scripts/check-canary-aliveness.sh, scripts/check-mirror-freshness.sh]
related_tasks: [T-1721, T-1696, T-1695]
created: 2026-05-20T07:07:06Z
last_update: 2026-06-06T12:54:51Z
date_finished: 2026-05-26T22:34:36Z
---

# T-1723: Meta-canary — warn when release-mirror-canary log is stale despite drift

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent

Design choice (decided 2026-05-27): **option (b) — sibling script + cron entry**,
augmented by a heartbeat-file touch added to the existing canary. The log-only
signal in the original task description is insufficient because the canary log
is append-on-drift-only: a healthy canary with no drift leaves the log mtime
stale-looking even when it's running fine. The heartbeat-touch fires on every
invocation regardless of drift, giving a clean "did the canary actually run?"
signal that the meta-check can stat.

- [x] `scripts/check-mirror-freshness.sh` touches `.context/working/.release-mirror-canary.heartbeat` on every invocation, BEFORE the network calls (so a network error doesn't suppress the heartbeat); `bash -n` passes. Also gains `--no-heartbeat` so the meta-canary can probe drift status without side-effecting the very signal it's checking (discovered during negative testing).
- [x] New `scripts/check-canary-aliveness.sh` exists, is executable, `bash -n` passes; exits 0 when heartbeat is fresh (≤48h), exits 1 when stale.
- [x] `.context/cron/release-mirror-canary.crontab` has a new line invoking `check-canary-aliveness.sh --quiet` daily (33 8 * * *, 80 min after the canary at 13 7 * * * so a load-time race can't take both); the file still parses as `/etc/cron.d/`-style USER-field syntax (15-field rows, header comment unchanged).
- [x] After a manual run of `check-mirror-freshness.sh` in this repo, the heartbeat file exists at `.context/working/.release-mirror-canary.heartbeat`.
- [x] `check-canary-aliveness.sh` against the fresh heartbeat exits 0. Negative test: backdated heartbeat to 72h → exit 1 with full diagnostic, AND the side-effect-free probe (via `--no-heartbeat`) preserved the stale mtime across multiple meta-canary invocations.

### Human

- [ ] [RUBBER-STAMP] Cron entry installed on .107 so the meta-canary actually fires.
  **Steps:**
  1. `sudo cp /opt/termlink/.context/cron/release-mirror-canary.crontab /etc/cron.d/termlink-release-mirror-canary`
  2. `sudo systemctl reload cron`
  3. `grep aliveness /etc/cron.d/termlink-release-mirror-canary`
  **Expected:** The grep returns the new meta-canary line.
  **If not:** Inspect `/etc/cron.d/termlink-release-mirror-canary` for syntax / permission issues; cron does NOT load files that are group/world-writable.

## Verification

bash -n scripts/check-mirror-freshness.sh
bash -n scripts/check-canary-aliveness.sh
test -x scripts/check-canary-aliveness.sh
bash scripts/check-mirror-freshness.sh --quiet || true
test -f .context/working/.release-mirror-canary.heartbeat
bash scripts/check-canary-aliveness.sh --quiet
grep -q "check-canary-aliveness.sh" .context/cron/release-mirror-canary.crontab

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

## Recommendation

**Recommendation:** GO — install the cron entry on .107.

**Rationale:** All five Agent ACs satisfied; 7/7 verification gate PASS in this
repo (heartbeat written, side-effect-free probe verified via `--no-heartbeat`,
crontab parses cleanly as 15-field USER-style rows). The only remaining step is
`sudo cp` + `systemctl reload cron` — mechanical, no judgment call. Once
installed, the meta-canary fires daily and closes the recursion T-1696 left
open (canary catches mirror drift; meta-canary catches canary failure).

**Evidence:**
- `scripts/check-mirror-freshness.sh` — added `HEARTBEAT_FILE` touch + `--no-heartbeat` flag (verified by negative test: backdated heartbeat to 72h preserved across multiple probe calls)
- `scripts/check-canary-aliveness.sh` — new script, GNU+BSD stat-compatible, exit codes 0/1/2 documented; tested live → `rc=0` on fresh, `rc=1` with full diagnostic on stale
- `.context/cron/release-mirror-canary.crontab` — new line `33 8 * * * root cd /opt/termlink && bash scripts/check-canary-aliveness.sh --quiet ...`, offset 80 min from the canary it watches
- `fw task verify T-1723` → 7/7 PASS

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

### 2026-06-06T13:05Z — Human AC fresh re-smoke for rubber-stamp click [agent autonomous]

Re-ran the Human AC verification steps verbatim on .107:

```
$ grep aliveness /etc/cron.d/termlink-release-mirror-canary
33 8 * * * root cd /opt/termlink && bash scripts/check-canary-aliveness.sh --quiet >> .context/working/.canary-aliveness.log 2>&1

$ ls -la .context/working/.release-mirror-canary.heartbeat
-rw-rw-r--+ 1 root root 0 Jun  6 11:19 .context/working/.release-mirror-canary.heartbeat
```

**PASS:** meta-canary cron line installed; heartbeat file is being touched fresh (mtime today 11:19Z = within 2h of this check). Box is ready to tick.

### 2026-05-20T07:07:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1723-meta-canary--warn-when-release-mirror-ca.md
- **Context:** Initial task creation

### 2026-05-26T22:29:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.4)

- **Scan ID:** R-a84342b2
- **Timestamp:** 2026-05-26T22:34:38Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#3 (Agent)** — `.context/cron/release-mirror-canary.crontab` has a new line invoking `check-canary-aliveness.sh --quiet` daily (33 8 * * *, 80 min after the canary at 13 7 * * * so a load-time race can't take both);
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: `.context/cron/release-mirror-canary.crontab` has a new line invoking `check-canary-aliveness.sh --quiet` daily (33 8 * * *, 80 min after the canary a`

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 4
     - evidence: `bash scripts/check-mirror-freshness.sh --quiet || true`

### 2026-05-26T22:34:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs verified 7/7; Recommendation block added; Human AC (cron install) pending operator
