# T-909 Symlink Fix — State Risk Evaluation

**Task:** T-909 (inception) — replace `/opt/termlink/.agentic-framework` symlink with vendored copy
**Angle:** State / data preservation
**Evaluator:** dispatched risk-eval worker (state angle)
**Date:** 2026-04-11

## Summary

**Verdict: GO-WITH-CAVEATS.** The proposed naive `rm .agentic-framework && cp -r /opt/999-...` is dangerous and must NOT be used — it would copy 349 MB (including `.git/` = 123 MB and `.context/` = 102 MB of framework state). However, a safe path already exists: `fw vendor` is implemented at `/opt/termlink/.agentic-framework/bin/fw` (lines 114-272) and explicitly excludes `.git/`, `.context/`, `.tasks/{active,completed}`, `.fabric/`. Use that instead. Critical pre-fix action: stop PID 1471772 (port 3002 Watchtower) before the symlink swap because its stdout/stderr are open file descriptors pointing at `/opt/999-.../.context/working/watchtower.log`.

## Directory Resolution Map

| Path as invoked | Symlink-resolved path | Real owner | Who writes |
|---|---|---|---|
| `/opt/termlink/.agentic-framework` | `/opt/999-Agentic-Engineering-Framework` | framework repo | framework dogfooding + termlink hook reads |
| `/opt/termlink/.tasks/` (inode 25036426) | itself (real dir) | TermLink | TermLink only (separate from framework) |
| `/opt/termlink/.agentic-framework/.tasks/` (inode 25560979) | `/opt/999-.../.tasks/` | framework repo | framework only |
| `/opt/termlink/.context/` (inode 25036169) | itself (real dir) | TermLink | TermLink `fw` tooling (PROJECT_ROOT resolved) |
| `/opt/termlink/.agentic-framework/.context/` (inode 25561342) | `/opt/999-.../.context/` | framework repo | **watchtower.sh bug** writes here |
| `/opt/999-.../.context/working/watchtower.pid` | itself | framework repo | both the framework's and TermLink's `watchtower.sh` (COLLISION — confirmed by G-001 live incident 13:30Z) |
| `/opt/025-WokrshopDesigner/.agentic-framework` | real directory (56 MB) | WorkshopDesigner | proper vendored copy; no `.git/`, has its own `.context/` |

**Key point:** TermLink's `fw` CLI finds `PROJECT_ROOT` by walking up from `$PWD` looking for `.framework.yaml` or `.tasks/` (`bin/fw` lines 54-62). When invoked from `/opt/termlink`, PROJECT_ROOT = `/opt/termlink`, which means tools correctly use `/opt/termlink/.tasks/` and `/opt/termlink/.context/`. The project-level state is already separate. The contamination happens only in tooling that uses `FRAMEWORK_ROOT/.context/` instead of `PROJECT_ROOT/.context/`.

## Running Processes Affected

**PID 1471772** — `python3 -m web.app --port 3002`
- cwd: `/opt/999-Agentic-Engineering-Framework` (framework repo, NOT termlink)
- stdout (fd 1): `/opt/999-Agentic-Engineering-Framework/.context/working/watchtower.log`
- stderr (fd 2): same
- This is the framework repo's own Watchtower, dogfooding itself. It is NOT termlink's Watchtower.
- TermLink has no Watchtower process of its own (no `/opt/termlink/.context/working/watchtower.pid`).
- **Impact of fix:** process is NOT running under the symlink-resolved path of termlink. Swapping the symlink at `/opt/termlink/.agentic-framework` does not directly affect this process. However, see "Pre-Fix Actions" — it should still be stopped cleanly to avoid a TermLink-triggered `watchtower.sh stop` accidentally killing it again during the fix window.

**PID 4170601** — `python3 -m web.app --port 3001`
- cwd: `/opt/025-WokrshopDesigner/.agentic-framework`
- This is WorkshopDesigner's vendored-copy Watchtower. Completely isolated. Not affected.

**No processes found with open fds resolving through `/opt/termlink/.agentic-framework/*`.** The symlink is traversed at invocation time by scripts, not held open by long-lived processes.

## Framework Repo Contamination

`git -C /opt/999-Agentic-Engineering-Framework status -s` shows 21 dirty paths. None are termlink-specific contamination:

