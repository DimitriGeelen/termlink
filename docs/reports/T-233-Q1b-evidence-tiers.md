# T-233 Q1b: Enforcement Tier System — Evidence Assessment

## Verdict: PARTIALLY WORKING — Hooks wired and active, but with known gaps

## Tier 0 (Destructive Command Gate) — WORKING

**Hook exists:** `/usr/local/opt/agentic-fw/libexec/agents/context/check-tier0.sh` — detects force-push, pkill, rm -rf, DROP TABLE, etc.

**Hook wired:** `.claude/settings.json` line 54-60 — PreToolUse matcher on `Bash` calls `fw hook check-tier0`.

**Bypass log proves real blocking:** `.context/bypass-log.yaml` contains 3 entries:
1. `2026-03-10` — pkill -9 (force-kill Terminal windows) → approved via `fw tier0 approve`
2. `2026-03-18` — `git push origin main --force` → approved via `fw tier0 approve`
3. `2026-03-18` — `git push github main --force` → approved via `fw tier0 approve`

**Conclusion:** Tier 0 is structurally enforced. The bypass log is the smoking gun — it proves commands were blocked, required human approval, and the approval was logged with command hash and timestamp.

## Tier 1 (Task Gate) — WORKING, with known sub-agent gap

**Hook exists:** `/usr/local/opt/agentic-fw/libexec/agents/context/check-active-task.sh` — blocks Write/Edit without active task.

**Hook wired:** `.claude/settings.json` line 44-52 — PreToolUse matcher on `Write|Edit` calls `fw hook check-active-task`.

**Evidence of real blocking:** Commit `f388636` (T-119) — "Inception — mesh workers blocked by task gate, 5 options enumerated." This was a real incident where sub-agents dispatched via TermLink mesh hit the task gate because they lacked task context. Resolution: `9d53447` — GO decision to add `--dangerously-skip-permissions` to mesh agent wrapper.

**Known gap (G-005):** Pure conversation sessions fire zero hooks. No PreToolUse means no task gate. Severity: medium. Status: watching. Mitigation: CLAUDE.md rule + `/capture` skill.

## Tier 2 (Situational Authorization) — PARTIALLY IMPLEMENTED

Evidence of `--force` bypass in task completion (lines 131-132 in settings.local.json permissions show `--force` flag usage on `update-task.sh`). No dedicated Tier 2 logging mechanism found beyond the bypass-log.yaml used by Tier 0.

## Tier 3 (Pre-approved Categories) — SPEC ONLY

CLAUDE.md states Tier 3 is "Spec only" in the enforcement table. No implementation found. Health checks, status queries, and git-status are implicitly allowed by not matching hook matchers, but there's no explicit pre-approval registry.

## Budget Gate — WORKING

**Hook exists:** `/usr/local/opt/agentic-fw/libexec/agents/context/budget-gate.sh`

**Hook wired:** `.claude/settings.json` line 62-70 — PreToolUse on `Write|Edit|Bash` calls `fw hook budget-gate`. Blocks at >=150K tokens.

## Additional Enforcement: Plan Mode Block — WORKING

PreToolUse hook on `EnterPlanMode` (line 36-42) blocks Claude's built-in plan mode, forcing use of framework's `/plan` skill instead.

## Summary Counts

| Tier | Status | Evidence |
|------|--------|----------|
| Tier 0 (Destructive) | **WORKING** | 3 logged blocks in bypass-log.yaml |
| Tier 1 (Task gate) | **WORKING** | T-119 real block incident; G-005 conversation gap |
| Tier 2 (Situational) | **PARTIAL** | --force flag usage exists; no dedicated log |
| Tier 3 (Pre-approved) | **SPEC ONLY** | No implementation |
| Budget gate | **WORKING** | Hook wired, checkpoint PostToolUse fallback |

**Total commits in project:** 538. Hook-related commits: ~13 (path fixes, integration, baseline updates). The enforcement system has survived 500+ commits of real work, with evidence of both blocking and authorized bypass.
