# arc-008 cycle 1 — complete finding census

Captured: 2026-09-09T17:53:56Z  |  VERSION 0.11.1879  |  commit 2ea686b22

Verbs run: `fw audit` (full, all sections) and `fw doctor`.
Audit summary: === SUMMARY === Pass: 368 Warn: 77 Fail: 5  
Doctor summary: 14 warn line(s), 0 failures.

This file is the durable record. Every raw line below is a finding, whether or not it
has become a task yet. Nothing here may be dropped without an explicit decision.

## Audit FAIL (5)

- [FAIL] Cron drift: /opt/termlink/.context/cron/agentic-audit.crontab differs from deployed /etc/cron.d/agentic-audit-termlink
- [FAIL] cron(substrate-smoke-canary): USER-field syntax but no install in /etc/cron.d
- [FAIL] D2: Human review queue — 57 task(s) waiting >30d: T-1417(132d) T-1419(132d) T-1435(131d) T-1442(130d) T-1482(128d) T-1483(128d) T-1484(128d) T-1485(128d) T-1486(128d) T-1487(128d) T-1488(128d) T-1489(128d) T-1490(128d) T-1491(128d) T-1492(128d) T-1493(128d) T-1494(127d) T-1495(127d) T-1496(127d) T-1498(127d) T-1499(127d) T-1500(127d) T-1501(127d) T-1502(127d) T-1506(127d) T-1529(127d) T-1530(127d) T-1531(127d) T-1532(127d) T-1533(127d) T-1534(127d) T-1535(127d) T-1536(127d) T-1537(127d) T-1557(127d) T-1558(127d) T-1559(127d) T-1570(127d) T-1635(116d) T-1673(114d) T-1691(114d) T-1695(88d) T-1696(114d) T-1722(105d) T-1723(105d) T-1795(110d) T-2013(88d) T-2014(88d) T-2209(88d) T-2210(88d) T-2211(88d) T-2212(88d) T-2213(88d) T-2297(69d) T-2385(62d) T-2402(60d) T-2408(59d) T-2409(16d) T-2723(25d) T-2822(16d) T-2836(16d)
- [FAIL] D8: Handover quality — LATEST.md has 5 [TODO] sections
- [FAIL] D8b: Handover archive rot — 10/10 recent handovers have unfilled [TODO]s

## Audit WARN (77 raw lines)