- `M .context/audits/...`, `M .context/project/metrics-history.yaml`, `M .context/working/.session-metrics.yaml`, `M .context/working/focus.yaml`, `M .context/working/session.yaml`, `M .context/working/watchtower.log`, `M .context/working/watchtower.pid` — all framework-repo self-audit and watchtower heartbeat writes (framework dogfooding itself).
- `M/?? .tasks/active|completed/T-1087..T-1092` — framework's own high-number task IDs, not termlink.
- `?? .context/working/.handover-in-progress` — framework session state.

**Termlink references in framework state** (searched `/opt/999-.../.context/project/*.yaml`):
- `concerns.yaml` line 666: `"/opt/termlink — 17 days stale, 11/13 hooks"` — this is a fleet-audit summary from a framework-side task (T-614). It is framework-owned commentary about termlink, not termlink state leaked in. Safe.
- `patterns.yaml`: several entries about `termlink dispatch` — these are framework patterns describing TermLink-the-product behavior. Framework-owned.

**Task ID collision** (important but not a data-corruption risk):
- Framework completed `T-909-generate-missing-episodics-...md` on 2026-04-05 and has `.context/episodic/T-909.yaml`.
- TermLink active `T-909-fix-agentic-framework-symlink-...md` created 2026-04-11.
- These DO NOT collide in storage because `.tasks/` and `.context/episodic/` live in different directories (separate inodes). `fw` allocates new IDs by scanning the project's own `.tasks/`, not via a shared counter. Different projects can freely reuse task numbers.
- Episodic files for T-9xx series exist in both projects with different content. No merge/overwrite risk from the fix.

**No evidence of termlink state having leaked INTO the framework repo.** TermLink's project-scoped state (`.tasks/`, `.context/`, `.fabric/`, `focus.yaml`, `session.yaml`) all lives at the TermLink project root, not through the symlink.

**The actual cross-project contamination** is one-way and limited to `watchtower.sh`: when termlink invokes `watchtower.sh`, the script reads/writes the framework repo's PID file. G-001 documents this as a live incident (2026-04-11 13:30Z). No termlink data was lost — only the framework's Watchtower got inadvertently stopped.

## Data Loss Risks

What gets LOST when the symlink is replaced:

1. **Nothing in `/opt/termlink/`** — the symlink target, not the symlink, holds the data. Removing the symlink (`rm /opt/termlink/.agentic-framework`) only deletes the pointer. `/opt/999-...` is unaffected.
2. **Active process fds (PID 1471772)** — stdout/stderr are held open on `/opt/999-.../.context/working/watchtower.log`. The fd survives even if the symlink is replaced because it points directly at the target inode. No data loss from that angle. However, the process will continue writing to the old file; a restarted watchtower on the framework side would look in the same path and append. Still safe.
3. **In-flight `fw` operations** — if any background `fw` process is mid-write (e.g., updating `focus.yaml`), it is writing through its own resolved PROJECT_ROOT = `/opt/termlink/.context/working/focus.yaml`, which is NOT under the symlink. Safe.
4. **Locks in `.context/locks/`** — termlink uses `/opt/termlink/.context/locks/` (real dir), framework uses `/opt/999-.../.context/locks/`. Separate.

**Data-loss conclusion:** the fix is non-destructive of data IF done on termlink's side only (removing the symlink and replacing with a vendored copy). The framework repo is not touched.

## Copy Bloat Risks

The naive `cp -r /opt/999-Agentic-Engineering-Framework .agentic-framework` would bring in **349 MB total** including:

| Path | Size | Should be included? |
|---|---|---|
| `.git/` | **123 MB** | NO — duplicates framework git history, inflates termlink backups |
| `.context/` | **102 MB** | NO — contains framework episodic (1.3k+ files), bypass-log, sessions, contaminates termlink state |
| `.context/working/fw-vec-index.db` | **76 MB** | NO — framework's search index |
| `lib/` | 38 MB | YES — runtime libraries |
| `docs/` | 6.9 MB | YES (filterable) |
| `web/` | 2.9 MB | YES |
| `tests/` | 1.8 MB | NO typically (already `lib/ts/...` excluded by vendor) |
| `agents/` | 1.1 MB | YES |
| `.tasks/active` | hundreds of MD files | NO — framework task state |
| `.tasks/completed` | 633 entries | NO — framework task history |
| `.fabric/` | framework components | NO |
| `*.png` files in root | dozens of MB screenshots | NO |

