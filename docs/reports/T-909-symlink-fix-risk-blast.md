# T-909 Symlink Fix — Multi-Project Blast Radius Evaluation

**Evaluator angle:** Multi-project / blast radius
**Date:** 2026-04-11
**Task:** T-909 (inception — replace `/opt/termlink/.agentic-framework` symlink with vendored copy)

## Summary

**Verdict: GO-WITH-CAVEATS.** The fix itself is low blast-radius — no cron entries, no systemd services, and no other consumer project touches `/opt/termlink/.agentic-framework`. TermLink is the ONLY symlinked consumer out of 11 projects under `/opt/`. The caveats are (1) a running Watchtower process currently holds file descriptors to `/opt/999-.../.context/working/watchtower.log` (confirmed active pollution) and must be restarted, (2) `fw upgrade` was last run against termlink while the symlink was live and wrote to termlink's `.framework.yaml` as v1.5.246 — but the framework repo's own VERSION file is also 1.5.246, so post-fix termlink will be identical to a fresh vendor clone, (3) the fix must target the symlink-resolved source (`/opt/999-.../`) excluding its own `.context/`/`.tasks/`/`.git`.

## Consumer Project Survey

Evidence:
```
for d in /opt/*/; do [ -e "$d.agentic-framework" ] || continue; ... done
```

| Project | Type | Framework Version | Notes |
|---|---|---|---|
| 001-sprechloop | REAL DIR | 1.5.246 | current |
| 025-WokrshopDesigner | REAL DIR | 1.1.13 | severely stale |
| 050-email-archive | REAL DIR | 1.5.246 | current |
| 051-Vinix24 | REAL DIR | 1.5.246 | current |
| 052-KCP | REAL DIR | 1.5.246 | current |
| 053-ntfy | REAL DIR | 1.5.246 | current |
| 150-skills-manager | REAL DIR | 1.5.246 | current |
| 3021-Bilderkarte-tool-llm | REAL DIR | 1.5.246 | current |
| 995_2021-kosten | REAL DIR | 1.5.246 | current |
| 999-Agentic-Engineering-Framework | REAL DIR (nested inside framework repo) | 1.5.81 | stale self-vendored copy; framework repo itself is at 1.5.246 (see `/opt/999-.../VERSION`) |
| openclaw-evaluation | REAL DIR | 1.5.246 | current |
| **termlink** | **SYMLINK → /opt/999-Agentic-Engineering-Framework** | (1.5.246 via symlink) | **outlier — only symlinked consumer** |

**Conclusion:** TermLink is the only symlinked consumer. Fix does not affect any other project.

## Framework Repo Contamination Evidence

Evidence:
```
git -C /opt/999-Agentic-Engineering-Framework log --oneline -25
git -C /opt/999-Agentic-Engineering-Framework status --short
```

**Good news:** The framework repo's git history is NOT contaminated with termlink task IDs. The framework repo has its own T-908/T-909 (episodic-summary tasks) which are ID collisions with TermLink's T-908 (API relay governance inception) and T-909 (this task), but they live in separate `.git` repos:
```
/opt/termlink/.git        → inode 25047001, toplevel /opt/termlink
/opt/999-.../.git         → inode 25741... (different repo)
```

`git -C /opt/termlink ...` correctly resolves to `/opt/termlink/.git` — the symlink does NOT cause shared git history because `.git`, `.context/`, and `.tasks/` all live OUTSIDE `.agentic-framework/`.

**Bad news (active pollution):** The Watchtower process currently running confirms live cross-project state writes:
```
$ ps -fp 1471772
python3 -m web.app --port 3002
$ tr '\0' '\n' < /proc/1471772/environ | grep -E 'PROJECT|FRAMEWORK'
PROJECT_ROOT=/opt/termlink
PWD=/opt/termlink/.agentic-framework
FRAMEWORK_ROOT=/opt/termlink/.agentic-framework
$ ls -l /proc/1471772/cwd
/proc/1471772/cwd -> /opt/999-Agentic-Engineering-Framework   ← symlink resolved by kernel
$ ls -l /proc/1471772/fd | grep watchtower
1 -> /opt/999-Agentic-Engineering-Framework/.context/working/watchtower.log
2 -> /opt/999-Agentic-Engineering-Framework/.context/working/watchtower.log
```

The process was launched from `/opt/termlink/.agentic-framework/...` as CWD; the kernel resolved that to `/opt/999-.../`, and `web.app` then wrote `watchtower.pid` and opened `watchtower.log` at CWD-relative paths. Result: TermLink's Watchtower runtime data is being written into the framework source repo right now. This is the G-001 "second live incident" the concerns.yaml already documents.

## CI / Backup / Cron Dependencies