- [WARN] All onboarding-seed corpus references resolve to existing maps — NOT EVALUATED: candidate set empty (0 onboarding-seed corpus reference(s))
- [WARN] Arc 'mcp-slimming' has no task commits in the last 30 days (2 task(s) in arc)
- [WARN] free driver F-ORCH: retire_when condition appears met (T-1643 completed cleanly OR G-064 closed) — review whether to retire
- [WARN] No PROJECT_ROOT resolution of framework-owned assets (T-2648, OBS-097) — NOT EVALUATED: candidate set empty (0 Python file(s) under web/ + lib/)
- [WARN] No stale-slice-references (L-417) — NOT EVALUATED: candidate set empty (0 file(s) under web/templates web/blueprints lib)
- [WARN] Found 77 GO-scope-not-propagated inception(s) of 158 GO-recorded completed inception(s) examined — GO recorded, related_tasks empty, nobody back-references, no unlocks_inception_decision
- [WARN] Fabric: 326/491 cards have no edges
- [WARN] Fabric: 10 card(s) point at files no watch pattern covers
- [WARN] Task T-2815-audit-cron-drift-slug-uses-worktree-base.md missing Updates section
- [WARN] Task T-2819-narrow-the-stale-agentic-framework-gitig.md missing Updates section
- [WARN] Task T-2822-blanket-contextworking-gitignore-makes-s.md missing Updates section
- [WARN] 4 active task(s) have every Agent AC ticked and no Human AC outstanding, but are still started-work/issues
- [WARN] Uncommitted changes present
- [WARN] Gate-bypass log: 7 safety bypasses in last 7 days (+ 0 drift overrides)
- [WARN] 7 episodics have empty or TODO summaries
- [WARN] Learnings ready for promotion — review graduation candidates
- [WARN] C-001: Inception T-1635 (recommendation) has no research artifact in docs/reports/
- [WARN] C-001: Inception T-2422 (recommendation) has no research artifact in docs/reports/
- [WARN] C-001: Inception T-2423 (recommendation) has no research artifact in docs/reports/
- [WARN] C-001: Inception T-2725 (started-work) has no research artifact in docs/reports/
- [WARN] C-001: Inception T-2753 (recommendation) has no research artifact in docs/reports/
- [WARN] C-001: Inception T-2828 (started-work) has no research artifact in docs/reports/
- [WARN] C-001: Inception T-2875 (started-work) has artifact but task doesn't reference it
- [WARN] C-001: Inception T-2879 (recommendation) has no research artifact in docs/reports/
- [WARN] C-001: Inception T-2918 (recommendation) has no research artifact in docs/reports/
- [WARN] C-006: Inception T-2862 has template-only Recommendation block
- [WARN] C-006: Inception T-2864 has template-only Recommendation block
- [WARN] C-006: Inception T-2867 has template-only Recommendation block
- [WARN] C-006: Inception T-2898 has template-only Recommendation block
- [WARN] C-006: Inception T-2907 has template-only Recommendation block
- [WARN] C-006: Inception T-2908 has template-only Recommendation block
- [WARN] C-006: Inception T-2909 has template-only Recommendation block
- [WARN] C-006: Inception T-2920 has template-only Recommendation block
- [WARN] C-006: Inception T-2925 has template-only Recommendation block
- [WARN] C-006: Inception T-2930 has template-only Recommendation block
- [WARN] C-006: Inception T-2931 has template-only Recommendation block
- [WARN] C-006: Inception T-2932 has template-only Recommendation block
- [WARN] C-006: Inception T-2934 has template-only Recommendation block
- [WARN] C-006: Inception T-2936 has template-only Recommendation block
- [WARN] C-006: Inception T-2937 has template-only Recommendation block
- [WARN] CTL-031: 1 stuck partial-complete task(s) — all ACs ticked, in active/ — run: bin/fw task archive-eligible
- [WARN] CTL-029: T-1415 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1420 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1426 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1428 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1430 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1432 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1451 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1453 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1632 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1633 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1799 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-1885 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-212 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2194 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2197 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2203 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2258 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2389 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2470 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2815 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2819 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2828 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2837 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2858 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-029: T-2870 has all Agent ACs ticked but status='started-work' — completable, not closed
- [WARN] CTL-012: Completed task T-1450 has unchecked AC
- [WARN] CTL-012: Completed task T-1497 has unchecked AC
- [WARN] CTL-012: Completed task T-2263 has unchecked AC
- [WARN] CTL-012: Completed task T-2264 has unchecked AC
- [WARN] CTL-012: Completed task T-2881 has unchecked AC
- [WARN] D5: Task lifecycle — 33 anomaly(s): T-2486(38d-active) T-2725(25d-active) T-1137(143d-active) T-1296(136d-active) T-1428(131d-active) T-1429(131d-active) T-1451(129d-active) T-1457(128d-active) T-1665(114d-active) T-1699(113d-active) (+23 more)
- [WARN] D13: Inception limbo — 2 task(s): A=1/B=1 T-1635(A:1hu) T-2828(B)
- [WARN] D14: Empty inception Recommendation — 15_empty: T-2862 T-2864 T-2867 T-2898 T-2907 (+10 more)
- [WARN] Arc 'arc-001': 42/45 tasks completed (0.9333) but arc still in-progress
- [WARN] Arc 'arc-002': 11/12 tasks completed (0.9167) but arc still in-progress
- [WARN] Arc 'arc-005': 2/2 tasks completed (1.0000) but arc still in-progress

## Doctor WARN (14 raw lines)