**The fix script proposed in the risk briefing (`rm + cp -r`) is wrong and must not be used.**

## Existing Safe Path: `fw vendor`

`/opt/termlink/.agentic-framework/bin/fw` already implements `do_vendor()` (lines 114-272). It:

- Includes only: `bin`, `lib`, `agents`, `web`, `docs`, `.tasks/templates`, `FRAMEWORK.md`, `metrics.sh`
- Excludes: `__pycache__`, `*.pyc`, `.DS_Store`, `lib/ts/{src,tsconfig.json,package.json,package-lock.json,node_modules}`
- Explicit dry-run note (line 219): `"Would exclude: .git, .context, .tasks/{active,completed}, .fabric, install.sh"`
- Uses `rsync -a --delete` with the exclude list, falls back to `cp -r` + post-clean if rsync is absent
- Detects self-referencing copies (if target resolves to source, aborts with error) — important for atomicity
- Writes VERSION file and chmods bin/fw executable after copy

The vendored framework at `/opt/025-WokrshopDesigner/.agentic-framework` is 56 MB (vs 349 MB naive copy), confirming what `fw vendor` actually produces.

## Pre-Fix Actions Required

Exact commands, from TermLink's root (`/opt/termlink`), copy-pasteable:

```bash
# 1. Verify no termlink process is holding the symlink open (should return nothing)
cd /opt/termlink && lsof +D .agentic-framework 2>/dev/null | grep -v "COMMAND" || echo "OK: no open handles"

# 2. Confirm no termlink watchtower is running (should be nothing)
cd /opt/termlink && ls .context/working/watchtower.pid 2>&1 || echo "OK: no termlink watchtower pidfile"

# 3. Note the framework's watchtower PID (do NOT stop it from termlink's dir — G-001 collision).
#    If you need to stop it, do so from the framework repo:
#    cd /opt/999-Agentic-Engineering-Framework && bin/watchtower.sh stop
#    Or signal by PID directly: kill 1471772

# 4. Dry-run the vendor to see exactly what will be copied
cd /opt/termlink && .agentic-framework/bin/fw vendor --dry-run

# 5. Commit any uncommitted state in the framework repo first (safety — outside this project boundary)
#    git -C /opt/999-Agentic-Engineering-Framework status -s   # (run from another shell)

# 6. Take a safety snapshot of the current symlink target (just the inode, not the tree)
cd /opt/termlink && readlink -f .agentic-framework > /tmp/T-909-symlink-was.txt && cat /tmp/T-909-symlink-was.txt
```

**Do NOT:**
- Run `rm .agentic-framework` while the framework repo's `watchtower.sh stop` is executing — it uses `FRAMEWORK_ROOT/.context/working/watchtower.pid` via the symlink and could race with the rename.
- Use `rm -rf .agentic-framework/` with a trailing slash — on some systems it dereferences the symlink and tries to delete the target. Always `rm .agentic-framework` with no trailing slash.

## Recommended Fix Sequence

```bash
# From /opt/termlink (PROJECT_ROOT). These commands use the CURRENT fw (via symlink) to vendor itself.
cd /opt/termlink

# 1. Dry run
.agentic-framework/bin/fw vendor --dry-run

# 2. Remove the symlink (the target is untouched)
rm .agentic-framework

# 3. Re-create as empty dir and vendor into it
#    Note: fw is now gone from PATH via symlink; invoke it explicitly from the framework source
/opt/999-Agentic-Engineering-Framework/bin/fw vendor --target /opt/termlink

# Alternative (equivalent): vendor from within the framework dir
#   cd /opt/999-Agentic-Engineering-Framework && bin/fw vendor --target /opt/termlink
#   (this crosses the project-boundary gate; may need PROJECT_ROOT=/opt/termlink override)

# 4. Verify
ls -la /opt/termlink/.agentic-framework/bin/fw
/opt/termlink/.agentic-framework/bin/fw version
/opt/termlink/.agentic-framework/bin/fw doctor
```

## Post-Fix Verification

