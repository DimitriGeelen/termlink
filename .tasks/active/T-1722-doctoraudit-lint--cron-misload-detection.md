---
id: T-1722
name: "Doctor/audit lint — cron-misload detection (PL-173 enforcement)"
description: >
  Add a fw doctor / fw audit check that catches the T-1721 cron-misload class structurally instead of waiting for the next silent failure. PL-173 documents the bidirectional signature: (1) a cron file present in BOTH /etc/cron.d/<name> AND root's user crontab → cron will run the wrong one or both; (2) a source-of-truth crontab at .context/cron/*.crontab using /etc/cron.d/ USER-field syntax ('m h dom mon dow USER cmd') with no matching /etc/cron.d/ counterpart → the crontab is dormant. The lint walks .context/cron/*.crontab files, classifies each by syntax (USER field present/absent), and verifies the install destination is correct. Wire into fw doctor under the existing 'cron registry' family of checks. Outcome: T-1721's silent failure mode becomes impossible to ship undetected.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [cron, doctor, lint, PL-173, G-058, prevention]
components: []
related_tasks: [T-1721, T-1696]
created: 2026-05-20T07:06:58Z
last_update: 2026-05-26T22:44:25Z
date_finished: 2026-05-26T22:44:25Z
---

# T-1722: Doctor/audit lint — cron-misload detection (PL-173 enforcement)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent

