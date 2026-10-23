#!/usr/bin/env bash
# guard-layer: source warn  # T-3544: a stale slash-command card misleads impact analysis but breaks nothing
# T-3544 — every .claude/commands/*.md has a fabric card whose depends_on/interfaces match what the file
# runs now. Wraps `scripts/fabric-command-cards.py --check`.
# Exit: 0 clean · 1 a command has no card or its card is stale (fix: python3 scripts/fabric-command-cards.py)
#       · 2 tooling.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v python3 >/dev/null 2>&1 || { echo "check-command-cards: python3 not found" >&2; exit 2; }
python3 "$ROOT/scripts/fabric-command-cards.py" --check --root "$ROOT"
rc=$?
[ "$rc" = 1 ] && echo "  fix: python3 scripts/fabric-command-cards.py   (then commit the refreshed cards)"
exit "$rc"