Evidence:
```
crontab -l; ls /etc/cron.d/; grep -rn 'PROJECT_ROOT="/opt/' /etc/cron.d/
systemctl list-units --type=service | grep -iE 'termlink|agent|watchtower|framework'
ls /etc/systemd/system/ | grep -iE 'termlink|agent|watchtower|framework'
```

**Cron entries affecting termlink:**

1. `/etc/cron.d/agentic-audit-termlink` — installed via `fw audit schedule install`. Uses:
   ```
   PROJECT_ROOT="/opt/termlink" "/root/.agentic-framework/bin/fw" audit ...
   ```
   **Note the fw binary path: `/root/.agentic-framework/bin/fw` (v1.4.520)**, NOT `/opt/termlink/.agentic-framework/bin/fw`. This is a separate global install in `$HOME`. It means termlink's cron audits currently run through a v1.4.520 binary that READS termlink's files — completely independent of the `/opt/termlink/.agentic-framework` symlink. **The fix has ZERO impact on these cron jobs.**

2. Root crontab entry:
   ```
   0 6 * * * PATH=... /opt/999-Agentic-Engineering-Framework/agents/termlink/termlink.sh update --quiet >> /var/log/termlink-update.log 2>&1
   ```
   This hits `/opt/999-...` directly (NOT via termlink's symlink). Unaffected.

3. `/etc/cron.d/agentic-audit-999-agentic-engineering-framework` — runs on `/opt/999-...` independently. Unaffected.

**Systemd services:**
- `watchtower-vinix24.service` (port 3056) — scoped to `/opt/051-Vinix24`. Unaffected.
- `pulse-agent.service` — unrelated.
- **No systemd unit for termlink or for `/opt/999-...`** — the Watchtower on port 3002 is a manually-launched foreground process (pid 1471772), not a service.

**/etc grep for termlink path:** `termlink/.agentic-framework` does not appear in `/etc/` at all.

**Verdict:** No external automation depends on `/opt/termlink/.agentic-framework` being a symlink.

## Watchtower Impact

Evidence: see `/proc/1471772/` inspection above.

**Current state (BEFORE fix):**
- pid 1471772, port 3002
- PROJECT_ROOT=/opt/termlink, FRAMEWORK_ROOT=/opt/termlink/.agentic-framework
- CWD kernel-resolved to `/opt/999-Agentic-Engineering-Framework`
- log fds pointing at `/opt/999-.../.context/working/watchtower.log`

**After `rm .agentic-framework`:**
- The symlink goes away. The already-running process is unaffected because the kernel already resolved CWD at launch. File descriptors 1 and 2 remain valid (they reference the inode, not the path).
- **BUT:** any new call the process makes to `PROJECT_ROOT/.agentic-framework/...` will fail because the path no longer exists. Watchtower's own reload, route handlers that spawn subprocesses, and `fw` invocations from within the process will break.

**After `cp -r /opt/999-... /opt/termlink/.agentic-framework`:**
- The Watchtower process is still using the old CWD (`/opt/999-...`). It will keep writing to the framework repo's `.context/working/watchtower.log`.
- **Mandatory:** kill and restart Watchtower after the fix so CWD resolves to the new real directory.

**Restart procedure:**
```
kill 1471772
# Wait for port 3002 to free
ss -tlnp | grep :3002
# Restart from /opt/termlink (not from /opt/999-...)
cd /opt/termlink && .agentic-framework/bin/fw watchtower start --port 3002
```

**Also clean up pollution in framework repo:**
```
rm /opt/999-Agentic-Engineering-Framework/.context/working/watchtower.pid
rm /opt/999-Agentic-Engineering-Framework/.context/working/watchtower.log
# (These belong to termlink's watchtower. framework-repo's own watchtower, if any, will regenerate them on next start.)
```

Note: the framework repo currently shows `M .context/working/watchtower.pid` and `M .context/working/watchtower.log` in its git status — confirming these are currently modified by termlink's watchtower. The framework repo should probably revert those after cleanup.

## Upgrade Flow After Fix

Evidence: read of `/opt/999-.../lib/upgrade.sh` lines 1-180.

**Current flow (pre-fix):**
- `fw upgrade` in termlink reads `target_dir="${PROJECT_ROOT:-$PWD}"` → `/opt/termlink`
- `FRAMEWORK_ROOT` is resolved from `bin/fw` location → `/opt/termlink/.agentic-framework` → symlink → `/opt/999-...`
- Line 71: `if [ "$target_dir" = "$FRAMEWORK_ROOT" ]; then echo "Cannot upgrade the framework project itself"; return 1`
- Because `target_dir=/opt/termlink` and `FRAMEWORK_ROOT=/opt/termlink/.agentic-framework` — the string comparison PASSES (they're different strings). The symlink resolution doesn't collapse them in this check.
- However, any file copy from `FRAMEWORK_ROOT/*` into `target_dir/*` will read from `/opt/999-...` (via symlink) and write to `/opt/termlink`. This has been working — `last_upgrade: 2026-04-11T10:50:40Z` and `version: 1.5.246` in `.framework.yaml` confirm recent successful upgrades.

**After fix:**
- `FRAMEWORK_ROOT=/opt/termlink/.agentic-framework` (now a real dir, vendored copy at v1.5.246)
- `fw upgrade` reads from the vendored copy — but the vendored copy IS the upgrade source, so upgrade becomes a no-op until the vendored copy is refreshed.
- To upgrade termlink in the future, either: (a) refresh the vendored copy first (`fw vendor refresh` or manual cp), or (b) run `fw upgrade` pointing at `/opt/999-...` as the framework source explicitly.

**Recommendation for T-909 fix scope:** The fix must also document the new upgrade workflow for termlink, or add a `fw vendor refresh` helper that re-copies from `/opt/999-...` before running `fw upgrade`. Otherwise termlink will silently stop receiving framework updates.

**Critical exclude list for `cp -r`:** Do NOT blindly `cp -r /opt/999-... /opt/termlink/.agentic-framework`. The framework repo's working dirs contain active state:
- `.context/working/` (including `watchtower.log`, `session.yaml`, `focus.yaml`, stale fw-vec-index.db 77MB)
- `.context/audits/cron/` (1129 cron audit yaml files, some modified/deleted — framework's own audit history, not termlink's)
- `.tasks/active/`, `.tasks/completed/` (framework's OWN tasks, not termlink's — would collide with termlink's `.tasks/`)
- `.git/` (entire framework repo git history, 300+ MB)
- `target/` (Rust build artifacts, if present)

**Required exclude:** Use `rsync -a --exclude='.git' --exclude='.context/working' --exclude='.context/audits' --exclude='.tasks' --exclude='target' /opt/999-Agentic-Engineering-Framework/ /opt/termlink/.agentic-framework.new/ && rm /opt/termlink/.agentic-framework && mv /opt/termlink/.agentic-framework.new /opt/termlink/.agentic-framework`. Or compare to how other consumers were populated (look at `/opt/051-Vinix24/.agentic-framework/` as a template — it has only framework code, no framework-project state).

## Symlink Intent Investigation

Evidence:
```
grep -rn 'symlink|\.agentic-framework' /opt/termlink/.context/episodic/
grep -rn 'symlink|agentic' /opt/termlink/.tasks/completed/T-215-*.md
git -C /opt/termlink log --all --oneline | grep -iE 'T-288|symlink|vendor.*framework|\.agentic-framework'
```

**Historical evidence:**

1. **T-215 (2026-03-22)** — "Add .agentic-framework symlink and settings backup to gitignore". Treats the symlink as "machine-specific" and adds it to `.gitignore`. No rationale given for symlink vs real dir. Symlink was created the same day (`Mar 22 17:51` per `ls -la`).

2. **T-290 (2026-03-25)** — `L-027`-linked task. Fixed pre-push hook to explicitly pass PROJECT_ROOT because it was resolving wrong through the symlink. Commit `a2f46ef`: "Fix pre-push audit — resolve symlink so audit finds framework root". **This is the first evidence the symlink caused structural problems.**

3. **T-288 (2026-03-25)** — "termlink vendor — per-project binary isolation (same pattern as framework `.agentic-framework/`)". Treats the framework's `.agentic-framework/` pattern as the model to copy for termlink's own binary vendoring. **Confirms the intent was ALWAYS vendoring for consumer projects.** TermLink's symlink is an incidental, pre-convention holdover.

4. **T-825 (2026-04-??)** — "Fix pre-push hook audit resolution for `.agentic-framework` layout". Yet another hook fix caused by symlink traversal edge cases.

5. **T-659, T-234** — More audit/hook fixes tied to symlink resolution.

6. **`/opt/termlink/.context/project/decisions.yaml`** — does NOT exist. Only `assumptions.yaml`, `concerns.yaml`, `learnings.yaml`, etc. No ADR explaining the symlink.

**Conclusion:** The symlink was NOT an intentional design choice. It was created on day 1 of termlink's framework integration as a dev-convenience shortcut (symlink → live framework tree for fast edit-test cycles), and `.gitignore` + T-215 normalized it without anyone questioning whether termlink should match the other consumers' vendoring pattern. Every subsequent pre-push/audit/PROJECT_ROOT bug (T-234, T-290, T-659, T-825, and now T-909) traces back to this decision. **The symlink is technical debt, not intent.**

## Coordination Required

Evidence: `who`, `last -n 5`.

**Users on this machine:**
- `dimitri-mint-dev` (primary user, multiple pts sessions, tty7 GUI)
- `testdev` (home dir only, no active sessions)

Only one human operator. No team coordination needed.

**Things to communicate BEFORE running the fix:**
1. **Watchtower on port 3002 will need restart** — active process pid 1471772. Any browser tab open to `http://localhost:3002` will 500 until restart.
2. **Framework repo has uncommitted state modifications** caused by termlink's processes (`M .context/working/watchtower.pid`, `M .context/working/watchtower.log`, modified cron audit files). The human should decide whether to `git checkout` those in the framework repo after the fix, or leave them.
3. **Framework repo git status shows MANY deletions in `.context/audits/cron/2026-03-28-*.yaml`** (matching termlink's local git status). These deletions were caused by termlink's cron jobs running cleanup against the framework's cron dir via the symlink. **Check whether the framework repo should be reverted to HEAD or whether those deletions are legitimate** — this is a framework-repo decision, not a termlink decision.
4. **Future `fw upgrade` workflow for termlink changes** — after vendoring, the vendored copy becomes stale and termlink needs a refresh step. Document this in T-909's recommendation.
5. **The other stale consumer (`/opt/025-WokrshopDesigner` at v1.1.13)** is unrelated but flagged for awareness — it's been out of date for a long time.

**No multi-machine concerns** — this is a single dev workstation.

## Recommendation

**GO-WITH-CAVEATS.**

From the multi-project blast-radius angle, the fix is safe:
- No other project touches `/opt/termlink/.agentic-framework`.
- No cron job, systemd service, or CI workflow depends on termlink's symlink.
- TermLink's `.git`, `.context/`, and `.tasks/` are OUTSIDE `.agentic-framework` and are already isolated — no data loss from replacing the symlink.
- Historical evidence shows the symlink was never intentional; every prior bug traces back to it.

**Required caveats (must be included in the T-909 fix plan):**

1. **Stop Watchtower first.** Kill pid 1471772 before `rm .agentic-framework`, restart after vendor copy.
2. **Use `rsync` with excludes**, not naive `cp -r`. The framework repo contains active state (`.context/working/`, `.context/audits/cron/`, `.tasks/`, `.git/`, 77MB `fw-vec-index.db`, possibly `target/`) that must not be cloned into termlink. Suggested command:
   ```
   rsync -a \
     --exclude='.git/' \
     --exclude='.context/working/' \
     --exclude='.context/audits/' \
     --exclude='.tasks/active/' \
     --exclude='.tasks/completed/' \
     --exclude='target/' \
     --exclude='node_modules/' \
     --exclude='*.bak' \
     /opt/999-Agentic-Engineering-Framework/ \
     /opt/termlink/.agentic-framework.new/
   ```
   Then atomic-swap: `rm .agentic-framework && mv .agentic-framework.new .agentic-framework`.
3. **Clean up framework-repo pollution:** after the fix, inspect `/opt/999-.../.context/working/watchtower.pid` and `.log`, and the cron audit deletions in `git status`. Decide whether to `git checkout` them in the framework repo.
4. **Define the new upgrade workflow** — how does termlink receive framework updates after vendoring? Add this to the T-909 recommendation BEFORE executing the fix. Option A: `fw vendor refresh` re-runs the rsync. Option B: `fw upgrade --from /opt/999-...` with explicit source. Option C: symlink `.agentic-framework` only during `fw upgrade` and unlink after.
5. **Add `fw doctor` check** — detect if a consumer project's `.agentic-framework` is a symlink, warn/block. This prevents regression (already listed as mitigation in G-001).
6. **Verify termlink git hooks still work** post-fix. Hooks reference `.agentic-framework/bin/fw` relatively (per `.claude/settings.json`), so they should work with a real dir identically. But run the audit + pre-push smoke test to confirm.
7. **Do not touch `/root/.agentic-framework`.** It is a separate v1.4.520 install used by ALL termlink cron jobs. Leaving it alone is correct.

**Estimated fix duration:** 10-15 min (stop watchtower, rsync with excludes, restart watchtower, smoke test `fw audit`, `fw doctor`, `git status`).

**Fallback / rollback:** Keep the removed symlink target known (`/opt/999-Agentic-Engineering-Framework`). If anything breaks, `rm -rf /opt/termlink/.agentic-framework && ln -s /opt/999-Agentic-Engineering-Framework /opt/termlink/.agentic-framework` restores the status quo in < 5 seconds.

**Multi-project angle: APPROVED.** No cross-project concerns block the fix. The only risks are local to termlink and framework-repo state cleanup.
