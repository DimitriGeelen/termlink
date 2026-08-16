#!/usr/bin/env bash
# verification-pipefail-fixtures.sh (T-2775)
#
# Hermetic load-bearing proof for scripts/check-verification-pipefail.sh — no repo task
# state, no gate execution. Builds a scratch .tasks dir of synthetic task files exercising
# each branch of the detector and asserts the verdict + exit code on each.
#
# The suite also PINS THE MEASURED IDIOM SAFETY that decides the detection rule, by
# actually running each idiom under the gate's own shell settings. That matters: the
# remediation two peer projects published (`printf '%s' "$out" | grep -q PAT`) is safe
# only while the captured value fits the pipe buffer, and a fixture that merely asserted
# the detector's opinion would happily encode a wrong opinion.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-verification-pipefail.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
TASKS="$SCRATCH/tasks"; mkdir -p "$TASKS"
ALLOW="$SCRATCH/allowlist"; : > "$ALLOW"

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
assert_contains() { # <desc> <haystack> <needle>
    case "$2" in *"$3"*) pass=$((pass+1)); echo "  ok: $1";;
    *) fail=$((fail+1)); echo "  FAIL: $1 — missing '$3'" >&2;; esac
}

run()      { bash "$CHECK" --tasks-dir "$TASKS" --allowlist "$ALLOW" "$@" >/dev/null 2>&1; echo $?; }
run_json() { bash "$CHECK" --tasks-dir "$TASKS" --allowlist "$ALLOW" --json 2>/dev/null; }

mktask() { # <name> <verification-body>
    rm -f "$TASKS"/*.md
    cat > "$TASKS/$1" <<EOF
---
id: T-9999
---

# fixture

## Acceptance Criteria
- [x] something

## Verification
$2
EOF
}

echo "=== A. the firing class: an early-exiting consumer can SIGPIPE the producer ==="

mktask "T-9001-grepq.md" 'cargo test 2>&1 | grep -q "test result: ok"'
assert_rc "bare | grep -q fires" 1 "$(run)"

mktask "T-9002-head.md" 'termlink fleet doctor --json | head -1'
assert_rc "| head fires" 1 "$(run)"

mktask "T-9003-python.md" 'termlink channel list --json | python3 -c "import sys,json; json.load(sys.stdin)"'
assert_rc "| python3 fires" 1 "$(run)"

mktask "T-9004-jq.md" 'termlink hub status --json | jq -e .ok'
assert_rc "| jq fires" 1 "$(run)"

mktask "T-9005-grepq-flags.md" 'some-cmd 2>&1 | grep -sq PATTERN'
assert_rc "clustered grep flags (-sq) still fire" 1 "$(run)"

echo
echo "=== B. cleared: idioms measured safe at any output size ==="

mktask "T-9010-substitution.md" 'test -n "$(termlink hub status --json | grep -m1 running)"'
assert_rc "PL-080 idiom (pipeline inside \$( )) does not fire" 0 "$(run)"

mktask "T-9011-herestring.md" 'out=$(cargo test 2>&1 || true); grep -q "test result: ok" <<< "$out"'
assert_rc "herestring does not fire" 0 "$(run)"

mktask "T-9012-nopipe.md" 'grep -q "pub fn decide_unix_peer" crates/termlink-session/src/auth.rs'
assert_rc "grep -q with a file argument (no pipe) does not fire" 0 "$(run)"

mktask "T-9013-plain.md" 'cargo test --workspace'
assert_rc "a plain command does not fire" 0 "$(run)"

mktask "T-9014-backtick.md" 'test -n "`termlink topics | grep -m1 x`"'
assert_rc "backtick substitution does not fire" 0 "$(run)"

echo
echo "=== C. the bounded-producer case gets its own reason, not silence ==="

mktask "T-9020-printf.md" 'out=$(cargo test 2>&1 || true); printf "%s" "$out" | grep -q "ok"'
assert_rc "printf producer still fires (size-dependent)" 1 "$(run)"
assert_contains "printf case is reported as bounded-producer-pipeline, not conflated" \
    "$(run_json)" '"kind": "bounded-producer-pipeline"'

echo
echo "=== D. parsing: only the ## Verification block, only executable lines ==="

mktask "T-9030-comment.md" '# cargo test 2>&1 | grep -q "ok"'
assert_rc "a commented-out risky line does not fire" 0 "$(run)"

rm -f "$TASKS"/*.md
cat > "$TASKS/T-9031-otherblock.md" <<'EOF'
---
id: T-9031
---
## Context
Prose describing `cargo test | grep -q ok` as the thing we must avoid.

## Verification
cargo test --workspace
EOF
assert_rc "a risky shape quoted OUTSIDE ## Verification does not fire" 0 "$(run)"

echo
echo "=== E. the allowlist: acknowledges, and cannot outlive its command ==="

mktask "T-9040-ack.md" 'cargo test 2>&1 | grep -q "test result: ok"'
sig="$(run_json | python3 -c 'import sys,json; print(json.load(sys.stdin)["firing"][0]["signature"])')"
printf '%s  # acknowledged by fixture\n' "$sig" > "$ALLOW"
assert_rc "an allowlisted line does not fire" 0 "$(run)"
assert_contains "an allowlisted line is still COUNTED, not hidden" \
    "$(run_json)" '"acknowledged": 1'

# Reword the command: the signature must change, so the stale acknowledgement lapses.
mktask "T-9040-ack.md" 'cargo test 2>&1 | grep -q "test result: OK"'
assert_rc "rewording an acknowledged command re-fires it" 1 "$(run)"
: > "$ALLOW"

echo
echo "=== F. contract: exit codes, JSON shape, scope disclaimer ==="

mktask "T-9050-clean.md" 'cargo test --workspace'
assert_contains "clean JSON reports ok:true" "$(run_json)" '"ok": true'
assert_contains "clean output states the scope limit" \
    "$(bash "$CHECK" --tasks-dir "$TASKS" --allowlist "$ALLOW" 2>&1)" "Scope:"

bash "$CHECK" --tasks-dir "$SCRATCH/does-not-exist" >/dev/null 2>&1
assert_rc "a missing tasks dir is a tooling error, not 'clean'" 2 $?

bash "$CHECK" --bogus-flag >/dev/null 2>&1
assert_rc "an unknown flag is a tooling error" 2 $?

echo
echo "=== G. measured idiom safety (the ground truth the rule rests on) ==="
# Run each idiom under the gate's own construct: `if ( eval "$cmd" )` with pipefail.
idiom_rc() { ( set -euo pipefail; if ( eval "$1" ); then echo 0; else echo $?; fi ) }

assert_rc "raw pipeline into grep -q is genuinely unsafe (141)" 141 \
    "$(idiom_rc "seq 1 3000000 | grep -q '^1\$'")"
assert_rc "PL-080 substitution idiom is genuinely safe" 0 \
    "$(idiom_rc "test -n \"\$(seq 1 3000000 | grep -m1 '^1\$')\"")"
assert_rc "herestring idiom is genuinely safe at large size" 0 \
    "$(idiom_rc "out=\$(seq 1 300000 || true); grep -q '^1\$' <<< \"\$out\"")"
assert_rc "printf idiom is genuinely UNSAFE at large size (the correction)" 141 \
    "$(idiom_rc "out=\$(seq 1 3000000 || true); printf '%s' \"\$out\" | grep -q '^1\$'")"

echo
echo "=== H. control: the real repo tree scans clean (PL-219) ==="
bash "$CHECK" >/dev/null 2>&1
assert_rc "the real tree is clean under its tracked allowlist" 0 $?

echo
echo "verification-pipefail-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
