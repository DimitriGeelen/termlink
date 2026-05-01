#!/usr/bin/env bash
# =============================================================================
# T-1442 / U-005 — regression test for tl-dispatch.sh meta.json schema.
# =============================================================================
# Pins:
#   1. The meta.json template emitted by cmd_spawn includes task_type, model,
#      model_used, fallback_used keys (schema parity with the framework's
#      agents/termlink/termlink.sh per T-1643/W4).
#   2. _resolve_dispatch_model produces the four documented (model_used,
#      fallback_used) tuples across explicit / per-type / default / none branches.
#   3. cmd_spawn writes meta.json with substrate-resolved model_used and
#      fallback_used populated (string / bool, or JSON null when no model
#      resolves) — closing the value loop the framework's null-writer leaves open.
#
# We do NOT spawn real termlink sessions or claude. We source tl-dispatch.sh,
# stub `termlink` and `claude`, and exercise the meta-write path directly.
#
# Usage: bash tests/test_tl_dispatch_meta.sh
# =============================================================================

set -uo pipefail
# We deliberately don't use `set -e` because cmd_spawn sources tl-dispatch.sh's
# own `set -e` and contains pipelines like `termlink list | grep $name | awk`
# that legitimately exit non-zero in the stubbed environment (no real hub →
# empty list → grep returns 1). The test asserts on outcomes, not exit codes.
set +o pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/tl-dispatch.sh"

PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); echo "  PASS: $*"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL: $*"; }

assert_eq() {
    local actual="$1" expected="$2" label="$3"
    if [ "$actual" = "$expected" ]; then
        ok "$label (got: $actual)"
    else
        fail "$label — expected: $expected, got: $actual"
    fi
}

# -----------------------------------------------------------------------------
# Stub harness: source the helpers without invoking the case dispatcher.
# tl-dispatch.sh ends with a `case "${1:-}" in ... esac` driven by argv.
# Sourcing it with no args triggers the "" branch (prints usage and exits 0),
# but only AFTER ensure_termlink / ensure_claude run. We stub those before
# sourcing so missing binaries don't fail the test.
# -----------------------------------------------------------------------------

ensure_termlink() { :; }
ensure_claude()   { :; }
export -f ensure_termlink ensure_claude

# Sourcing the script will print usage to stdout and call ensure_*. Suppress
# stdout. We only care about the function definitions becoming available.
# The script uses `set -e` at the top — we tolerate that by sourcing in a
# subshell-ish guard.
set +e
# shellcheck disable=SC1090
source "$SCRIPT" >/dev/null 2>&1 || true
set -e

# -----------------------------------------------------------------------------
# Pin 1: schema template static check.
# -----------------------------------------------------------------------------
echo "--- Pin 1: meta.json template includes orchestrator-substrate keys ---"
SRC=$(cat "$SCRIPT")
case "$SRC" in
    *'"task_type"'*)     ok "meta.json template has task_type field" ;;
    *)                   fail "meta.json template missing task_type" ;;
esac
case "$SRC" in
    *'"model_used"'*)    ok "meta.json template has model_used field" ;;
    *)                   fail "meta.json template missing model_used" ;;
esac
case "$SRC" in
    *'"fallback_used"'*) ok "meta.json template has fallback_used field" ;;
    *)                   fail "meta.json template missing fallback_used" ;;
esac

# -----------------------------------------------------------------------------
# Pin 2: _resolve_dispatch_model resolution branches.
# -----------------------------------------------------------------------------
echo "--- Pin 2: _resolve_dispatch_model branches ---"

# Clean env to ensure deterministic behavior.
unset DISPATCH_MODEL_DEFAULT DISPATCH_MODEL_FOR_BUILD DISPATCH_MODEL_FOR_TEST 2>/dev/null || true

# Branch 1: explicit --model wins.
out=$(_resolve_dispatch_model "haiku" "build")
assert_eq "$out" "haiku|false" "explicit --model wins, no fallback"

# Branch 2: per-type override.
DISPATCH_MODEL_FOR_BUILD="sonnet" out=$(DISPATCH_MODEL_FOR_BUILD="sonnet" _resolve_dispatch_model "" "build")
assert_eq "$out" "sonnet|false" "DISPATCH_MODEL_FOR_BUILD honored, no fallback"

# Branch 3: default → fallback_used=true.
DISPATCH_MODEL_DEFAULT="opus" out=$(DISPATCH_MODEL_DEFAULT="opus" _resolve_dispatch_model "" "inception")
assert_eq "$out" "opus|true" "DISPATCH_MODEL_DEFAULT used, fallback_used=true"

# Branch 4: nothing resolves → empty model and empty flag.
unset DISPATCH_MODEL_DEFAULT DISPATCH_MODEL_FOR_BUILD 2>/dev/null || true
out=$(_resolve_dispatch_model "" "build")
assert_eq "$out" "|" "no config → empty model, empty fallback flag"

