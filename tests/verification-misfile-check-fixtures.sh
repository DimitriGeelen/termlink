#!/usr/bin/env bash
# tests/verification-misfile-check-fixtures.sh (T-2831)
#
# Fixtures for scripts/check-verification-misfile.sh — hermetic, no live binary,
# no real .tasks tree. Each case builds a scratch task corpus and pins the
# checker's verdict against it.
#
# Weighted toward the FIRING cases and the FALSE-POSITIVE guards. A detector on a
# clean corpus is trivially green, and a green check that cannot go red is not a
# check — so the mutant legs below matter more than the happy path.
set -uo pipefail

SCRIPT="${SCRIPT:-scripts/check-verification-misfile.sh}"
[ -f "$SCRIPT" ] || { echo "fixtures: $SCRIPT not found"; exit 2; }

PASS=0
FAIL=0
TMPROOT="$(mktemp -d)"
trap 'rm -rf "$TMPROOT"' EXIT

ok() { PASS=$((PASS+1)); echo "  ok   — $1"; }
no() { FAIL=$((FAIL+1)); echo "  FAIL — $1"; }

assert_rc() { # <expected> <actual> <label>
    if [ "$1" = "$2" ]; then ok "$3 (rc=$2)"; else no "$3 (expected rc=$1, got rc=$2)"; fi
}
assert_contains() { # <haystack> <needle> <label>
    case "$1" in *"$2"*) ok "$3" ;; *) no "$3 (missing: $2)" ;; esac
}
assert_not_contains() { # <haystack> <needle> <label>
    case "$1" in *"$2"*) no "$3 (unexpectedly present: $2)" ;; *) ok "$3" ;; esac
}

# mktask <dir> <name> — writes a task file from stdin, with valid frontmatter.
mktask() {
    mkdir -p "$1/active"
    {
        echo '---'
        echo "id: T-9999"
        echo "name: \"fixture\""
        echo "status: started-work"
        echo '---'
        echo
        cat
    } > "$1/active/$2.md"
}

run() { bash "$SCRIPT" --tasks-dir "$1" "${@:2}" 2>&1; }

echo "== case 1: commands under ## Evolution fire (the T-2830 shape) =="
C="$TMPROOT/c1"
mktask "$C" misfiled <<'EOF'
## Verification

# no commands here — the block landed one heading down

## Evolution

git rev-parse --verify some-branch
cargo build --release --quiet
bash scripts/verify-register-union.sh
EOF
out="$(run "$C")"; rc=$?
assert_rc 1 "$rc" "misfiled block fires"
assert_contains "$out" "FIRING" "names the firing state"
assert_contains "$out" "## Evolution" "names the offending section"
assert_contains "$out" "cargo build --release --quiet" "prints the offending command"
assert_contains "$out" "VACUOUSLY" "explains why a vacuous pass is the danger"

echo "== case 2: the SAME commands under ## Verification are clean =="
C="$TMPROOT/c2"
mktask "$C" correct <<'EOF'
## Verification

git rev-parse --verify some-branch
cargo build --release --quiet
bash scripts/verify-register-union.sh

## Evolution

Nothing structural changed during the build.
EOF
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "correctly-filed commands are clean"
assert_contains "$out" "healthy" "reports healthy"

echo "== case 3: a bare 'bash script.sh' with no flags still fires =="
# Regression: rule 5 originally required a flag or shell operator, so a plain
# `bash scripts/foo.sh` — a very common verification line — was missed. This leg
# caught that gap against the real pre-repair T-2830 (5 of 6 commands detected).
C="$TMPROOT/c3"
mktask "$C" pathonly <<'EOF'
## Verification

## RCA

bash scripts/verify-register-union.sh
python3 tools/check.py
EOF
out="$(run "$C")"; rc=$?
assert_rc 1 "$rc" "flagless command with a path argument fires"
assert_contains "$out" "bash scripts/verify-register-union.sh" "catches bare bash+path"
assert_contains "$out" "python3 tools/check.py" "catches bare python3+path"

echo "== case 4: fenced code blocks never fire (docs quoting commands) =="
C="$TMPROOT/c4"
mktask "$C" fenced <<'EOF'
## Context

Run it like this:

```
cargo build --release --quiet
bash scripts/verify-register-union.sh
```

## Verification
EOF
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "fenced commands do not fire"

