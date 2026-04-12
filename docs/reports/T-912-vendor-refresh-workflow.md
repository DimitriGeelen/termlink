# T-912: Vendor-Refresh Workflow Research

## Problem

`fw upgrade` section 4b blindly copies framework files to the consumer's vendored `.agentic-framework/` directory. Local modifications are overwritten without warning, backup, or diff.

**Incident:** Session S-2026-0412-1253 — fw upgrade v1.5.356 silently destroyed T-962, T-970/T-971, T-963 local fixes.

## Spike 1: Quantify the Problem

### What upgrade.sh syncs (section 4b)

| Category | Source pattern | Count |
|----------|---------------|-------|
| Hook scripts | `agents/context/*.sh` | ~15 files |
| Hook libraries | `agents/context/lib/*` | ~5 files |
| bin/fw | `bin/fw` | 1 file |
| lib/*.sh | `lib/*.sh` | ~12 files |
| Agent scripts | `agents/{task-create,handover,git,...}/**` | ~25 files |
| VERSION | `VERSION` | 1 file |
| **Total** | | **~60 files** |

### Which files had local changes in the v1.5.356 incident

| File | Local change (task) | Destroyed by upgrade? |
|------|--------------------|-----------------------|
| `lib/compat.sh` | T-962: date helpers | Yes |
| `lib/review.sh` | T-970/T-971: port/browser | Yes |
| `lib/init.sh` | T-963: concerns init | Yes |
| `lib/watchtower.sh` | T-974: new file | Survived (not in upstream) |

**Observation:** 3/60 files had local changes. All 3 were in `lib/*.sh`. New files survive because `cp` only overwrites existing targets when the source exists.

## Spike 2: Options Analysis

### Option A: Checksum Manifest

**How:** On each upgrade, record `sha256sum` of every synced file in `.agentic-framework/.upstream-checksums`. Before the next upgrade, compare local file checksum against the manifest. If different → file was locally modified → skip and warn.

**Pros:**
- Simple to implement (~30 lines added to upgrade.sh)
- No external dependencies
- Backward-compatible (first run creates manifest, no manifest = upgrade all)
- Works with .gitignore (no git tracking needed)

**Cons:**
- First upgrade after adoption has no protection (no baseline checksums yet)
- No merge capability — just skip or overwrite

### Option B: Git Subtree

**How:** Use `git subtree` to track `.agentic-framework/` as a subtree from the upstream framework repo.

**Pros:**
- Full merge capability with conflict resolution
- Git history preserved

**Cons:**
- Requires `.agentic-framework/` NOT in .gitignore (breaking change)
- Adds git complexity
- Many consumer projects have private repos — subtree from a different repo requires access

### Option C: Patch-Based

**How:** Consumer maintains `.framework-patches/` with patch files. Upgrade applies upstream, then re-applies patches.

**Pros:**
- Clean separation of upstream vs local

**Cons:**
- Patches can fail to apply after significant upstream changes
- Consumer must create and maintain patches manually
- Overkill for 3/60 files

### Option D: Backup + Diff

**How:** Before overwriting any file, create `.agentic-framework/.upgrade-backup/` with the old version. After upgrade, show a diff summary of what changed. If local modifications detected, offer `fw upgrade --restore` to roll back individual files.

**Pros:**
- Always safe — backup means zero data loss risk
- Combined with checksum manifest, gives best of both worlds
- User can review changes post-upgrade

**Cons:**
- Backup directory can accumulate (needs cleanup)
- Slightly more complex than pure checksum

## Recommendation

**Option A + D combined: Checksum manifest with backup.**

1. On upgrade, check each file against `.upstream-checksums`
2. If file is locally modified: **back up** to `.upgrade-backup/`, then **warn** (but still copy)
3. After upgrade, show summary: "N files had local modifications — backed up to .upgrade-backup/"
4. Provide `fw upgrade --restore <file>` to recover individual files
5. With `--force`: skip the warning (current behavior)
6. Without `--force` and with local changes: prompt for confirmation

This gives protection without blocking upgrades. The backup ensures recoverability. The checksum manifest enables detection.

### Build Tasks (if GO)

1. **T-XXX: Add checksum manifest to fw upgrade** — Record `.upstream-checksums` after each sync, detect local modifications before next sync
2. **T-XXX: Add backup + restore to fw upgrade** — Create `.upgrade-backup/` for modified files, add `fw upgrade --restore`