- WARN  Repo root: 1 untracked binary file(s) — not source, not screenshots
- WARN  Stale Watchtower triple (pid 1453550 not running)
- WARN  Installed claude-fw drifted from repo source — supervision export may be stale
- WARN  Task debt: 69 stale tasks (no updates in 7+ days)
- WARN  T-2209-history-skills (missing 8: check-settings-edit, check-active-completed-dup, check-arc-id, check-heredoc-cmd-sub, check-inception-decisions, check-inception-schema, check-onboarding-gate, check-rail-mcp-label)
- WARN  T-2398-findings (missing 8: check-settings-edit, check-active-completed-dup, check-arc-id, check-heredoc-cmd-sub, check-inception-decisions, check-inception-schema, check-onboarding-gate, check-rail-mcp-label)
- WARN  charter-review-2026-0814 (missing 8: check-settings-edit, check-active-completed-dup, check-arc-id, check-heredoc-cmd-sub, check-inception-decisions, check-inception-schema, check-onboarding-gate, check-rail-mcp-label)
- WARN  governance-canary-signal (missing 8: check-settings-edit, check-active-completed-dup, check-arc-id, check-heredoc-cmd-sub, check-inception-decisions, check-inception-schema, check-onboarding-gate, check-rail-mcp-label)
- WARN  merge-trial (missing 8: check-settings-edit, check-active-completed-dup, check-arc-id, check-heredoc-cmd-sub, check-inception-decisions, check-inception-schema, check-onboarding-gate, check-rail-mcp-label)
- WARN  Hook crashes: 1 today (9 total) — check .context/working/.hook-crashes.log
- WARN  Cron registry drift: /opt/termlink/.context/cron/agentic-audit.crontab differs from /etc/cron.d/agentic-audit-termlink
- WARN  Cron flock parity: registry declares 2 wrapped jobs, deployed crontab has 0
- WARN  Mirror divergence: 1 ref(s) differ between origin and github (T-1591/T-1592)
- WARN  Untracked task files: 2 under .tasks/{active,completed}/

## Findings discovered by DOING the work (not present in either verb's output)

- **F-A: `audit.sh` D8 can never PASS.** `handover.sh:712` emits an instructional comment
  containing the literal placeholder marker; D8 counts raw occurrences file-wide. Floor is 1,
  so `pass "D8: Handover quality — no [TODO] in LATEST.md"` (audit.sh:5098) is dead code and a
  perfectly filled handover reports WARN forever. Prose *about* the marker also counts.
  → Tasked as **T-2943 (closed)**. Filed upstream: P-073, `framework:pickup` offset 115.

- **F-B: own upstream filings round-trip back as local tasks.** Posting P-073 to
  `framework:pickup` caused the pickup processor to auto-create **T-2947** in this project —
  a task to fix the bug we had just reported upstream. T-2816 added `FW_PICKUP_SELF_PROJECT`
  self-filtering to the pickup *canary*, but the *processor* that mints tasks applies no
  equivalent filter. Net effect: every upstream filing inflates the local register by one
  duplicate, which then shows up as task debt in the very audit that prompted the filing.
  → NOT yet tasked. Next cycle.

- **F-C: the pre-push audit is not the audit.** The pre-push hook runs `--sections structure`
  only (38P/8W/2F) while the full run reports 368P/77W/5F. 69 warnings and 3 failures are
  invisible to the gate that runs on every push. Not a defect in either check — but the
  subset's summary line is indistinguishable in shape from the full one, so it reads as a
  whole-project verdict. → NOT yet tasked. Next cycle.

---

## Cycle 3 — findings and outcomes (2026-09-09)

Three findings from cycle 1's "discovered by doing the work" section were tasked, scored
and driven. A fourth (F-D) was found by executing the resume protocol itself.

### F-C → T-2948 — CLOSED. Filed upstream as P-074, `framework:pickup` offset 117.
The finding narrowed under measurement. `audit.sh` DOES declare scope — in its header,
~40 lines above the `=== SUMMARY ===` block. The defect is that the SUMMARY, the only part
operators read and the only part quoted into push output and handovers, is byte-shape
identical for a one-section run and a full one. The pre-push subset (`--section structure`,
T-862, for speed) is a sound tradeoff and is NOT the bug. Measured: 38P/8W/2F at push vs
368P/77W/5F full — 69 warnings and 3 failures read as *absent* rather than *unexamined*.
Fix proposed upstream is one line: emit the scope inside the SUMMARY block.