echo "== case 5: HTML comment blocks never fire (the task template itself) =="
C="$TMPROOT/c5"
mktask "$C" commented <<'EOF'
## Evolution

<!-- Example:
cargo build --release --quiet
bash scripts/verify-register-union.sh
-->

## Verification
EOF
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "commented commands do not fire"

echo "== case 6: prose mid-paragraph never fires (the 49-false-positive class) =="
# Measured on the real corpus: rule 4 alone yields 49 candidates across 2559 task
# files and every one is a wrapped prose line beginning with a command word.
C="$TMPROOT/c6"
mktask "$C" prose <<'EOF'
## Context

The hub rejects the call because the
timeout convention is applied before the retry, and the
test coverage proved the emit exists but never fired, so
git --no-pager log was the only way to see it.

## Verification
EOF
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "wrapped prose starting with a command word does not fire"

echo "== case 7: indented lines never fire (nested list bodies) =="
C="$TMPROOT/c7"
mktask "$C" indented <<'EOF'
## Context

- Steps:
    cargo build --release --quiet
    bash scripts/verify-register-union.sh

## Verification
EOF
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "indented commands do not fire"

echo "== case 8: markdown markers never fire (tables, lists, quotes) =="
C="$TMPROOT/c8"
mktask "$C" markers <<'EOF'
## Context

| cargo build --release | passes |
- bash scripts/verify-register-union.sh
> git rev-parse --verify main

## Verification
EOF
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "markdown marker lines do not fire"

echo "== case 9: allowlist suppresses but still counts =="
C="$TMPROOT/c9"
mktask "$C" acked <<'EOF'
## Verification

## Evolution

cargo build --release --quiet
EOF
AL="$TMPROOT/c9-allow"
echo "$C/active/acked.md::Evolution  # fixture: deliberate" > "$AL"
out="$(run "$C" --allowlist "$AL")"; rc=$?
assert_rc 0 "$rc" "allowlisted section does not fire"
assert_contains "$out" "1 acknowledged" "acknowledged entry is still counted, not hidden"

echo "== case 10: --json envelope shape =="
C="$TMPROOT/c10"
mktask "$C" j <<'EOF'
## Verification

## RCA

cargo build --release --quiet
EOF
out="$(run "$C" --json)"; rc=$?
assert_rc 1 "$rc" "json mode preserves the exit contract"
assert_contains "$out" '"ok": false' "json carries ok:false when firing"
assert_contains "$out" '"firing_count": 1' "json carries firing_count"
assert_contains "$out" '"scope"' "json carries the scope disclaimer"

echo "== case 11: fail-closed on tooling errors =="
out="$(bash "$SCRIPT" --tasks-dir "$TMPROOT/does-not-exist" 2>&1)"; rc=$?
assert_rc 2 "$rc" "missing tasks dir exits 2, never a false clean"
assert_contains "$out" "fail-closed" "says it failed closed"

mkdir -p "$TMPROOT/empty"
out="$(bash "$SCRIPT" --tasks-dir "$TMPROOT/empty" 2>&1)"; rc=$?
assert_rc 2 "$rc" "a corpus with zero task files exits 2, never a vacuous clean"

out="$(bash "$SCRIPT" --tasks-dir "$TMPROOT/c1" --bogus-flag 2>&1)"; rc=$?
assert_rc 2 "$rc" "unknown flag exits 2"

echo "== case 12: templates/ is excluded (it is all commented boilerplate) =="
C="$TMPROOT/c12"
mkdir -p "$C/templates" "$C/active"
mktask "$C" real <<'EOF'
## Verification
EOF
{ echo '---'; echo 'id: T-0'; echo '---'; echo; echo '## Evolution'; echo;
  echo 'cargo build --release --quiet'; } > "$C/templates/default.md"
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "templates/ dir is not scanned"

echo "== case 13: --quiet prints nothing when healthy, still prints when firing =="
out="$(run "$TMPROOT/c2" --quiet)"; rc=$?
assert_rc 0 "$rc" "quiet healthy keeps exit 0"
assert_not_contains "$out" "healthy" "quiet suppresses the healthy line"
out="$(run "$TMPROOT/c1" --quiet)"; rc=$?
assert_contains "$out" "FIRING" "quiet still reports a firing finding"

echo
echo "=== $((PASS+FAIL)) assertion(s): $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] || exit 1
