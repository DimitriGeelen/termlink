#!/usr/bin/env bash
# pool-run.sh — run one TermLink job on the ring20 build pool, with ring20's code pinned (T-3858, T-3859)
#
# WHY: T-3858 sends our test / clippy / per-crate / build jobs to the ring20 pool through ring20's submit script and
# pool client. That code runs here as root under our hub identity (risk matrix docs/reports/T-3858-risk-matrix.md,
# risk 1). This entry point refuses to run unless the submit script, pool-client.py and poold.py match the reviewed
# sha256 in scripts/pool-run.pins. A new version is read in full before its pin changes, in a commit.
#
# ADVISORY (T-3858 risk 3): a pool result is advice. Task closure and releases keep their local runs; nothing may
# treat a pool pass as a gate. Every run writes ADVISORY.txt next to its results.
#
# Usage: pool-run.sh {test|clippy|crates|build} [COMMIT]   (COMMIT defaults to HEAD of the repo)
# Output: <out-root>/<UTC>-<kind>-<commit12>/  (default out-root .context/working/pool-runs, gitignored)
# Exit: 0 every job passed · 1 a job failed · 2 could not run or could not look (pin mismatch included; never 0)
# Seams (fixtures only): POOL_RUN_PINS, POOL_RUN_REPO, POOL_RUN_OUT_ROOT, POOL_RUN_HEARTBEAT, POOL_RUN_PYTHON
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PINS="${POOL_RUN_PINS:-$ROOT/scripts/pool-run.pins}"
REPO="${POOL_RUN_REPO:-$ROOT}"
OUT_ROOT="${POOL_RUN_OUT_ROOT:-$ROOT/.context/working/pool-runs}"
HBF="${POOL_RUN_HEARTBEAT:-$ROOT/.context/working/.pool-run.heartbeat}"
PY="${POOL_RUN_PYTHON:-python3}"
EXCLUDES=(.claude/settings.local.json .context .tasks .fabric .agentic-framework)

die() { echo "pool-run: $*" >&2; exit 2; }
kind="${1:-}"; case "$kind" in test|clippy|crates|build) ;; *) die "usage: pool-run.sh {test|clippy|crates|build} [COMMIT]";; esac
commit="$(git -C "$REPO" rev-parse --verify "${2:-HEAD}^{commit}" 2>/dev/null)" || die "unknown commit: ${2:-HEAD}"

# --- pins: every line must match, or nothing runs ---
[ -r "$PINS" ] || die "pin file unreadable: $PINS"
submit="" client="" n=0
while read -r sum path; do
    case "$sum" in ''|'#'*) continue ;; esac
    [ -r "$path" ] || die "PIN: $path missing or unreadable"
    got="$(sha256sum "$path" | cut -c1-64)"
    [ "$got" = "$sum" ] || die "PIN MISMATCH: $path is $got, pinned $sum (read the new version, then re-pin)"
    case "$path" in */010-pool-submit.py) submit="$path" ;; */pool-client.py) client="$path" ;; esac
    n=$((n+1))
done < "$PINS"
[ "$n" -ge 3 ] && [ -n "$submit" ] && [ -n "$client" ] || die "pin file must pin the submit script, pool-client.py and poold.py"

# Run PRIVATE copies of all three, each re-checked after copying (T-3859, 2026-11-24): the submit script calls the pool
# client minutes into a run, so a pin checked only at start let a client replaced mid-run execute unreviewed. The client
# imports poold.py from its own directory, so both live side by side in the private dir.
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/client"
while read -r sum path; do
    case "$sum" in ''|'#'*) continue ;; esac
    case "$path" in */010-pool-submit.py) dst="$tmp/submit.py" ;; *) dst="$tmp/client/$(basename "$path")" ;; esac
    cp "$path" "$dst" || die "cannot copy $path"
    [ "$(sha256sum "$dst" | cut -c1-64)" = "$sum" ] || die "PIN MISMATCH after copy: $path"
done < "$PINS"
client="$tmp/client/$(basename "$client")"

out="$OUT_ROOT/$(date -u +%Y%m%dT%H%M%SZ)-$kind-${commit:0:12}"
mkdir -p "$out" || die "cannot create $out"
printf 'ADVISORY: pool result, not a gate (T-3858 risk 3, T-3859). Task closure and releases keep their local runs.\ncommit %s, kind %s\n' "$commit" "$kind" > "$out/ADVISORY.txt"
args=()
for e in "${EXCLUDES[@]}"; do args+=(--exclude "$e"); done
POOL_CLIENT="$client" "$PY" -I "$tmp/submit.py" "$kind" "$commit" --repo "$REPO" --out "$out" "${args[@]}" --timeout 5400
rc=$?
case "$rc" in 0|1|2) ;; *) echo "pool-run: submit script exited $rc, reported as could-not-run" >&2; rc=2 ;; esac
echo "pool-run: ADVISORY result rc=$rc ($kind, ${commit:0:12}, $out)"
touch "$HBF" 2>/dev/null
exit "$rc"