1. `/opt/termlink/.agentic-framework` is a directory, not a symlink: `test -d /opt/termlink/.agentic-framework && ! test -L /opt/termlink/.agentic-framework`
2. No `.git/` copied: `test ! -e /opt/termlink/.agentic-framework/.git`
3. No framework `.context/`: `test ! -e /opt/termlink/.agentic-framework/.context`
4. No framework `.tasks/active`: `test ! -e /opt/termlink/.agentic-framework/.tasks/active`
5. Size is <60 MB: `du -sh /opt/termlink/.agentic-framework | awk '{print $1}'`
6. `bin/fw` executes and reports version: `/opt/termlink/.agentic-framework/bin/fw version`
7. `fw doctor` passes: `/opt/termlink/.agentic-framework/bin/fw doctor`
8. Watchtower.sh now uses termlink's own PID file: `grep PID_FILE /opt/termlink/.agentic-framework/bin/watchtower.sh` should still say `$FRAMEWORK_ROOT/.context/working/watchtower.pid`, BUT now `FRAMEWORK_ROOT=/opt/termlink/.agentic-framework` (real dir), so the PID file lives at `/opt/termlink/.agentic-framework/.context/working/watchtower.pid`. **Note:** that's still inside the framework subdir, not in `/opt/termlink/.context/`. This is a separate latent bug — the fix is structurally the same as what WorkshopDesigner uses and the live collision (G-001) stops, but termlink still doesn't share watchtower state with its own project-level `.context/`. Document this as follow-up in T-909 findings.

## Additional Findings

1. **watchtower.sh script-level bug** (not part of T-909 but surfaced):
   - `/opt/termlink/.agentic-framework/bin/watchtower.sh` lines 21-22 use `$FRAMEWORK_ROOT/.context/` for PID and log. All other stateful scripts (handover.sh, update-task.sh, audit.sh) correctly use `$PROJECT_ROOT` for state. Only watchtower.sh mixes the two. This is the root cause of the cross-project collision in G-001.
   - Recommend separate task to change watchtower.sh to use `$PROJECT_ROOT/.context/working/watchtower.pid`. This fix is orthogonal to T-909 but eliminates the class of bug.

2. **Task ID allocator is per-project** — verified by inode comparison. No shared counter, no risk of overwriting framework episodics even when numbers collide.

3. **Bus and locks are project-scoped** — `/opt/termlink/.context/bus/` and `/opt/termlink/.context/locks/` are real dirs separate from the framework's. No bus message contamination.

4. **`.agentic-framework/.context/` already does not exist as a real dir** in the framework repo's self-hosted vendored framework (`/opt/999-.../.agentic-framework/`). So once termlink vendors, `/opt/termlink/.agentic-framework/.context/` will return ENOENT — which is correct behavior; any script that tries to write there is a bug to fix.

## Recommendation

**GO-WITH-CAVEATS** from the state-preservation angle.

- The fix is safe for TermLink's data: termlink's project-level state at `/opt/termlink/.context/` and `/opt/termlink/.tasks/` is already separate from the symlink target and will not be touched.
- The fix is safe for the framework repo: only termlink's symlink is removed; `/opt/999-...` is not modified.
- **Caveat 1 (hard blocker if ignored):** use `fw vendor`, NOT the naive `cp -r` from the risk briefing. The naive copy would include 349 MB of framework git, state, and build artifacts, polluting termlink's repo.
- **Caveat 2:** do not run `watchtower.sh stop` from `/opt/termlink` during the fix window — it targets the framework's watchtower via the symlink. Use the framework repo directly or `kill 1471772`.
- **Caveat 3 (follow-up task, not a blocker):** watchtower.sh's use of `$FRAMEWORK_ROOT` for state (not `$PROJECT_ROOT`) is an underlying bug that will persist after vendoring. The collision is gone, but state still lives inside `.agentic-framework/` rather than `.context/`. File this as a separate T-### after the T-909 GO decision.
- **Caveat 4:** the T-909 task ID collides with a completed framework task `T-909-generate-missing-episodics`. Not a data risk (different storage), but confusing if anyone searches by ID. Consider noting the collision in the T-909 decision record.

No data loss, no contamination, no running-process fd hazard. Proceed with `fw vendor`.

DONE-T909-STATE
