#!/usr/bin/env bash
# target-cache-trim.sh — weekly trim of this repo's cargo debug cache (T-3799)
#
# WHY: target/debug regrew to 88 GB in about 20 days of ordinary cargo build/test, on /opt
# (btrfs, Timeshift-snapshotted) at 89%. The 2026-10-26 operator request to clean it sat
# unanswered for 20 days (T-3798); the operator asked for a weekly cap on 2026-11-15.
#
# WHAT: when target/debug is at least TRIM_MIN_GB, run `cargo clean --profile dev`, and
# clean target/arc011-mutants (the mutant scripts' build dir). release, musl and local-fast
# are kept: release artefacts may be what an install step copies from.
# NEVER while a build runs: skipped when any cargo/rustc process exists, or when cargo's
# build lock (target/debug/.cargo-lock) is held.
# Verified on disk: after the clean, target/debug must be gone or under 1 GiB, else FAILED.
#
# T-3820 (ruling A): also removes cargo build dirs under .context/working/ untouched for 3+ days (TRIM_SCRATCH_DAYS).
# Usage: target-cache-trim.sh [--dry-run] [--quiet]
#   --quiet  print nothing when there is nothing to do (below the threshold)
# Exit: 0 trimmed / nothing to do / skipped busy · 1 the clean ran but space was not freed ·
#       2 tooling (cargo missing, target dir unreadable)
# Seams (fixtures only): TRIM_ROOT (repo), TRIM_MIN_GB, TRIM_CARGO (cargo binary),
#   TRIM_BUSY_PATTERN (pgrep -f regex), TRIM_SIZE_CMD (prints GiB of a dir: <cmd> <dir>)
set -uo pipefail

ROOT="${TRIM_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
MIN_GB="${TRIM_MIN_GB:-30}"
CARGO="${TRIM_CARGO:-$(command -v cargo || echo /root/.cargo/bin/cargo)}"
BUSY_PATTERN="${TRIM_BUSY_PATTERN:-^(/[^ ]*/)?(cargo|rustc|cargo-mutants)( |$)}"
DRY=0 QUIET=0
for a in "$@"; do
    case "$a" in
        --dry-run) DRY=1 ;;
        --quiet) QUIET=1 ;;
        -h|--help) sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "target-cache-trim: unknown arg: $a" >&2; exit 2 ;;
    esac
done

ts() { date -u +%Y-%m-%dT%H:%M:%SZ; }
say() { echo "$(ts) $*"; }
gib() {  # <dir> -> whole GiB used (0 when absent)
    [ -d "$1" ] || { echo 0; return; }
    if [ -n "${TRIM_SIZE_CMD:-}" ]; then $TRIM_SIZE_CMD "$1"; return; fi
    du -s --block-size=1G "$1" 2>/dev/null | cut -f1
}

# ---------------------------------------------------------------------------------------------------------------
# T-3820 (operator ruling A, 2026-11-19): cargo build dirs left under .context/working/ by scratch and worker builds
# (90 GB of them filled /opt to 97% on 2026-11-18). Removed when their newest file is older than SCRATCH_DAYS; only
# dirs cargo made (a `target` / `target-*` dir holding CACHEDIR.TAG or a debug/ or release/ subdir), never through a
# symlink, never outside .context/working/, never while a build runs. Each removal is verified on disk.
SCRATCH_DAYS="${TRIM_SCRATCH_DAYS:-3}"
SCRATCH_FAILED=0
finish() { local rc=$?; if [ "$SCRATCH_FAILED" = 1 ] && [ "$rc" = 0 ]; then exit 1; fi; exit "$rc"; }
trap finish EXIT

is_cargo_dir() { [ -e "$1/CACHEDIR.TAG" ] || [ -d "$1/debug" ] || [ -d "$1/release" ]; }

