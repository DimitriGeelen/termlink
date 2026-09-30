#!/usr/bin/env bash
# T-3282 — full-corpus equivalence PROOF for scripts/lib/verification-block-batch.py.
#
# For every task file under .tasks/{active,completed} (or --tasks-dir), compares the
# framework's REAL `extract_verification_block` (lib/verification-port.sh) with the
# batch extractor, as both are consumed by the check (command-substitution form,
# trailing newlines stripped). Prints the file count and any mismatch; exit 0 only
# when every file is identical.
#
# SLOW BY DESIGN: it runs the real per-file pipeline ~3000 times, which is the cost
# the batch extractor exists to remove. It is deliberately NOT named *fixtures*.sh, so
# the guard layer does not run it on every push. Run it after touching the batch
# extractor, the framework's verification-port.sh, or comment_strip.py. The fast,
# always-on guard is the check's own run-time sample cross-check (plus the
# hermetic cases in verification-heading-shadow-fixtures.sh).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TASKS_DIR="$ROOT/.tasks"
[ "${1:-}" = "--tasks-dir" ] && TASKS_DIR="$2"
FW="${FRAMEWORK_ROOT:-$ROOT/.agentic-framework}"
export FRAMEWORK_ROOT="$FW"
# shellcheck disable=SC1090
source "$FW/lib/verification-port.sh" || { echo "proof: cannot source verification-port.sh"; exit 2; }
declare -F extract_verification_block >/dev/null || { echo "proof: extractor missing"; exit 2; }

shopt -s nullglob
FILES=("$TASKS_DIR"/active/*.md "$TASKS_DIR"/completed/*.md)
shopt -u nullglob
[ "${#FILES[@]}" -gt 0 ] || { echo "proof: no task files under $TASKS_DIR"; exit 2; }

OUT="$(mktemp -d)"; trap 'rm -rf "$OUT"' EXIT
printf '%s\0' "${FILES[@]}" | python3 "$ROOT/scripts/lib/verification-block-batch.py" extract "$FW" "$OUT" \
  || { echo "proof: batch extractor failed"; exit 2; }

n=0; mism=0; nonempty=0
for i in "${!FILES[@]}"; do
  f="${FILES[$i]}"
  real="$(extract_verification_block "$f" 2>/dev/null)"
  if [ -f "$OUT/$i" ]; then batch="$(cat "$OUT/$i")"; else batch=""; fi
  n=$((n+1)); [ -n "$real" ] && nonempty=$((nonempty+1))
  if [ "$real" != "$batch" ]; then
    mism=$((mism+1))
    echo "MISMATCH: ${f#"$ROOT"/}"
    diff <(printf '%s\n' "$real") <(printf '%s\n' "$batch") | head -8 | sed 's/^/    /'
  fi
done
echo "verification-block-batch proof: $n file(s) compared, $nonempty non-empty, $mism mismatch(es)"
[ "$mism" = "0" ]
