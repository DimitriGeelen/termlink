# T-1192: Dispatched Claude workers blocked as root

**Status:** inception, started-work. Template filled, awaiting human review before spikes.

**Working directory:** `/opt/termlink` (host .107, running as root)

## Trigger

2026-04-22T20:30Z: trying to execute T-1188 + T-1190 upstream-mirror via `termlink dispatch --count 1 --backend background -- claude -p --dangerously-skip-permissions "..."`. Worker registered in ~1s and exited immediately. Direct reproduction:

```
$ claude -p --dangerously-skip-permissions "say OK"
--dangerously-skip-permissions cannot be used with root/sudo privileges for security reasons
```

Exit code 0 (misleading — command "succeeded" at printing the error, dispatched worker has no way to tell the orchestrator).

The T-914 fix (G-002 repair: `USER_RC=$?; kill $TL_PID; exit $USER_RC`) correctly prevented the worker from hanging on `wait`. That fix is orthogonal to this one — it made fast-fail visible, didn't make it avoidable.

## Why this is a structural problem, not a workaround

- T-559 sanctions dispatch as THE cross-project path.
- Every fleet host runs as root (containers, systemd, LXC).
- Therefore T-559 is vacuous on every current host. Every honest attempt at "mirror the patch upstream" or "run this refactor in the other project" dead-ends at the same claude-refuses-root wall.
- The pressure is toward boundary violations: agents that can't dispatch learn to `cd /opt/other && git commit` and the hook blocks them, but the governance goal (isolated governance in the target project's process) is already lost by the time the hook fires.

## Four candidate channels (to evaluate in spikes)

| # | Channel | Claude needed? | Per-host cost | Notes |
|---|---------|----------------|---------------|-------|
| 1 | `termlink dispatch --shell` (plain bash in target workdir) | No | 0 | Mechanical work only. Governed by target project's git hooks. |
| 2 | `sudo -u fw-dispatch -H claude -p ...` | Yes | ~10 min user setup + auth copy | Needs a new system user on every host + claude auth migration per host |
| 3 | `podman run --user 1000 -v /opt:/opt:rw claude:latest -p ...` | Yes | Image pull + auth mount | Adds container dep; auth mount or fresh login each dispatch |
| 4 | Direct human-in-the-loop for dispatched tasks (fallback only) | N/A | None | What T-1188/T-1190 currently do — Human RUBBER-STAMP |

## Open question ordering (for spike planning)

1. Is Spike 5's decision matrix (what fraction of real dispatch use-cases need Claude's judgment vs pure mechanical work?) the fastest way to pick the dominant channel? If ≥80% are mechanical, Channel 1 is the answer and we don't need 2/3 at all.
2. Does non-root user require per-host onboarding forever, or can it be one-shot via the same install script that deploys termlink + fw?
3. Is there a middle path: mechanical by default, Claude-channel as opt-in via a new `--backend` enum value (`claude-root` fails loudly, `claude-user` uses sudo, `claude-container` uses podman)?

## Dialogue Log

### 2026-04-22T20:38Z — problem framed with user

- **User prompt:** "ok incept how we can solve this structurally"
- **Trigger:** termlink dispatch failed with root-block (this session, attempting T-1188/T-1190 mirror).
- **Agent action:** created T-1192 inception, filled Problem Statement + Assumptions + Exploration Plan + Scope Fence + GO/NO-GO criteria. Presenting for review before running any spike.
