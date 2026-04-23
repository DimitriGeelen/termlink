---
id: T-915
name: "Poll framework for T-909 RCA fixes; fw upgrade when landed"
description: "Wait-and-poll task for 4 framework bugs surfaced during T-909 (symlink fix, 2026-04-11). When upstream fixes land, run fw upgrade from termlink and re-verify. If no fixes available, leave on horizon: later and recheck periodically. See body for findings + check procedure."
status: captured
workflow_type: build
owner: agent
horizon: later
tags: [framework, upgrade, rca, polling]
components: []
related_tasks: [T-909, T-910, T-911, T-912, T-913, T-914]
created: 2026-04-11T12:47:25Z
last_update: 2026-04-23T15:46:34Z
date_finished: null
---

## Findings

All framework-side; do not patch from termlink.

**F1 [HIGH]** — `fw inception decide --force` bypasses build readiness gate (G-020) and AC verification (P-010). T-909 was completed with all 3 Agent ACs unchecked and an empty Recommendation section. Episodic generated 1s after the bypass-completion. Framework should refuse to close any inception task with unchecked Agent ACs OR empty Recommendation, even with --force; --force should require explicit per-AC override flags.

**F2 [MEDIUM]** — Framework's task-review prompt prints the wrong runnable command path: it shows the in-repo bin/fw path, but consumer projects (like termlink) reach fw via `.agentic-framework/bin/fw`. T-609 'copy-pasteable commands' learning never propagated into the framework's own UI/output messages. Reproduced live during T-909.

**F3 [MEDIUM]** — Episodic for T-909 (`.context/episodic/T-909.yaml`) was generated immediately after `fw inception decide --force`, BEFORE the actual fix work commits. It captures only 2 evidence/research commits, missing the actual vendoring fix, the 5 follow-up tasks (T-910..T-914), the 3 risk subreports, and the enforcement baseline. Episodic generation should be deferred until task is genuinely closed.

**F4 [LOW]** — `fw vendor` is undocumented in CLAUDE.md (not in Quick Reference, not in Component Fabric, not anywhere). Manual workaround: add `fw vendor` line to local CLAUDE.md.

## Check Procedure

Grep framework git log since 2026-04-11 for keywords (`inception decide`, `--force`, `build readiness`, G-020, episodic, `fw vendor`). If matches found, run `fw upgrade` and re-verify. If no matches, update `last_update` and leave `horizon=later`.

# T-915: Poll framework for T-909 RCA fixes; fw upgrade when landed

## Context

Periodic check: have upstream fixes landed for F1-F4?

## Acceptance Criteria

### Agent
- [x] Polled framework git log since 2026-04-11 for F1-F4 keywords
- [x] Findings recorded below; no direct F1 (build readiness) hit — leave horizon=later

**Poll results (2026-04-18):**
- T-1259 (framework): Added CLAUDECODE guard to `fw inception decide` — adjacent but not F1
- T-1223 (framework): Fixed `inception decide` 500 by adding captured→started-work transition
- T-1258 (framework): Learnings.yaml RCA — identified Write-tool bypass as a truncation source
- T-1232 (framework): Mined 232 bugfix learnings — improved retrospective capture
- **No direct fix for F1** (`--force` still bypasses build readiness G-020 + AC P-010)
- **No F2** (task-review prompt showing wrong consumer path — still shows `bin/fw`)
- **F3 partial** (episodic ordering improvements mentioned in T-1236, but not structural deferral)
- **No F4** (`fw vendor` still undocumented in Quick Reference)

**Decision:** leave horizon=later. No `fw upgrade` warranted yet.

**Poll results (2026-04-21):**
- Framework active on T-1288/T-1368-T-1375 series — G-048/G-052/G-053/G-054 fixes, task-ID allocator serialization, keylock hardening, absolute hook paths. Nothing touches F1-F4.
- `agents/inception/` directory: only T-1279 (task-ID serialization) — unrelated to F1 `decide --force` bypass.
- T-1324 (b55309e7) "fw inception decide auto-ticks [REVIEW]/[RUBBER-STAMP] Human AC" — adjacent to F1 (improves completion hygiene) but does NOT block `--force` from bypassing build-readiness/AC gates.
- `CLAUDE.md` edits: T-1325 added `fw prompt` to Quick Reference. No `fw vendor` doc addition (F4 still open).
- No commits touching F2 (consumer path `.agentic-framework/bin/fw` in task-review output) or F3 (episodic deferral).
- **Decision:** leave horizon=later. No `fw upgrade` warranted. Next poll in ~3 days.