### F-B → T-2949 — CLOSED. Filed upstream as P-075.
Worse than cycle 1 recorded, in two separable ways. Posting P-073 produced **two**
byte-identical local tasks (T-2946, T-2947), not one — so minting is also not idempotent per
(topic, offset). And attribution was never missing: every minted title carries the literal
suffix `(from termlink)`, so the processor holds the source project and mints regardless.
The filter is absent over data that is present. Filed with T-2816 cited as the precedent to
mirror, including its stance on UNKNOWN attribution (mint when provenance is unprovable — a
false task is cheap, a missed inbound filing is the G-063 class the rail exists to prevent).

Echo latency is UNMEASURED here, stated as a limit rather than reported as absence: active
task count was captured immediately before the P-074 post (245) and after (244, fully
explained by T-2948 moving to `completed/`). No echo landed inside that window, so minting
is asynchronous on a period this session did not observe.

### F-D → T-2950 — the `/resume` skill's G-087-safe budget read does not execute.
`checkpoint.sh` exposes exactly three case arms — `post-tool`, `reset`, `status`. There is no
`budget` arm. The `/resume` skill mandates `checkpoint.sh budget` and explicitly forbids a
raw `cat .context/working/.budget-status`, citing G-087/T-222 where a stale or foreign-session
cache read back as a plausible healthy `{level:ok,tokens:0}` — measured in production at
0 vs 297,923 and 0 vs 70,549 tokens. Invoking the documented verb prints usage, exits 1, and
trips the hook-crash banner. The only documented safe read therefore does not exist, and an
operator following the skill falls back to precisely the read G-087 forbids.

**Cause is VERSION SKEW, not a missing upstream feature.** The string `checkpoint.sh budget`
occurs nowhere in the vendored framework; the skill quotes framework-repo IDs (T-222, G-087)
from a numbering this project never reaches. Vendored `.agentic-framework` is **1.6.29**,
declared baseline **2026-06-08** — three months stale, against a CLAUDE.md that references
upstream at v1.6.295. **No upstream filing was made**, deliberately: upstream almost certainly
ships the arm already, and filing a likely-already-fixed report would add exactly the phantom
register debt T-2949/P-075 was filed to stop, one cycle after filing it.

### Sovereign question (new, cycle 3): re-vendor the framework?
The remediation for F-D — and plausibly for an unknown number of other skill↔code mismatches
— is a re-vendor. Per CLAUDE.md that rewrites everything below `## Core Principle` (844 lines
here, including the entire Quick Reference table and its 29 operator entry points) and deletes
local divergences not yet landed upstream. `.vendor-divergence.yaml` currently registers three
at-risk local fixes. This is a human decision; it is surfaced, not taken.

### Observations recorded, deliberately NOT tasked (scope discipline)
- **G-020 blocks the edit that satisfies it.** Authoring real ACs via a shell heredoc is
  refused while the task still has placeholder ACs. The gate's own message names editing the
  task file as the remedy, so it is navigable via the file editor. Working as designed.
- **P-002 gates read-only compound reads.** A `for` loop of `grep`s is refused because it is
  not on the read-only allowlist. The block message itself invites filing this as an allowlist
  gap. Fail-closed is the correct default; not pursued here.
- **The BVP estimator cannot discriminate.** All three cycle-3 tasks scored identically
  (D1=4 D2=0 D3=3 D4=2; blast_radius=None tier=2 effort=8) despite materially different value.
  Prioritisation was done by judgment and said so. This is the standing PL-371 question.

### F-E (cycle 3, found by executing the completion path) — the framework's own suggested next command is unreachable when suggested
On completing T-2950, `fw task update --status work-completed` printed:

    LEARNING PROMPT — This looks like a bugfix task
    Consider: fw fix-learned T-2950 "what was learned"

Running it immediately was refused by P-002 with `BLOCKED: No active task`, because the same
completion clears focus. The framework prompts a command that its own completion step has
just made unrunnable. Navigable — take focus on any open task first — but the prompt as
printed cannot be followed literally at the moment it appears. Same class as T-2723 (handover
commit collides with the focus gate), which is already open. Recorded, not separately tasked:
the mechanism is T-2723's, and the mandate's rule is one finding one task, not one symptom
one task.
