#!/bin/bash
# Fixtures for scripts/check-decisions-yaml-format.sh (T-3317).
# Hermetic: every case is a scratch file. Load-bearing case: the real pre-fix
# decisions.yaml extracted from git MUST fire, and a real capture through the
# vendored writer into the fixed form MUST stay clean.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHK="$ROOT/scripts/check-decisions-yaml-format.sh"
S="$(mktemp -d)"; trap 'rm -rf "$S"' EXIT
pass=0; fail=0
expect() {  # $1 name, $2 expected rc, $3 file
    bash "$CHK" --file "$3" > "$S/out" 2>&1; local rc=$?
    if [ "$rc" -eq "$2" ]; then pass=$((pass+1)); echo "  ok   $1 (rc=$rc)"
    else fail=$((fail+1)); echo "  FAIL $1 (rc=$rc, want $2)"; sed 's/^/       /' "$S/out" | head -4; fi
}

printf 'decisions:\n  - id: PD-001\n    decision: "a"\n  - id: PD-002\n    decision: "b"\n' > "$S/good.yaml"
expect "2-space list is clean" 0 "$S/good.yaml"

printf 'decisions:\n- id: PD-001\n  decision: "a"\n' > "$S/col0.yaml"
expect "column-0 list fires (writer would restart at PD-001)" 1 "$S/col0.yaml"

printf 'decisions:\n- id: PD-180\n  decision: "a"\n\n  - id: PD-001\n    decision: "b"\n' > "$S/mixed.yaml"
expect "mixed indent (the observed corruption) fires" 1 "$S/mixed.yaml"

printf 'decisions: [unclosed\n' > "$S/bad.yaml"
expect "unparseable file fires" 1 "$S/bad.yaml"

printf 'decisions: {}\n' > "$S/notlist.yaml"
expect "non-list decisions fires" 1 "$S/notlist.yaml"

expect "absent file is tooling (2), never clean" 2 "$S/does-not-exist.yaml"

# Load-bearing: a real capture through the vendored writer keeps the fixed form clean.
mkdir -p "$S/proj/.context/project"; cp "$S/good.yaml" "$S/proj/.context/project/decisions.yaml"
PROJECT_ROOT="$S/proj" CONTEXT_DIR="$S/proj/.context" \
    "$ROOT/.agentic-framework/agents/context/context.sh" add-decision "fixture capture" --task T-3317 --rationale fixture > /dev/null 2>&1
expect "real writer capture into 2-space form stays clean" 0 "$S/proj/.context/project/decisions.yaml"
grep -q '^  - id: PD-003' "$S/proj/.context/project/decisions.yaml" \
    && { pass=$((pass+1)); echo "  ok   real writer continues the sequence (PD-003)"; } \
    || { fail=$((fail+1)); echo "  FAIL real writer continues the sequence (PD-003)"; }

# Load-bearing: the same capture into a column-0 file reproduces the defect.
cp "$S/col0.yaml" "$S/proj/.context/project/decisions.yaml"
PROJECT_ROOT="$S/proj" CONTEXT_DIR="$S/proj/.context" \
    "$ROOT/.agentic-framework/agents/context/context.sh" add-decision "fixture capture" --task T-3317 --rationale fixture > /dev/null 2>&1
expect "real writer capture into column-0 form fires (defect reproduced)" 1 "$S/proj/.context/project/decisions.yaml"

# Load-bearing: the real pre-fix file from git fires.
if git -C "$ROOT" show "${T3317_PRE_FIX_COMMIT:-f64d6d5c9}:.context/project/decisions.yaml" > "$S/prefix.yaml" 2>/dev/null; then
    expect "real pre-fix decisions.yaml (f64d6d5c9) fires" 1 "$S/prefix.yaml"
else
    fail=$((fail+1)); echo "  FAIL could not extract the pre-fix decisions.yaml"
fi

echo; echo "passed=$pass failed=$fail"
[ "$fail" -eq 0 ] && { echo "ALL PASS"; exit 0; }
exit 1