**Poll results (2026-04-23):**
- Framework activity since 2026-04-21: T-1268 inception → 5 build units shipped (`fw pending` registry, doctor surfacing, Watchtower /pending page, `fw pending remind`, B3 nav follow-on); T-1394 audit trend windowing; T-1395 TASKS_DIR/CONTEXT_DIR env-inheritance trap; T-1396 pre-push hook prefers `agents/` over vendored; T-1402 audit.sh null-timestamp crash + fabric edge enrichment.
- **F1 (decide --force bypass):** T-1194 added "extend tick_inception_decide_acs to tick ceremonial Agent ACs" — adjacent (auto-tick of ceremonial ACs reduces some legitimate `--force` use) but does NOT block `--force` from bypassing G-020/P-010. Still open.
- **F2 (consumer path in task-review prompt):** No commits touching this. Still open.
- **F3 (episodic ordering):** No commits. Still open.
- **F4 (`fw vendor` undocumented in CLAUDE.md):** **FIXED upstream.** `grep "fw vendor" /opt/999-Agentic-Engineering-Framework/CLAUDE.md` → 1 match in Quick Reference: `| Vendor framework | fw vendor | Copy framework for full isolation |`. Local `/opt/termlink/CLAUDE.md` does not yet have it (will pick up on next vendor sync).
- Adjacent observation: T-1396 (`Prefer source-of-truth agents/ over vendored .agentic-framework/ in pre-push hook`) is in the same neighborhood as F2 (consumer-path correctness) — worth re-checking F2 next poll.
- **Decision:** leave horizon=later. F4 is fixed but does not warrant a `fw upgrade` cycle on its own (purely cosmetic doc fix); will be picked up automatically when next sync runs. F1-F3 still open. Next poll in ~3 days.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-11T12:47:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-915-poll-framework-for-t-909-rca-fixes-fw-up.md
- **Context:** Initial task creation

### 2026-04-11T14:55:00Z — poll-result [agent]
- **Action:** Ran the check procedure against `/opt/999-Agentic-Engineering-Framework`
- **Command used:** `git -C /opt/999-Agentic-Engineering-Framework log --all --oneline --since=2026-04-11`
- **Scanned for keywords:** inception decide --force, G-020, G-031..G-034, build readiness, episodic defer, fw vendor, T-909
- **Framework activity since 2026-04-11:** 10+ commits in the T-1100-series. Notable:
  - `39f9b2a7 T-1101: Register G-032..G-034 + widen G-031 from /opt/termlink T-909 RCA` — gaps *registered* (so the framework has seen our findings) but not fixed.
  - `eb07962e T-1105: Structural-fix discipline pass — chokepoint+test upgrades for T-1100..T-1104`
  - `a692c60d T-1106: Structural upgrade — chokepoint+invariant tests for Watchtower port bleed`
  - T-1107, T-1108: unrelated defense-in-depth / Watchtower rendering
- **Inspected code paths:** `git log --since=2026-04-09 -- bin/fw agents/inception lib/inception` → only one unrelated commit (`4fb978f4 T-1081: Fix fw gaps — honor T-397 rename to concerns.yaml`)
- **Result:** No fixes landed for F1 (decide --force bypass), F2 (wrong runnable path), F3 (premature episodic), or F4 (fw vendor docs). Framework has acknowledged the gaps (T-1101) but no remediation commits yet.
- **Action:** Leaving horizon=later. Will recheck next session.

### 2026-04-18T15:54:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-18T15:55:05Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)

### 2026-04-21T11:01:37Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-21T11:02:35Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)

### 2026-04-22T04:52:51Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-23T15:46:34Z — status-update [task-update-agent]
- **Change:** horizon: next → later
