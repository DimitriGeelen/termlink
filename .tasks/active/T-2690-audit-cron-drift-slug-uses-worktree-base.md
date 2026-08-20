---
id: T-2690
name: "Audit cron-drift slug uses worktree basename — blocks push, mitigation installs stray cron"
description: >
  audit.sh derives the expected /etc/cron.d unit name from basename(PROJECT_ROOT). In a
  git worktree that basename is the ephemeral worktree name, so the check FAILs on a cron
  that must never exist, and the pre-push hook blocks every worktree session. The printed
  mitigation ("Run: fw cron install") installs a duplicate cron set keyed to the ephemeral
  worktree path — which has already happened on this host.
status: started-work
workflow_type: build
horizon: now
owner: claude-code
created: 2026-08-20
last_update: 2026-08-20
tags: [governance, framework-defect, cross-repo, cron, worktree]
---

# T-2690: Audit cron-drift slug uses worktree basename

## Context

The framework documents git worktrees as the isolation mechanism for parallel agent
sessions (and this project's background-job guidance *mandates* `EnterWorktree` before
any edit). But the audit's cron-drift check is not worktree-aware, and the pre-push hook
treats its FAIL as blocking.

`.agentic-framework/agents/audit/audit.sh:1364`:

```bash
_cron_slug=$(basename "$PROJECT_ROOT" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9_-]/-/g')
_cron_target="$_cron_target_dir/agentic-audit-${_cron_slug}"
```

`.agentic-framework/bin/fw:1772` (`fw doctor`) carries the **identical** derivation. There
is **zero** occurrence of the string `worktree` anywhere in `audit.sh`.

In `/opt/termlink` the slug is `termlink` and `/etc/cron.d/agentic-audit-termlink` exists,
so the check passes. In the worktree `/opt/termlink/.claude/worktrees/t2687-pickup-failopen`
the slug becomes `t2687-pickup-failopen` and the check FAILs on
`/etc/cron.d/agentic-audit-t2687-pickup-failopen` — a per-worktree audit cron that should
never exist. Worktrees are ephemeral; installing crons keyed to them is always wrong.

### Severity: the mitigation is actively harmful, not merely unactionable

The FAIL prints `Run: fw cron install`. Following that advice installs a cron unit named
for the ephemeral worktree, whose every job `cd`s into a directory that will be deleted
when the worktree is cleaned up.

**This is not hypothetical — it already happened on this host.**
`/etc/cron.d/agentic-audit-agent-a1edeca4dc54e9ac7` contains 40+ jobs, several on a
`* * * * *` schedule, all pointing into a `.claude/worktrees/agent-a1edeca4dc54e9ac7`
directory of a *different* project (002-Claude-Partner-Network). Its framework-audit job
is failing every 30 minutes with `No such file or directory` on the `fw` binary path baked
into the generated crontab (four occurrences in the two hours before this task was filed:
00:30, 01:00, 01:30, 02:00 on 2026-08-20, tag `agentic-cron`).

That is the framework's own advice producing a permanently-failing scheduled job — a
Directive #2 (Reliability / no silent failures) violation caused by a Directive #2 check.

### Why the framework was blind (G-019)

The cron-drift check was built (T-1771) to catch "generated but not deployed". Its notion
of project identity is a *path basename*, which silently stops being the project identity
the moment the framework's own recommended isolation mechanism is used. Nothing tests the
check from a worktree, so the false positive has been latent since T-1771.

The correct identity source is the **main worktree** path
(`git rev-parse --path-format=absolute --git-common-dir` → strip `/.git`, or
`git worktree list --porcelain | head -1`), not `basename $PWD`.

## Scope note (cross-repo, G-062)

The defect lives in the AEF framework (`999-agentic-engineering-framework`), vendored here
under `.agentic-framework/`. Per G-062 the fix is filed to AEF via `framework:pickup`
rather than forked locally. **This task's deliverable is the filing plus the evidence**,
not a local patch — patching the vendored copy would fork it and the fix would be lost on
the next framework sync.

## Acceptance Criteria

### Agent
- [x] Root cause identified to file:line in both `audit.sh` and `bin/fw`
- [x] Confirmed zero worktree-awareness in `audit.sh`
- [x] Confirmed the harmful-mitigation claim with a live on-host example + syslog evidence
- [x] Filed to `framework:pickup` with reproduction, root cause, and suggested fix
- [x] Recorded the blocked-push consequence in the session handover

### Human
- [ ] Decide whether to remove the stray `/etc/cron.d/agentic-audit-agent-a1edeca4dc54e9ac7`
      (40+ jobs pointing at another project's ephemeral worktree; the audit job has been
      failing every 30 min). It belongs to 002-Claude-Partner-Network, so removing it is a
      cross-project action this session must not take unilaterally.
      **Steps:** 1. Check whether the worktree still exists (its path is in the cron file's
      `# Project:` header). 2. If absent, the cron is pure garbage:
      `sudo rm /etc/cron.d/agentic-audit-agent-a1edeca4dc54e9ac7`
      3. `journalctl -t agentic-cron --since "1 hour ago" | grep -c "No such file"` → expect 0
      after the next half-hour.
      **Expected:** no `agentic-cron` "No such file" lines in syslog.
      **If not:** another stray worktree-keyed cron exists; list `/etc/cron.d/agentic-audit-*`
      and check each `# Project:` header for a `.claude/worktrees/` path.

## Verification

# The defective derivation is present in both places (this is the bug, so it must match)
grep -q 'basename "$PROJECT_ROOT"' .agentic-framework/agents/audit/audit.sh
grep -q 'basename "$PROJECT_ROOT"' .agentic-framework/bin/fw
# audit.sh has no worktree awareness at all
test "$(grep -c worktree .agentic-framework/agents/audit/audit.sh)" = "0"

## Decisions

**Filed upstream rather than patched locally.** The vendored `.agentic-framework/` tree is
a copy of another repo. A local patch would (a) fork the vendored copy, (b) be silently
reverted on the next framework sync, and (c) leave every other consumer project on this
host still blocked. G-062 routes cross-repo defects through `framework:pickup`.

**Did not bypass the gate to push.** `git push --no-verify` is a Tier 0 action requiring
human approval; the Autonomous Mode Boundaries rule explicitly withholds gate-bypass from
a broad "proceed as you see fit" directive. The branch stays unpushed, and the blocked push
is itself the clearest evidence of the defect's severity.
