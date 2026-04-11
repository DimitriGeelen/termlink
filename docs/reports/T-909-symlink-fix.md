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

| Angle | Sub-report | Verdict |
|-------|-----------|---------|
| Technical / path resolution / tooling | [T-909-symlink-fix-risk-tech.md](./T-909-symlink-fix-risk-tech.md) | GO-WITH-CAVEATS |
| State / data preservation | [T-909-symlink-fix-risk-state.md](./T-909-symlink-fix-risk-state.md) | GO-WITH-CAVEATS |
| Multi-project / blast radius | [T-909-symlink-fix-risk-blast.md](./T-909-symlink-fix-risk-blast.md) | GO-WITH-CAVEATS |

Each sub-report answers a self-contained prompt (from `/tmp/t909-risk-{tech,state,blast}.md`) and emits a verdict (GO / GO-WITH-CAVEATS / NO-GO) plus cited evidence.

## Dispatch History

- **Attempt 1 — `termlink dispatch` (background backend, 3 workers):** workers registered but the child `claude -p` never completed. Root cause: `claude -p --dangerously-skip-permissions` refuses to run under root (exit 1 in <100ms). Dispatch's shell template uses `wait $TL_PID` at the end, so when `user_cmd` fast-fails, sh hangs on the registrar that's waiting for orchestrator instructions. Workers appeared `ready` in `termlink list` but pstree showed no claude grandchild. Registered as **G-002** (high severity, dispatch observability bug). See `crates/termlink-cli/src/commands/dispatch.rs:293`.
- **Attempt 2 — direct `claude -p &`:** blocked by the same root restriction.
- **Attempt 3 — Claude Code `Agent` subagents:** succeeded. First 2 dispatches allowed by framework policy; 3rd needed `fw dispatch approve`. Each subagent writes its sub-report to disk and returns a short summary.

## Dialogue Log

**2026-04-11 (T-909 start):** User flagged the symlink with "THAT IS NOT CORRECT RIGHT??!!" and explicitly requested "three termlink agents first, no running until my explicit approval" with risk evaluation from different angles. Chose the clean path: new inception task T-909 + gap registration G-001 + 3-worker dispatch under T-909 scope. Compacted before firing the dispatch.

**2026-04-11 (post-compaction):** Fired dispatch. Hit 3 separate blockers (termlink dispatch observability bug, claude -p root restriction, framework Agent dispatch cap). Resolved each and got 3 subagents running.

## Findings

### Technical / Path Resolution (GO-WITH-CAVEATS)
- **Latent trap defused by the fix:** `lib/update.sh:216` — a future `fw update` on termlink currently would `rsync --delete` through the symlink into `/opt/999-Agentic-Engineering-Framework/{bin,lib,agents,web}/`, corrupting the framework repo. Vendoring eliminates this entirely.
- **Watchtower state collision** at `bin/watchtower.sh:16-22` — uses `$FRAMEWORK_ROOT` instead of `$PROJECT_ROOT` for PID/log. Already leaking (see blast report live evidence).
- **Harvest regression** at `lib/harvest.sh:74-75, 363` — writes to `$FRAMEWORK_ROOT/.context/`. Currently accidentally-correct (writes to live framework); post-swap would write to the static vendored copy. Mitigation: avoid `fw harvest` from termlink until rerouted.
- **Swap window is safe**: zero processes with cwd under `.agentic-framework`, zero file descriptors inside it, no git tracking (`.gitignore:18`).

