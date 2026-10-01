#!/bin/bash
# guard-layer: source
#
# check-decisions-yaml-format.sh (T-3317) — is .context/project/decisions.yaml in the
# form the framework's decision writer expects?
#
# The vendored writer (agents/context/lib/decision.sh) finds the next id with
# `grep "^  - id: PD-"` and appends a 2-space-indented `  - id:` entry. If the file's
# list is at column 0 (`- id:`, the default shape when PyYAML re-serialises a file),
# the grep finds nothing: every capture restarts at PD-001, and the appended entry
# is indented differently from its siblings, so the file stops parsing. That broke
# four task closes in one day (T-3150 class) and blocked the pre-push audit each time.
# The audit notices AFTER the damage; this check names the drift BEFORE a capture.
#
# Fires (exit 1) on: any column-0 `- id:` entry; entries at both indents; a file that
# does not parse; a `decisions` key that is not a list. Exit 0 = writer-compatible.
# Exit 2 = tooling (file absent, python3/PyYAML missing) — never a false clean.
#
# Usage: bash scripts/check-decisions-yaml-format.sh [--file PATH] [--quiet]
# Seam:  DECISIONS_YAML_FILE=<path>
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILE="${DECISIONS_YAML_FILE:-$ROOT/.context/project/decisions.yaml}"
QUIET=0
while [ $# -gt 0 ]; do
    case "$1" in
        --file) FILE="$2"; shift 2 ;;
        --quiet) QUIET=1; shift ;;
        -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
        *) echo "check-decisions-yaml-format: unknown arg $1" >&2; exit 2 ;;
    esac
done

[ -f "$FILE" ] || { echo "check-decisions-yaml-format: $FILE not found" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "check-decisions-yaml-format: python3 missing" >&2; exit 2; }

python3 - "$FILE" "$QUIET" <<'EOF'
import re, sys
path, quiet = sys.argv[1], sys.argv[2] == "1"
try:
    import yaml
except ImportError:
    print("check-decisions-yaml-format: PyYAML missing", file=sys.stderr); sys.exit(2)
text = open(path).read()
col0 = [i + 1 for i, l in enumerate(text.split("\n")) if re.match(r"^- id:", l)]
ind2 = [i + 1 for i, l in enumerate(text.split("\n")) if re.match(r"^  - id:", l)]
problems = []
try:
    d = yaml.safe_load(text)
    if not isinstance(d, dict) or not isinstance(d.get("decisions"), list):
        problems.append("top-level `decisions:` is not a list")
except yaml.YAMLError as e:
    problems.append("does not parse: " + str(e).split("\n")[0])
if col0 and ind2:
    problems.append(f"mixed indent: {len(col0)} column-0 entries and {len(ind2)} 2-space entries "
                    f"(first 2-space at line {ind2[0]}) — a capture landed in a column-0 file")
elif col0:
    problems.append(f"{len(col0)} column-0 `- id:` entries (first at line {col0[0]}); the writer "
                    "greps `^  - id:` and will restart at PD-001 and break the file on the next capture")
if problems:
    print(f"check-decisions-yaml-format: FIRING — {path}")
    for p in problems:
        print("  - " + p)
    print("  fix: re-indent the list under `decisions:` by two spaces (parsed content is unchanged), "
          "and renumber any PD-001..N tail captured after the highest existing id (T-3317)")
    sys.exit(1)
if not quiet:
    print(f"check-decisions-yaml-format: OK — {len(ind2)} entries in the writer's 2-space form")
sys.exit(0)
EOF