# Mixed: per-type beats default.
out=$(DISPATCH_MODEL_FOR_BUILD="haiku" DISPATCH_MODEL_DEFAULT="opus" _resolve_dispatch_model "" "build")
assert_eq "$out" "haiku|false" "per-type beats default"

# Explicit beats both.
out=$(DISPATCH_MODEL_FOR_BUILD="haiku" DISPATCH_MODEL_DEFAULT="opus" _resolve_dispatch_model "sonnet" "build")
assert_eq "$out" "sonnet|false" "explicit beats per-type and default"

# -----------------------------------------------------------------------------
# Pin 3: end-to-end cmd_spawn meta.json content.
# -----------------------------------------------------------------------------
echo "--- Pin 3: cmd_spawn writes populated meta.json ---"

# Stub termlink so cmd_spawn doesn't hit a real hub. Stub fields:
#   spawn   — must succeed (exit 0)
#   list    — empty (no PID recorded, fine)
#   pty     — succeed silently
termlink() {
    case "$1" in
        spawn|pty) return 0 ;;
        list)      return 0 ;;
        event)     return 0 ;;
        *)         return 0 ;;
    esac
}
export -f termlink

WORK_BASE=$(mktemp -d)
trap 'rm -rf "$WORK_BASE"' EXIT
DISPATCH_DIR="$WORK_BASE/tl-dispatch"

# Case A: explicit --model haiku, --task-type build → ("haiku", false).
unset DISPATCH_MODEL_DEFAULT DISPATCH_MODEL_FOR_BUILD 2>/dev/null || true
cmd_spawn --name caseA --prompt "x" --project "$WORK_BASE" --model haiku --task-type build >/dev/null
meta="$DISPATCH_DIR/caseA/meta.json"
[ -f "$meta" ] && ok "caseA meta.json exists" || fail "caseA meta.json missing"
mu=$(python3 -c "import json,sys; print(json.load(open('$meta'))['model_used'])")
fu=$(python3 -c "import json,sys; print(json.load(open('$meta'))['fallback_used'])")
tt=$(python3 -c "import json,sys; print(json.load(open('$meta'))['task_type'])")
assert_eq "$mu" "haiku" "caseA model_used"
assert_eq "$fu" "False" "caseA fallback_used (Python bool repr)"
assert_eq "$tt" "build" "caseA task_type"

# Verify model_used is a JSON string and fallback_used is a JSON bool (not strings).
type_mu=$(python3 -c "import json; v=json.load(open('$meta'))['model_used']; print(type(v).__name__)")
type_fu=$(python3 -c "import json; v=json.load(open('$meta'))['fallback_used']; print(type(v).__name__)")
assert_eq "$type_mu" "str" "caseA model_used is a string"
assert_eq "$type_fu" "bool" "caseA fallback_used is a bool"

# Case B: no --model, per-type set → ("sonnet", false).
export DISPATCH_MODEL_FOR_BUILD="sonnet"
unset DISPATCH_MODEL_DEFAULT 2>/dev/null || true
cmd_spawn --name caseB --prompt "x" --project "$WORK_BASE" --task-type build >/dev/null
meta="$DISPATCH_DIR/caseB/meta.json"
mu=$(python3 -c "import json; print(json.load(open('$meta'))['model_used'])")
fu=$(python3 -c "import json; print(json.load(open('$meta'))['fallback_used'])")
assert_eq "$mu" "sonnet" "caseB model_used (per-type)"
assert_eq "$fu" "False" "caseB fallback_used (per-type, no fallback)"

# Case C: no --model, no per-type, only default → ("opus", true).
unset DISPATCH_MODEL_FOR_BUILD
export DISPATCH_MODEL_DEFAULT="opus"
cmd_spawn --name caseC --prompt "x" --project "$WORK_BASE" --task-type build >/dev/null
meta="$DISPATCH_DIR/caseC/meta.json"
mu=$(python3 -c "import json; print(json.load(open('$meta'))['model_used'])")
fu=$(python3 -c "import json; print(json.load(open('$meta'))['fallback_used'])")
assert_eq "$mu" "opus" "caseC model_used (default)"
assert_eq "$fu" "True" "caseC fallback_used (default → fallback)"

# Case D: nothing resolves → JSON null, null.
unset DISPATCH_MODEL_DEFAULT DISPATCH_MODEL_FOR_BUILD 2>/dev/null || true
cmd_spawn --name caseD --prompt "x" --project "$WORK_BASE" >/dev/null
meta="$DISPATCH_DIR/caseD/meta.json"
mu=$(python3 -c "import json; v=json.load(open('$meta'))['model_used']; print('NULL' if v is None else repr(v))")
fu=$(python3 -c "import json; v=json.load(open('$meta'))['fallback_used']; print('NULL' if v is None else repr(v))")
assert_eq "$mu" "NULL" "caseD model_used is JSON null"
assert_eq "$fu" "NULL" "caseD fallback_used is JSON null"

# -----------------------------------------------------------------------------
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ]