scratch_trim() {
    local work="$ROOT/.context/working" real_work d real n=0 freed=0 g
    [ -d "$work" ] || return 0
    real_work="$(readlink -f "$work")"
    if [ -n "$(pgrep -af "$BUSY_PATTERN" 2>/dev/null | head -1)" ]; then
        say "skip    scratch build dirs under .context/working: a build is running; next week"
        return 0
    fi
    while IFS= read -r -d '' d; do
        real="$(readlink -f "$d")"
        case "$real" in "$real_work"/*) ;; *) continue ;; esac
        [ -L "$d" ] && continue   # find -P never descends a symlinked dir; a symlinked target itself is skipped
        is_cargo_dir "$d" || continue
        # newest file younger than SCRATCH_DAYS: in use, keep
        [ -n "$(find "$d" -newermt "-${SCRATCH_DAYS} days" -print -quit 2>/dev/null)" ] && continue
        g="$(gib "$d")"
        if [ "$DRY" = 1 ]; then
            say "[DRY]   would remove scratch build dir ${d#"$ROOT"/} (${g} GiB, untouched ${SCRATCH_DAYS}+ days)"; n=$((n+1)); continue
        fi
        rm -rf -- "$d"
        if [ -e "$d" ]; then
            say "FAILED  could not remove scratch build dir ${d#"$ROOT"/}"; SCRATCH_FAILED=1
        else
            n=$((n+1)); freed=$((freed+${g:-0}))
        fi
    done < <(find "$work" -xdev -type d \( -name target -o -name 'target-*' \) -prune -print0 2>/dev/null)
    if [ "$DRY" != 1 ] && [ "$n" -gt 0 ]; then
        say "OK      removed $n scratch build dir(s) under .context/working, about ${freed} GiB"
    elif [ "$n" = 0 ] && [ "$QUIET" != 1 ]; then
        say "nothing to do: no scratch build dir under .context/working untouched for ${SCRATCH_DAYS}+ days"
    fi
}
scratch_trim

DEBUG="$ROOT/target/debug" MUT="$ROOT/target/arc011-mutants"
[ -d "$ROOT/target" ] || { [ "$QUIET" = 1 ] || say "nothing to do: no $ROOT/target"; exit 0; }
[ -x "$CARGO" ] || { echo "target-cache-trim: cargo not found ($CARGO)" >&2; exit 2; }

before="$(gib "$DEBUG")"
case "$before" in ''|*[!0-9]*) echo "target-cache-trim: cannot size $DEBUG" >&2; exit 2 ;; esac
if [ "$before" -lt "$MIN_GB" ]; then
    [ "$QUIET" = 1 ] || say "nothing to do: target/debug ${before} GiB < ${MIN_GB} GiB"
    exit 0
fi

busy="$(pgrep -af "$BUSY_PATTERN" 2>/dev/null | head -1)"
if [ -z "$busy" ] && [ -e "$DEBUG/.cargo-lock" ] && command -v flock >/dev/null 2>&1 \
   && ! flock -n "$DEBUG/.cargo-lock" true 2>/dev/null; then
    busy="cargo build lock held: $DEBUG/.cargo-lock"
fi
if [ -n "$busy" ]; then
    say "skip    target/debug ${before} GiB, a build is running (${busy:0:120}); next week"
    exit 0
fi

mut="$(gib "$MUT")"
if [ "$DRY" = 1 ]; then
    say "[DRY]   would run cargo clean --profile dev (target/debug ${before} GiB) and clean target/arc011-mutants (${mut} GiB)"
    exit 0
fi

(cd "$ROOT" && env -u PROJECT_ROOT "$CARGO" clean --profile dev) > /dev/null 2>&1
[ -d "$MUT" ] && (cd "$ROOT" && env -u PROJECT_ROOT "$CARGO" clean --target-dir "$MUT") > /dev/null 2>&1

after="$(gib "$DEBUG")"
if [ "${after:-99}" -ge 1 ]; then
    say "FAILED  cargo clean --profile dev ran, but target/debug is still ${after} GiB (was ${before})"
    exit 1
fi
say "OK      trimmed target/debug ${before} GiB -> ${after} GiB, arc011-mutants ${mut} GiB -> $(gib "$MUT") GiB; target now $(gib "$ROOT/target") GiB"
exit 0
