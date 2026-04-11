# T-909 — Fix `.agentic-framework` Symlink (Inception)

**Task:** `.tasks/active/T-909-fix-agentic-framework-symlink--replace-w.md`
**Gap:** G-001 (medium, watching) in `.context/project/concerns.yaml`
**Status:** exploration — risk evaluation in progress
**Decision authority:** human

## Problem

`/opt/termlink/.agentic-framework` is a **symlink** to `/opt/999-Agentic-Engineering-Framework` (the framework source repo), while every other consumer project under `/opt/` uses a real vendored copy. The framework is blind to this misconfiguration because path resolution still succeeds — just to the wrong project root.

Surfaced on 2026-04-11 when starting Watchtower: `PROJECT_ROOT` defaulted via `$(cd "$SCRIPT_DIR/.." && pwd)` which resolved through the symlink, so Watchtower served framework tasks (T-1017, T-1087, …) instead of TermLink tasks. Mitigated in-session by passing `PROJECT_ROOT=/opt/termlink` explicitly.

Second live incident during this exploration session (13:30Z): running `watchtower.sh stop` from `/opt/termlink` killed the *framework's* watchtower instance on :3000 — both projects share a PID file at `.context/working/watchtower.pid` via the symlink. Confirms cross-project PID/log/state collision risk.

## Proposed Fix

```
cd /opt/termlink && rm .agentic-framework && cp -r /opt/999-Agentic-Engineering-Framework .agentic-framework
```

Gated on three-angle risk evaluation (this artifact) + explicit human approval.

## Risk Evaluation — Three Angles

| Angle | Sub-report | Status |
|-------|-----------|--------|
| Technical / path resolution / tooling | [T-909-symlink-fix-risk-tech.md](./T-909-symlink-fix-risk-tech.md) | in progress |
| State / data preservation | [T-909-symlink-fix-risk-state.md](./T-909-symlink-fix-risk-state.md) | in progress |
| Multi-project / blast radius | [T-909-symlink-fix-risk-blast.md](./T-909-symlink-fix-risk-blast.md) | in progress |

Each sub-report answers a self-contained prompt (from `/tmp/t909-risk-{tech,state,blast}.md`) and emits a verdict (GO / GO-WITH-CAVEATS / NO-GO) plus cited evidence.

## Dispatch History

- **Attempt 1 — `termlink dispatch` (background backend, 3 workers):** workers registered but the child `claude -p` never completed. Root cause: `claude -p --dangerously-skip-permissions` refuses to run under root (exit 1 in <100ms). Dispatch's shell template uses `wait $TL_PID` at the end, so when `user_cmd` fast-fails, sh hangs on the registrar that's waiting for orchestrator instructions. Workers appeared `ready` in `termlink list` but pstree showed no claude grandchild. Registered as **G-002** (high severity, dispatch observability bug). See `crates/termlink-cli/src/commands/dispatch.rs:293`.
- **Attempt 2 — direct `claude -p &`:** blocked by the same root restriction.
- **Attempt 3 — Claude Code `Agent` subagents:** succeeded. First 2 dispatches allowed by framework policy; 3rd needed `fw dispatch approve`. Each subagent writes its sub-report to disk and returns a short summary.

## Dialogue Log

**2026-04-11 (T-909 start):** User flagged the symlink with "THAT IS NOT CORRECT RIGHT??!!" and explicitly requested "three termlink agents first, no running until my explicit approval" with risk evaluation from different angles. Chose the clean path: new inception task T-909 + gap registration G-001 + 3-worker dispatch under T-909 scope. Compacted before firing the dispatch.

**2026-04-11 (post-compaction):** Fired dispatch. Hit 3 separate blockers (termlink dispatch observability bug, claude -p root restriction, framework Agent dispatch cap). Resolved each and got 3 subagents running.

## Findings

*Populated from the three sub-reports as they complete.*

### Technical / Path Resolution
*pending*

### State / Data Preservation
*pending*

### Multi-Project / Blast Radius
*pending*

## Synthesis

*Populated after all three sub-reports return.*

## Recommendation

*Populated after synthesis.*

## Decision

*Human-owned. `fw inception decide T-909 go|no-go --rationale "..."`*