### State / Data Preservation (GO-WITH-CAVEATS)
- **TermLink project state is already separate** from the symlink target — verified via inode comparison. `/opt/termlink/.context/` and `/opt/termlink/.tasks/` are real per-project directories. The fix is non-destructive of data.
- **Naive `cp -r` copies 349 MB of framework pollution**: `.git/` (123 MB), `.context/` (102 MB, including a 76 MB vector index and all framework episodics), 633 completed framework tasks, dozens of root PNGs. **Use `fw vendor`** instead — already at `bin/fw:114-272`, excludes `.git`, `.context`, `.tasks/{active,completed}`, `.fabric`. Output ~56 MB (matching WokrshopDesigner's vendored copy).
- **Live PID-file collision** confirms active cross-project writes via the symlink. During the fix window, do NOT run `watchtower.sh stop` from `/opt/termlink` — it'd kill the framework's Watchtower via shared PID. Kill PID 1471772 directly.
- **T-909 ID collision**: framework has completed `T-909-generate-missing-episodics` with its own `.context/episodic/T-909.yaml`. Different task, separate directories, no data corruption. Noted.

### Multi-Project / Blast Radius (GO-WITH-CAVEATS)
- **Live Watchtower pollution (running now)**: PID 1471772 has kernel-resolved CWD to `/opt/999-...` and is actively writing `watchtower.log` and `watchtower.pid` into the framework repo's `.context/working/` — confirmed via `/proc/1471772/fd`. Must be killed and restarted after the fix.
- **TermLink is the ONLY symlinked consumer** out of 11 projects under `/opt/`.
- **Historical trail** T-215 → T-290 → T-659 → T-825 shows the symlink was a day-1 dev-convenience shortcut, never an intentional design.
- **`fw upgrade` becomes a no-op** after vendoring until a "vendor refresh" workflow is defined. Other consumers use `/root/.agentic-framework` (v1.4.520) or direct paths — unaffected.
- **Cron is unaffected** — termlink cron uses `/root/.agentic-framework/bin/fw` (separate install).

## Synthesis

All three angles converge on **GO-WITH-CAVEATS**. The fix is safe **if** executed with:

1. Pre-flight kill of live Watchtower (PID 1471772) via direct `kill`, not `watchtower.sh stop`.
2. **`fw vendor` instead of `cp -r`** — 56 MB vs 349 MB, proper exclusions.
3. Atomic rollback staging via `mv .agentic-framework .agentic-framework.symlink.bak` before vendoring.
4. Post-flight: restart Watchtower with explicit `PROJECT_ROOT`, verify PID file now resolves to real path, `fw doctor` green, delete `.symlink.bak`.
5. **Do NOT run `fw harvest` from termlink** until `lib/harvest.sh` uses `$PROJECT_ROOT`.
6. **Do NOT run `fw update`/`upgrade` from termlink** until follow-ups land (vendor refresh workflow).

The task description's literal fix command (`rm + cp -r`) is **unsafe as written** and must not be executed.

## Recommended Fix Procedure

```bash
# Pre-flight — verify nothing is rooted under .agentic-framework
cd /opt/termlink
lsof +D /opt/termlink/.agentic-framework 2>/dev/null | head

# Stop the contaminating Watchtower directly (kernel PID, not via watchtower.sh)
kill 1471772 2>/dev/null || true
sleep 1

# Atomic stage: rename symlink, don't delete yet (rollback via mv back)
mv /opt/termlink/.agentic-framework /opt/termlink/.agentic-framework.symlink.bak

# Vendor via the existing command (56 MB, proper exclusions)
/opt/999-Agentic-Engineering-Framework/bin/fw vendor

# Confirm the result is a real directory, not a symlink, and ~56 MB
ls -la /opt/termlink/.agentic-framework
du -sh /opt/termlink/.agentic-framework

# Sync version metadata into .framework.yaml
fw upgrade

# Restart Watchtower on :3002 with explicit PROJECT_ROOT
PROJECT_ROOT=/opt/termlink /opt/termlink/.agentic-framework/bin/watchtower.sh start --port 3002

# Verify Watchtower's PID file is now under /opt/termlink real path (not symlinked)
readlink -f /opt/termlink/.agentic-framework/.context/working/watchtower.pid

# Health check
fw doctor

# Rollback plan if anything above fails:
#   mv /opt/termlink/.agentic-framework.symlink.bak /opt/termlink/.agentic-framework
#   (restart watchtower as before)
```

## Follow-up Tasks (post-fix, out of T-909 scope)

- **T-910** (proposed): Fix `bin/watchtower.sh:16-22` to use `$PROJECT_ROOT` for PID/log instead of `$FRAMEWORK_ROOT`.
- **T-911** (proposed): Fix `lib/harvest.sh:74-75, 363` to write learnings to `$PROJECT_ROOT/.context/` instead of `$FRAMEWORK_ROOT/.context/`.
- **T-912** (proposed): Define a "vendor refresh" workflow so `fw upgrade` from a consumer re-syncs from the live framework repo.
- **T-913** (proposed): Add `fw doctor` check — warn if any consumer project's `.agentic-framework` is a symlink.
- **G-002** (registered): Fix `termlink dispatch` observability bug (wait-for-registrar hangs when user_cmd fast-fails). See `crates/termlink-cli/src/commands/dispatch.rs:293`.

## Decision

*Human-owned. After the recommendation is accepted: `fw inception decide T-909 go --rationale "3-angle risk eval converged on GO-WITH-CAVEATS; fix refined to use fw vendor with pre-flight kill of contaminating watchtower"`*

Agent recommendation: **GO** (with the refined procedure above — NOT the literal task description command).
