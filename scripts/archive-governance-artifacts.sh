#!/usr/bin/env bash
# archive-governance-artifacts.sh — T-2998 (value-review C-25).
#
# C-25 measured governance-artifact volume (.context/handovers, .context/audits)
# growing unboundedly against a ~6MB product-source baseline (105MB total at
# measurement time, .context/handovers alone 59MB / 1,706 files, a 24:1 touch
# ratio) and recommended applying this project's own T-2562 retention discipline
# (compress/archive old records rather than deleting them or letting them grow
# forever) to its OWN filesystem trees, not just TermLink channel topics.
#
# Moves (never deletes) files older than ARCHIVE_CUTOFF_DAYS (default 90) from
# .context/handovers and .context/audits into a same-named archive/ subtree,
# preserving relative subdirectory structure. Content is unchanged — this is a
# plain relocation, not a compression pipeline, kept deliberately simple: the
# reviewer's own success framing ("<20MB active") is about the ACTIVE working
# set an operator/tool scans, not total bytes on disk, and a relocation alone
# satisfies that without adding restore tooling this task did not ask for.
#
# Age source (T-2801 convention: prefer a RECORDED timestamp over mtime — a
# checkout/clone/worktree stamps every file with checkout time, not the file's
# real history). Every handover/audit filename in this corpus already encodes
# its own date (S-YYYY-MMDD-HHMM.md, YYYY-MM-DD.yaml, YYYYMMDDHH.txt, ...), so
# the date is parsed from the FILENAME. A file with no parseable date is never
# guessed at via mtime — it is left alone and counted as "undated", because
# several real files in this corpus (orchestrator-mcp-baseline.yaml,
# arc-008-cycle*-census.md, upgrades.yaml) are long-lived reference documents
# without a date in their name, not stale logs, and archiving one of those on
# an mtime guess would be the exact class of accidental-loss T-2811 warns about.
#
# Never touches: symlinks (e.g. LATEST.md pointers), any file named LATEST*
# (including in-place-rewritten pointer files with no symlink), or anything
# already under an archive/ subtree.
#
# NOT a guard-layer member: this mutates the filesystem (moves files) and is
# named archive-*, not check-*, so run-guard-layer.sh's check-*.sh glob never
# picks it up — correctly, since guard-layer members must be read-only/hermetic.
set -euo pipefail

DRY_RUN=0
CUTOFF_DAYS="${ARCHIVE_CUTOFF_DAYS:-90}"
JSON=0
DIRS=(".context/handovers" ".context/audits")

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=1 ;;
        --json) JSON=1 ;;
        --cutoff-days=*) CUTOFF_DAYS="${arg#*=}" ;;
        --help|-h)
            echo "Usage: $0 [--dry-run] [--json] [--cutoff-days=N]"
            exit 0
            ;;
        *)
            echo "unknown arg: $arg" >&2
            exit 2
            ;;
    esac
done

if ! command -v git >/dev/null 2>&1; then
    echo "archive-governance-artifacts: git not found" >&2
    exit 2
fi

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$REPO_ROOT" ]; then
    echo "archive-governance-artifacts: not a git repo" >&2
    exit 2
fi
cd "$REPO_ROOT"

cutoff_epoch=$(date -u -d "-${CUTOFF_DAYS} days" +%s)

archived=0
skipped_recent=0
skipped_latest=0
skipped_undated=0
skipped_symlink=0
archived_paths=()
undated_paths=()

for base in "${DIRS[@]}"; do
    [ -d "$base" ] || continue
    while IFS= read -r -d '' f; do
        rel="${f#"$base"/}"
        case "$rel" in
            archive/*) continue ;;
        esac
        base_name="$(basename "$f")"
        case "$base_name" in
            LATEST*) skipped_latest=$((skipped_latest + 1)); continue ;;
        esac
        if [ -L "$f" ]; then
            skipped_symlink=$((skipped_symlink + 1))
            continue
        fi

        # Extract a YYYY[-]MM[-]DD date from the filename only — never mtime.
        if [[ "$base_name" =~ (20[0-9]{2})-?([0-9]{2})-?([0-9]{2}) ]]; then
            y="${BASH_REMATCH[1]}"; mo="${BASH_REMATCH[2]}"; d="${BASH_REMATCH[3]}"
            file_epoch=$(date -u -d "${y}-${mo}-${d}" +%s 2>/dev/null || echo "")
        else
            file_epoch=""
        fi

        if [ -z "$file_epoch" ]; then
            skipped_undated=$((skipped_undated + 1))
            undated_paths+=("$f")
            continue
        fi

        if [ "$file_epoch" -ge "$cutoff_epoch" ]; then
            skipped_recent=$((skipped_recent + 1))
            continue
        fi

        rel_dir="$(dirname "$rel")"
        if [ "$rel_dir" = "." ]; then
            dest_dir="$base/archive"
        else
            dest_dir="$base/archive/$rel_dir"
        fi

        if [ "$DRY_RUN" = "1" ]; then
            echo "[DRY-RUN] would archive: $f -> $dest_dir/$base_name"
        else
            mkdir -p "$dest_dir"
            # git mv -k silently no-ops (exit 0, file left in place) for a file
            # git does not track (e.g. .context/audits/cron/ is gitignored) —
            # verified by hand before wiring this in. Check tracked status
            # explicitly rather than trusting -k's exit code, or an "archived"
            # file would silently stay exactly where it started.
            if git ls-files --error-unmatch -- "$f" >/dev/null 2>&1; then
                git mv -- "$f" "$dest_dir/$base_name"
            else
                mv -- "$f" "$dest_dir/$base_name"
            fi
        fi
        archived=$((archived + 1))
        archived_paths+=("$f")
    done < <(find "$base" -type f -print0)
done

if [ "$JSON" = "1" ]; then
    python3 - "$archived" "$skipped_recent" "$skipped_latest" "$skipped_undated" "$skipped_symlink" "$CUTOFF_DAYS" "$DRY_RUN" <<'PYEOF'
import json, sys
archived, skipped_recent, skipped_latest, skipped_undated, skipped_symlink, cutoff_days, dry_run = sys.argv[1:8]
print(json.dumps({
    "ok": True,
    "archived": int(archived),
    "skipped_recent": int(skipped_recent),
    "skipped_latest": int(skipped_latest),
    "skipped_undated": int(skipped_undated),
    "skipped_symlink": int(skipped_symlink),
    "cutoff_days": int(cutoff_days),
    "dry_run": bool(int(dry_run)),
}))
PYEOF
else
    echo "archived=$archived skipped_recent=$skipped_recent skipped_latest=$skipped_latest skipped_undated=$skipped_undated skipped_symlink=$skipped_symlink cutoff_days=$CUTOFF_DAYS dry_run=$DRY_RUN"
    if [ "$skipped_undated" -gt 0 ]; then
        echo "undated (left alone, review manually if they are actually stale logs):"
        printf '  %s\n' "${undated_paths[@]}"
    fi
fi