Scope decision: MVP targets PL-173's case (2) — **dormant USER-field cron file**
(syntactically routed at `/etc/cron.d/` but never copied there). This is the
specific class that ran 16 days silently in G-058 and is what T-1696 / T-1723
exist to prevent at runtime. Case (1) — dual-install (file in `/etc/cron.d/`
AND root's user crontab) — needs richer parsing and is left to a follow-up
when there's an actual observed instance. The lint lives in `fw audit`'s
structure section (alongside the existing 'Cron registry' check) so it runs
on every `fw audit` invocation, including the project's own scheduled audit.

- [x] `.agentic-framework/agents/audit/audit.sh` walks `$PROJECT_ROOT/.context/cron/*.crontab` and, for each USER-field-syntax file (detected by awk-parsing for a row whose first field is cron-numeric and 6th field looks like a username), checks for a matching install in `/etc/cron.d/`. Matches via canonical-name heuristic (`<basename>`, `termlink-<basename>`, `<project-slug>-<basename>`) then a content-substring fallback if names don't match. Emits PASS when found, FAIL when not. The framework's own `agentic-audit.crontab` is excluded — it's covered by the existing registry check.
- [x] Live verification on this repo: lint emits PASS for `release-mirror-canary.crontab` (installed at `/etc/cron.d/termlink-release-mirror-canary`) and PASS for `heartbeat.crontab` (installed at `/etc/cron.d/termlink-heartbeat`).
- [x] Negative-tested by staging a temporary USER-field crontab in `.context/cron/__test-T1722.crontab` with no install, running the audit section, and confirming the FAIL appears with the install command in the suggestion. Then removing the test file and confirming the FAIL disappears.
- [x] Channel-1 mirror: identical patch landed at `/opt/999-Agentic-Engineering-Framework/agents/audit/audit.sh` on `origin/master` as commit `a3566cb3`. Verified via `git ls-remote https://onedev.docker.ring20.geelenandcompany.com/agentic-engineering-framework master` → `a3566cb37ea76bef4d10712141ba9cccb7a4110b`.
- [x] `bash -n .agentic-framework/agents/audit/audit.sh` passes; `fw audit --section structure` returns clean (no spurious WARN/FAIL added; existing PASS lines unaffected — 3 new PASS lines added).

### Human

- [ ] [RUBBER-STAMP] Upstream landed on `/opt/999-AEF` `origin/master`.
  **Steps:**
  1. From any host that can reach OneDev: `git ls-remote https://onedev.docker.ring20.geelenandcompany.com/agentic-engineering-framework master | awk '{print $1}'`
  2. Compare against the Channel-1 commit SHA recorded in Updates.
  **Expected:** OneDev master SHA matches the Channel-1 push.
  **If not:** Re-fire the Channel-1 push via `termlink_run` from /opt/termlink; check upstream OneDev for branch-protection rejection.

## Verification

bash -n .agentic-framework/agents/audit/audit.sh
test "$(.agentic-framework/bin/fw audit --section structure 2>&1 | grep -cE 'cron\(|Cron registry')" -ge 3

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

**Recommendation:** GO — verify upstream landing and close.

**Rationale:** Five Agent ACs all verified. Consumer copy at
`/opt/termlink/.agentic-framework/agents/audit/audit.sh` and upstream at
`/opt/999-AEF/agents/audit/audit.sh` are byte-equivalent (same Python
patcher applied to both); upstream pushed to OneDev master as `a3566cb3`.
Live test exercises the exact G-058 prevention case (dormant USER-field
crontab) and reports a FAIL with the install command. No spurious noise
added — only PASS lines for installed crons and a FAIL for genuinely
dormant ones.

**Evidence:**
- Consumer commit: `e89e94a6` on `/opt/termlink` `main`
- Upstream commit: `a3566cb3` on `/opt/999-AEF` `master` (verified via `git ls-remote`)
- Live audit output: 3 new PASS lines (heartbeat, release-mirror-canary; agentic-audit already covered by registry check)
- Negative test: stage `.context/cron/__test-T1722.crontab` (no install) → `[FAIL] cron(__test-T1722)` surfaces; remove → disappears

**Follow-up filed if needed:**
- PL-173 case (1) (dual-install in /etc/cron.d/ + user crontab) deferred to a future task when an actual instance is observed — current MVP catches the silent-failure mode that is the established G-058 cause.

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

### 2026-06-06T15:27Z — Human AC fresh re-smoke for [RUBBER-STAMP] click [agent autonomous]

Per `[Fresh re-smoke before rubber-stamp]` memory: task is 17 days old. The AC compares OneDev master SHA against the Channel-1 commit `a3566cb3`. Master has naturally moved on since (HEAD = `cb266216`), but the right semantic check is "is `a3566cb3` in master's history?" — which I ran:

```
$ git clone https://onedev.docker.ring20.geelenandcompany.com/agentic-engineering-framework /tmp/aef-vt
$ cd /tmp/aef-vt && git log --oneline a3566cb3 -1
  a3566cb37 T-1722: cron-misload lint — detect dormant USER-field crontab files

$ git branch --contains a3566cb3
  * master
```

**a3566cb3 IS on master** ✓. The Channel-1 push landed and has been retained across 13+ subsequent commits.

**Live local-side verification (Verification block reproduced):**
```
$ .agentic-framework/bin/fw audit --section structure | grep -cE 'cron\(|Cron registry'
5      ← passes "≥ 3" gate

$ .agentic-framework/bin/fw audit --section structure | grep -E 'cron'
[PASS] Cron registry in sync with /etc/cron.d/agentic-audit-termlink
[PASS] cron(fleet-adoption-snapshot): USER-field syntax installed at /etc/cron.d/termlink-fleet-adoption-snapshot
[PASS] cron(fleet-doorbell-mail-canary): USER-field syntax installed at /etc/cron.d/termlink-fleet-doorbell-mail-canary
[PASS] cron(heartbeat): USER-field syntax installed at /etc/cron.d/termlink-heartbeat
[PASS] cron(release-mirror-canary): USER-field syntax installed at /etc/cron.d/termlink-release-mirror-canary
```

**Suggestion to operator:** the AC's literal SHA-match check is fragile (assumes no upstream activity since push). Click anyway — the structural intent ("did the patch land?") is satisfied. Or amend the AC steps to use `git log --oneline a3566cb3 -1` against master before final closure.

### 2026-05-20T07:06:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1722-doctoraudit-lint--cron-misload-detection.md
- **Context:** Initial task creation

### 2026-05-26T22:37:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.4)

- **Scan ID:** R-5b51a70a
- **Timestamp:** 2026-05-26T22:44:30Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 3

**Per-AC findings:**

- **AC#1 (Agent)** — `.agentic-framework/agents/audit/audit.sh` walks `$PROJECT_ROOT/.context/cron/*.crontab` and, for each USER-field-syntax file (detected by awk-parsing for a row whose first field is cron-numeric and 6
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: `.agentic-framework/agents/audit/audit.sh` walks `$PROJECT_ROOT/.context/cron/*.crontab` and, for each USER-field-syntax file (detected by awk-parsing`
- **AC#2 (Agent)** — Live verification on this repo: lint emits PASS for `release-mirror-canary.crontab` (installed at `/etc/cron.d/termlink-release-mirror-canary`) and PASS for `heartbeat.crontab` (installed at `/etc/cro
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: Live verification on this repo: lint emits PASS for `release-mirror-canary.crontab` (installed at `/etc/cron.d/termlink-release-mirror-canary`) and PA`
- **AC#4 (Agent)** — Channel-1 mirror: identical patch landed at `/opt/999-Agentic-Engineering-Framework/agents/audit/audit.sh` on `origin/master` as commit `a3566cb3`. Verified via `git ls-remote https://onedev.docker.ri
  - **AC-verify-mismatch** (narrow, heuristic) — `path=opt/999-Agentic-Engineering-Framework/agents/audit/audit.sh in: Channel-1 mirror: identical patch landed at `/opt/999-Agentic-Engineering-Framework/agents/audit/audit.sh` on `origin/master` as commit `a3566cb3`. Ve`

### 2026-05-26T22:44:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Verification gate fixed (pipefail-safe with grep -c)
