# T-909 Symlink Fix — Technical Risk Evaluation

**Angle:** Technical / Path Resolution / Tooling
**Evaluator:** Sub-agent (independent)
**Date:** 2026-04-11
**Scope:** Replace `/opt/termlink/.agentic-framework` (currently a symlink to `/opt/999-Agentic-Engineering-Framework`) with a vendored real directory.

## Summary

**Verdict: GO-WITH-CAVEATS.** The swap is technically safe right now — no process holds open handles inside the symlink, no termlink process is cwd'd there, and the project's own `.context/` / `.tasks/` / `.fabric/` already live under `$PROJECT_ROOT` (not through the symlink). **Vendoring will actually *close* latent failure paths** (notably `fw update`'s rsync-through-symlink that would overwrite the framework repo). The caveats concern preserving the `.framework.yaml` metadata, choosing the vendoring method (use `fw vendor`, not raw `cp -r`), and handling one cosmetic doctor/audit reporting change.

## Tools Surveyed

| Tool / Script | Path Resolution Method | Current Behavior (symlink) | Post-Swap Behavior | Impact |
|---|---|---|---|---|
| `bin/fw` (main CLI) `L52-53` | `readlink -f "$0"`, then `cd ... && pwd` on the resolved dir. `resolve_framework` at L72 returns `"$(cd "$FW_BIN_DIR/.." && pwd)"`. | `FRAMEWORK_ROOT = /opt/999-Agentic-Engineering-Framework` (symlink resolved). | `FRAMEWORK_ROOT = /opt/termlink/.agentic-framework` (now a real dir). | **Behavioral change**: this is the *correct* consumer mode. All downstream libs now see the project-local framework. |
| `bin/fw-shim` `L22-27` | Walks up from CWD looking for `bin/fw + FRAMEWORK.md` or `.agentic-framework/bin/fw`. No `readlink`. | Finds `/opt/termlink/.agentic-framework/bin/fw` (symlink target exec'd). | Same walk, finds real `.agentic-framework/bin/fw`. | **No change.** Shim is symlink-agnostic. |
| `bin/claude-fw` | Uses `git rev-parse --show-toplevel` for signal file. No `.agentic-framework` path construction. | N/A | N/A | **No change.** |
| `bin/watchtower.sh` `L16-22` | `SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)` → **does not resolve symlinks**. `PID_FILE=$FRAMEWORK_ROOT/.context/working/watchtower.pid` | If invoked via `.agentic-framework/bin/watchtower.sh`, writes PID/log into the symlinked target's `.context` (i.e., **into the framework repo**). Currently termlink has no watchtower running, so not observed. | Writes PID/log into `/opt/termlink/.agentic-framework/.context/working/` (a real directory the swap creates). | **Fix**: resolves a latent cross-project state pollution bug. If termlink ever started watchtower via its own path, it would have polluted the framework repo. |
| `lib/paths.sh` `L28, L34` | `FRAMEWORK_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"`; PROJECT_ROOT from `git -C "$FRAMEWORK_ROOT" rev-parse --show-toplevel`. | When sourced through symlink path, `cd+pwd` follows the literal path → `/opt/termlink/.agentic-framework`. But sourced by `fw` which passes `FRAMEWORK_ROOT` env-exported from L398 (= `/opt/999-...`), the guard at L23 prevents override. | Unchanged semantics — FRAMEWORK_ROOT now equals the real vendored dir. | **No functional change** for normal `fw` invocations. |
| `lib/update.sh` `L58-216` | `vendored_dir="$project_root/.agentic-framework"`, `rsync -a --delete ... "$vendored_dir/$item/"` | **DANGEROUS LATENT BUG**: rsync `--delete` through symlink writes into `/opt/999-Agentic-Engineering-Framework/bin/`, `lib/`, `agents/`. Detects `.agentic-framework/VERSION` at L60 — and the framework repo **does** have a VERSION file, so `fw update` would take the vendored branch and mutate the framework source. Not triggered because TermLink has been running `fw upgrade`, not `fw update`. | rsync writes into the real, isolated directory. | **Fix**: eliminates a trap that would corrupt the framework repo on first `fw update`. |
| `lib/upgrade.sh` `L320-443` | Individual `cp` per file, guarded by `diff -q` to skip identical files. | Copies source→dest where both resolve to the same file (symlink). `diff -q` returns equal → skip. No damage, but nothing ever actually syncs for termlink either. | Real copy into vendored dir — upgrade finally does real work. | **Fix**: `fw upgrade` starts behaving correctly for termlink. |
| `lib/version.sh` `L78-84, 189-202, 310-328` | `vendored_version="$FRAMEWORK_ROOT/.agentic-framework/VERSION"`, `vendored_bin="$FRAMEWORK_ROOT/.agentic-framework/bin/fw"` | Uses `$FRAMEWORK_ROOT = /opt/999-...` → looks for `/opt/999-.../​.agentic-framework/VERSION` (does not exist, version bump script skips). | Unchanged (this code only runs inside the framework repo, and `FRAMEWORK_ROOT != PROJECT_ROOT` for termlink). | **No change.** |
| `agents/audit/audit.sh L32` | `FW_PATH=$(readlink -f "$FRAMEWORK_ROOT/bin/fw" 2>/dev/null || echo …)` | Resolves through and gets framework-repo path. | Resolves to real vendored `bin/fw`. | **No functional change** — audit only uses FW_PATH for reporting. |
| `.git/hooks/commit-msg`, `pre-push` | Uses `$PROJECT_ROOT/.agentic-framework/...` (relative). | Traverses symlink for lib lookup. | Traverses real dir. | **No change.** |
| `.claude/settings.json` (15 hooks) | All use relative `.agentic-framework/bin/fw hook <name>`. | Resolves via the symlink. | Resolves via real dir. | **No change.** |
| `lib/harvest.sh L74-75, 363` | Writes *framework-side* harvest state to `$FRAMEWORK_ROOT/.context/...`. | Writes to `/opt/999-.../.context/` (intentional — harvest is a cross-project command). | Writes to `/opt/termlink/.agentic-framework/.context/` — **wrong!** Vendored dir is not the "real" framework. | **REGRESSION RISK**: `fw harvest` run from termlink would post learnings into the vendored copy instead of the framework repo. See Post-Swap Risks. |

## During-Swap Window Risks

**Checked at evaluation time (2026-04-11 ~13:38):**

1. **No process holds open handles under `/opt/termlink/.agentic-framework`.**
   - `lsof +D /opt/termlink/.agentic-framework/` returned empty (exit 1).
   - `ls -la /proc/*/cwd | grep /opt/termlink/.agentic-framework` returned nothing.
   - `ls /proc/*/cwd | xargs readlink | grep ^/opt/termlink` returned nothing — no process is even cwd'd under termlink.

2. **Watchtower processes on ports 3001/3002 belong to OTHER projects, not termlink.**
   - PID `1471772` (port 3002): cwd = `/opt/999-Agentic-Engineering-Framework`. This is the framework repo's own watchtower.
   - PID `4170601` (port 3001): cwd = `/opt/025-WokrshopDesigner/.agentic-framework`. Unrelated project.
   - Neither process holds file descriptors into `/opt/termlink/.agentic-framework` (verified via lsof output — all their FDs are in their own cwd).

3. **Active hook invocation observed during probe**: `.agentic-framework/bin/fw hook check-project-boundary` (PID 1495806, bash). This is **this session's own** Claude Code PreToolUse hook, firing per-command. It executes to completion in ~100-300ms and does not hold long-lived handles. **Mitigation**: execute the swap *between* tool calls (the rm+cp sequence is atomic from the agent's perspective because a single Bash invocation runs it before any next hook fires).

4. **Git hooks**: `.git/hooks/commit-msg`, `post-commit`, `pre-push` are all event-triggered. None run continuously.

5. **Swap atomicity**: The proposed `rm .agentic-framework && cp -r ... .agentic-framework` creates a ~0.1-1s window where `.agentic-framework` does not exist. During this window any hook invocation would fail with `No such file or directory`. **Mitigation**: use the swap pattern below (stage alongside, then atomic rename).

6. **Git tracking**: `.gitignore:18` contains `.agentic-framework`. `git ls-files .agentic-framework` returns empty. **No git staging side-effects** from `rm`.

## Post-Swap Risks

### R1 — `fw harvest` destination changes (MODERATE)
Path: `lib/harvest.sh:74-75, 363`. Runs `$FRAMEWORK_ROOT/.context/project`. Pre-swap, FRAMEWORK_ROOT resolves via readlink to the framework repo → harvest writes into the canonical framework context (correct). Post-swap, FRAMEWORK_ROOT = vendored dir → harvest writes into `.agentic-framework/.context/project/` inside termlink, which is not the framework repo and is under termlink's `.gitignore`. **Impact**: harvested learnings from termlink would be lost / not propagated. **Mitigation**: if `fw harvest` is ever run from termlink, pass an explicit upstream source, or re-route harvest via the framework repo directly. Verify `/opt/999-.../.context/harvest.log` after any harvest operation. Low severity — harvest is rarely run from consumer projects.

### R2 — `fw doctor` / audit cosmetic changes (LOW)
Path: `bin/fw L719-723` and similar. When `FRAMEWORK_ROOT = PROJECT_ROOT` checks, plus the test-infrastructure SKIP message, the output will now identify termlink's `.agentic-framework` as its framework root. `fw doctor` currently reports `/opt/999-Agentic-Engineering-Framework` as Framework — post-swap it will report `/opt/termlink/.agentic-framework`. This is a **reporting-only change**; no behavior changes. Update any operator docs that cite the current path.

### R3 — Stale path caches in running sessions (LOW)
Any currently-running long-lived shell or editor that captured FRAMEWORK_ROOT into an env var at start time would keep the old resolution (`/opt/999-...`). Nothing within termlink currently does this (verified: no process cwd'd under termlink, no lingering env with stale FRAMEWORK_ROOT tied to termlink). **Mitigation**: after swap, any new `fw` invocation picks up the new resolution automatically since `fw` re-resolves on every call.

### R4 — Version drift vs framework repo (MODERATE — *expected*)
Post-swap, the vendored copy is a snapshot. The live framework repo will drift as development continues. That is **the intended design** — `fw update` and `fw upgrade` are the sync mechanisms. **Action required**: the first `fw upgrade` after the swap should be run deliberately to verify it syncs cleanly (with the real dir, not the symlink).

### R5 — `.framework.yaml` `version:` field drift (LOW)
Currently `version: 1.5.246` in `.framework.yaml`, but `fw version` reports `1.5.250` (derived from the framework repo's git describe). Post-swap, the vendored copy has no `.git`, so `fw version` falls back to the VERSION file — which `fw vendor` populates with the snapshot version. The `.framework.yaml` `version:` field should be updated to match, or left to be fixed by the next `fw upgrade` (which syncs VERSION files at upgrade.sh L425-432).

### R6 — `upgrade.sh L425-432` VERSION stamping (LOW)
First post-swap `fw upgrade` run will write `$FRAMEWORK_ROOT/VERSION` = `$fw_version` into the vendored dir. Verify the VERSION value is sensible (should be the current framework git-derive version, not "dev" or similar).

### R7 — Disk space (TRIVIAL)
`fw vendor` copies ~50MB (bin + lib + agents + web + docs + FRAMEWORK.md + metrics.sh). Full framework repo is 349MB (125MB of which is .git, 102MB .context). Naïve `cp -r` would copy all 349MB. **Use `fw vendor`, not `cp -r`** — it excludes `.git`, `.context/`, `.tasks/{active,completed}`, `.fabric`, `__pycache__`, which is the whole point of the vendor workflow.

## Mitigations — Concrete Commands

### Before the swap
```bash
# 1. Verify no process is using the symlink
lsof +D /opt/termlink/.agentic-framework/ 2>/dev/null | head
# Expect: empty

# 2. Verify git tracking state
git -C /opt/termlink ls-files .agentic-framework
# Expect: empty (confirmed — .gitignore:18)

# 3. Save pre-swap readlink for rollback
readlink /opt/termlink/.agentic-framework
# Expect: /opt/999-Agentic-Engineering-Framework

# 4. Record current fw resolution for comparison
cd /opt/termlink && PROJECT_ROOT=/opt/termlink .agentic-framework/bin/fw version
```

### The swap (atomic-style, stage then rename)
**DO NOT use raw `cp -r`.** Use `fw vendor` which has exclusion logic, self-referencing guards (bin/fw L159-178), and deterministic output:

```bash
cd /opt/termlink && \
  mv .agentic-framework .agentic-framework.symlink.bak && \
  /opt/999-Agentic-Engineering-Framework/bin/fw vendor \
    --target /opt/termlink \
    --source /opt/999-Agentic-Engineering-Framework
```

If `fw vendor` refuses (e.g., self-referencing detection), fall back to:

```bash
cd /opt/termlink && \
  mv .agentic-framework .agentic-framework.symlink.bak && \
  rsync -a \
    --exclude=.git --exclude=.context --exclude=.fabric \
    --exclude='.tasks/active' --exclude='.tasks/completed' \
    --exclude=__pycache__ --exclude='*.pyc' --exclude='.DS_Store' \
    /opt/999-Agentic-Engineering-Framework/ .agentic-framework/
```

The `mv` (not `rm`) preserves the symlink as a rollback target in case something goes wrong mid-vendor.

### After the swap
```bash
# 1. Verify resolution changed
cd /opt/termlink && .agentic-framework/bin/fw version
# Expect: Framework: /opt/termlink/.agentic-framework

# 2. Verify doctor still passes
cd /opt/termlink && .agentic-framework/bin/fw doctor 2>&1 | tail -20
# Expect: same pass/warn counts as pre-swap (minus the "version mismatch" which vendoring fixes)

# 3. Verify hooks still fire
cd /opt/termlink && .agentic-framework/bin/fw hook check-active-task < /dev/null
# Expect: exits without "file not found"

# 4. Verify watchtower path is correctly rooted (only if starting it)
# bin/watchtower.sh writes to $FRAMEWORK_ROOT/.context/working/watchtower.pid
# For termlink, we want PID in /opt/termlink/.context/working/, not .agentic-framework/.context/
# NOTE: this is a DISTINCT concern from the symlink fix — watchtower writes to
# FRAMEWORK_ROOT not PROJECT_ROOT, which is an unfixed bug either way.

# 5. Run a test audit
cd /opt/termlink && .agentic-framework/bin/fw audit 2>&1 | tail -20

# 6. Sync .framework.yaml version field
cd /opt/termlink && .agentic-framework/bin/fw upgrade
# This runs upgrade.sh section 4b which syncs vendored scripts (now no-ops since just vendored)
# And updates .framework.yaml version to match.

# 7. Remove backup once satisfied
rm /opt/termlink/.agentic-framework.symlink.bak
```

### Rollback (if needed)
```bash
cd /opt/termlink && \
  rm -rf .agentic-framework && \
  mv .agentic-framework.symlink.bak .agentic-framework
```

## Recommendation

**GO-WITH-CAVEATS** from the technical angle.

**Why GO:**
- The technical environment is safe: zero processes depend on the symlink path right now.
- The swap *fixes* real latent bugs (watchtower PID path, `fw update` rsync-through-symlink, `fw upgrade` effectively no-opping for termlink).
- Git hooks and Claude Code hooks use relative paths — symlink vs real dir is transparent to them.
- The framework itself is designed for the vendored model; TermLink is the anomaly, not the reference.

**Caveats (in priority order):**
1. **Use `fw vendor`, not raw `cp -r`.** `cp -r` would copy 349MB including `.git`, `.context`, and hundreds of task files — not what's wanted. `fw vendor` (bin/fw L118-278) has the right exclusion list and self-referencing guard.
2. **Stage via `mv` to `.symlink.bak`, not `rm`, for atomic rollback.** A failed vendor mid-flight leaves the project unusable if the symlink is already gone.
3. **Do not run `fw harvest` from termlink post-swap** until harvest.sh is fixed or re-routed — it will write learnings into the vendored copy instead of the framework repo (harvest.sh:74-75). This is a pre-existing design consideration, not a swap regression, but the swap makes it observable.
4. **After vendoring, run `fw upgrade` once** to sync the `.framework.yaml` `version:` field and stamp VERSION correctly, then commit the vendored dir (respecting `.gitignore:18` — it stays ignored, but the staged/committed state of `.framework.yaml` and any touched hook files should be clean).
5. **Update T-909-symlink-fix.md** to cite `/opt/termlink/.agentic-framework` as the new FRAMEWORK_ROOT for cross-project documentation accuracy.

The technical risk is low; the main hazards come from *how* the swap is executed (raw `cp -r` vs `fw vendor`), not from whether it's executed at all.

---

**Files cited:**
- `/opt/999-Agentic-Engineering-Framework/bin/fw` (L52-53, L68-108, L115-278, L397-418, L719-723)
- `/opt/999-Agentic-Engineering-Framework/bin/fw-shim` (L22-34)
- `/opt/999-Agentic-Engineering-Framework/bin/claude-fw` (L87-109)
- `/opt/999-Agentic-Engineering-Framework/bin/watchtower.sh` (L16-22)
- `/opt/999-Agentic-Engineering-Framework/lib/paths.sh` (L23-42)
- `/opt/999-Agentic-Engineering-Framework/lib/upgrade.sh` (L320-446)
- `/opt/999-Agentic-Engineering-Framework/lib/update.sh` (L56-232)
- `/opt/999-Agentic-Engineering-Framework/lib/version.sh` (L78-84, L189-202, L310-328)
- `/opt/999-Agentic-Engineering-Framework/lib/harvest.sh` (L66-75, L363)
- `/opt/999-Agentic-Engineering-Framework/agents/audit/audit.sh` (L32)
- `/opt/999-Agentic-Engineering-Framework/agents/git/lib/hooks.sh` (L94-95, L326-358)
- `/opt/termlink/.gitignore` (L18)
- `/opt/termlink/.framework.yaml` (version field)
- `/opt/termlink/.claude/settings.json` (L9-141, 15 hook commands — all relative paths)
- `/opt/termlink/.git/hooks/commit-msg` (L43-51)
- `/opt/termlink/.git/hooks/pre-push` (L19-52)

**Process probe results (live at evaluation time):**
- `/proc/1471772/cmdline`: `python3 -m web.app --port 3002`, cwd `/opt/999-Agentic-Engineering-Framework`
- `/proc/4170601/cmdline`: `python3 -m web.app --port 3001`, cwd `/opt/025-WokrshopDesigner/.agentic-framework`
- No process cwd under `/opt/termlink/`, no file descriptor under `/opt/termlink/.agentic-framework/`.

DONE-T909-TECH
