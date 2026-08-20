#!/usr/bin/env bash
# T-2810 — fixtures for the G-053 revisit-due cron invocation.
#
# What these pin is the OUTCOME, not the installation. T-1452 shipped this
# mechanism with a correct, ticked acceptance criterion — "the crontab contains
# the line (verified live)" — and the line did nothing for months, because
# containing a line and finding ripe revisits are different properties. These
# fixtures assert the second one.
#
# The defect: revisit-due-scan.sh resolves PROJECT_ROOT by walking up from its
# own directory for `.framework.yaml` OR `FRAMEWORK.md`. A vendored framework
# ships its own FRAMEWORK.md, so the walk stops at `.agentic-framework/`,
# `.tasks/active` is absent there, and the script exits 0 — silence that reads
# exactly like "nothing due". The `cd` in the cron line cannot help: the walk
# starts at ${BASH_SOURCE[0]}.
#
# Run: bash tests/revisit-due-cron-fixtures.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCAN="$REPO_ROOT/.agentic-framework/agents/context/revisit-due-scan.sh"
REGISTRY="$REPO_ROOT/.context/cron-registry.yaml"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }

echo "== revisit-due cron fixtures =="

if [ ! -f "$SCAN" ]; then
    echo "  SKIP  revisit-due-scan.sh not present (vendored framework absent)" >&2
    exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- Build a fixture project that reproduces the real shape -----------------
# Both markers present: the project's own `.framework.yaml`, AND a vendored
# framework carrying `FRAMEWORK.md`. That collision is the whole defect.
PROJ="$TMP/proj"
mkdir -p "$PROJ/.tasks/active" "$PROJ/.context/working" \
         "$PROJ/.agentic-framework/agents/context"
printf 'project: fixture\n' > "$PROJ/.framework.yaml"
printf '# vendored framework marker\n' > "$PROJ/.agentic-framework/FRAMEWORK.md"
cp "$SCAN" "$PROJ/.agentic-framework/agents/context/revisit-due-scan.sh"
chmod +x "$PROJ/.agentic-framework/agents/context/revisit-due-scan.sh"

mk_task() {  # mk_task <id> <revisit_at> <name>
    cat > "$PROJ/.tasks/active/$1-fixture.md" <<EOF
---
id: $1
name: "$3"
status: captured
workflow_type: inception
revisit_at: $2
---
body
EOF
}
PAST=$(date -u -d '30 days ago' +%Y-%m-%d 2>/dev/null || echo 2020-01-01)
FUTURE=$(date -u -d '30 days' +%Y-%m-%d 2>/dev/null || echo 2999-01-01)
mk_task T-RIPE "$PAST" "ripe deferral"
mk_task T-FUTURE "$FUTURE" "not yet due"

SCAN_FIX="$PROJ/.agentic-framework/agents/context/revisit-due-scan.sh"
OUT="$PROJ/.context/working/.revisits-due.txt"

# --- 1. The BUG: no PROJECT_ROOT, invoked the way the old cron line did ------
# `cd` into the project first, exactly as the generated cron line does. This
# must demonstrate the failure, otherwise the fix below proves nothing.
rm -f "$OUT"
( cd "$PROJ" && env -u PROJECT_ROOT bash "$SCAN_FIX" ) >/dev/null 2>&1
rc=$?
if [ "$rc" -eq 0 ] && [ ! -f "$OUT" ]; then
    ok "reproduces the defect: without PROJECT_ROOT the scan exits 0 and finds nothing"
else
    bad "expected the unset-PROJECT_ROOT case to exit 0 with no output file" "rc=$rc, out exists: $([ -f "$OUT" ] && echo yes || echo no)"
fi

# --- 2. The FIX: PROJECT_ROOT set, as the registry now does ------------------
rm -f "$OUT"
( cd "$PROJ" && PROJECT_ROOT="$(pwd)" bash "$SCAN_FIX" ) >/dev/null 2>&1
if [ -f "$OUT" ]; then
    ok "with PROJECT_ROOT set, the scan produces its output file"
else
    bad "with PROJECT_ROOT set, expected an output file" "none at $OUT"
fi

if [ -f "$OUT" ] && grep -q 'T-RIPE' "$OUT"; then
    ok "the ripe deferral is named"
else
    bad "the ripe deferral should be named" "$( [ -f "$OUT" ] && cat "$OUT" )"
fi

if [ -f "$OUT" ] && ! grep -q 'T-FUTURE' "$OUT"; then
    ok "a future revisit_at is not reported"
else
    bad "a future revisit_at must not be reported" "$( [ -f "$OUT" ] && cat "$OUT" )"
fi

# --- 3. `$(pwd)` is what the registry uses — prove that form works -----------
# The registry sets PROJECT_ROOT="$(pwd)" rather than a hardcoded path, relying
# on the generated line's `cd "<project_root>" &&` prefix. Exercise that exact
# shape through sh, the shell cron uses.
rm -f "$OUT"
sh -c "cd '$PROJ' && PROJECT_ROOT=\"\$(pwd)\" bash '$SCAN_FIX'" >/dev/null 2>&1
if [ -f "$OUT" ] && grep -q 'T-RIPE' "$OUT"; then
    ok "the registry's \$(pwd) form works under sh, as cron invokes it"
else
    bad "the \$(pwd) form should resolve under sh" "$( [ -f "$OUT" ] && cat "$OUT" )"
fi

# --- 4. The registry entry actually carries it ------------------------------
if [ -f "$REGISTRY" ]; then
    blk=$(awk '/^  - id: revisit-due-scan$/,/^  - id: [^r]/' "$REGISTRY")
    case "$blk" in
        *'PROJECT_ROOT='*) ok "cron-registry revisit-due-scan entry sets PROJECT_ROOT" ;;
        *) bad "cron-registry entry must set PROJECT_ROOT" "block: $(printf '%s' "$blk" | head -c 200)" ;;
    esac
else
    bad "cron-registry.yaml not found" "$REGISTRY"
fi

echo
echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
