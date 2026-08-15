#!/usr/bin/env bash
# substrate-preflight-runtime-dir-fixtures.sh (T-2729)
#
# Hermetic proof for Check 1 of scripts/substrate-preflight.sh — the PL-021
# volatile-runtime_dir guard. No root, no mounting, no host mutation: the
# resolver is driven through the real env vars it reads, and the classifier
# through the TERMLINK_PREFLIGHT_TEST_MOUNT_OUTPUT / TERMLINK_PREFLIGHT_TEST_UID
# seams.
#
# The headline fixtures are 2 and 6. Before T-2729 the check hardcoded
# `${TERMLINK_RUNTIME_DIR:-/tmp/termlink-0}`, so with XDG_RUNTIME_DIR set and no
# explicit override it inspected /tmp/termlink-0 while the hub used
# /run/user/<uid>/termlink — an unrelated path — and, because /run/user matches
# no /tmp prefix, reported PASS "persists across reboot" for a tmpfs that
# systemd destroys at logout. The guard was most confident exactly where it was
# most wrong, on the one failure mode it exists to catch.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# PREFLIGHT_SCRIPT lets the suite be pointed at another copy of the script —
# used to prove these fixtures are load-bearing by running them against the
# pre-T-2729 version, which must fail them.
CHECK="${PREFLIGHT_SCRIPT:-$REPO_ROOT/scripts/substrate-preflight.sh}"
[ -f "$CHECK" ] || { echo "FAIL: preflight not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

# A mount table with the shapes that matter: a disk root, /tmp as plain disk,
# a tmpfs at /run/user/1000, and a persistent /var/lib.
MOUNTS="$SCRATCH/mounts"
cat > "$MOUNTS" <<'EOF'
/dev/sda1 on / type ext4 (rw,relatime)
tmpfs on /run type tmpfs (rw,nosuid,nodev,mode=755)
tmpfs on /run/user/1000 type tmpfs (rw,nosuid,nodev,relatime,size=6575968k,uid=1000)
/dev/sda2 on /var type xfs (rw,relatime)
EOF

# macOS-format table (Directive #4 — the parser must handle both).
MOUNTS_MAC="$SCRATCH/mounts-mac"
cat > "$MOUNTS_MAC" <<'EOF'
/dev/disk1s5s1 on / (apfs, sealed, local, read-only, journaled)
devfs on /dev (devfs, local, nobrowse)
EOF

pass=0; fail=0

# Run preflight with a controlled environment and return Check 1's JSON object.
# `env -u` clears the vars the resolver reads so each fixture starts from a
# known state rather than inheriting the developer's shell.
check1() { # <env assignments...>
    env -u TERMLINK_RUNTIME_DIR -u XDG_RUNTIME_DIR -u TMPDIR \
        TERMLINK_PREFLIGHT_TEST_MOUNT_OUTPUT="$MOUNTS" \
        TERMLINK_PREFLIGHT_TEST_UID=1000 \
        "$@" bash "$CHECK" --json 2>/dev/null \
        | jq -c '.checks[] | select(.name=="runtime_dir")'
}

assert_field() { # <desc> <json> <jq-filter> <expected>
    local got; got="$(printf '%s' "$2" | jq -r "$3" 2>/dev/null)"
    if [ "$got" = "$4" ]; then
        pass=$((pass+1)); echo "  ok: $1"
    else
        fail=$((fail+1)); echo "  FAIL: $1 — $3 expected '$4' got '$got'" >&2
    fi
}

assert_contains() { # <desc> <json> <jq-filter> <substring>
    local got; got="$(printf '%s' "$2" | jq -r "$3" 2>/dev/null)"
    case "$got" in
        *"$4"*) pass=$((pass+1)); echo "  ok: $1" ;;
        *) fail=$((fail+1)); echo "  FAIL: $1 — expected '$3' to contain '$4', got '$got'" >&2 ;;
    esac
}

echo "== 1. explicit override on persistent disk =="
j="$(check1 TERMLINK_RUNTIME_DIR=/var/lib/termlink)"
assert_field "step 1 wins, verdict pass" "$j" '.status' 'pass'
assert_contains "names the resolved path" "$j" '.message' '/var/lib/termlink'
# AC: a PASS must show its evidence, not just assert persistence.
assert_contains "PASS states the filesystem as evidence" "$j" '.message' 'xfs'

echo "== 2. XDG_RUNTIME_DIR set, no override — the regression =="
j="$(check1 XDG_RUNTIME_DIR=/run/user/1000)"
assert_field "tmpfs under /run/user FAILS (was: pass)" "$j" '.status' 'fail'
assert_contains "resolves via step 2, not /tmp/termlink-0" "$j" '.message' '/run/user/1000/termlink'
assert_contains "names tmpfs" "$j" '.message' 'tmpfs'
# The trigger is logout, and saying "reboot" would understate it.
assert_contains "names logout as the trigger" "$j" '.message' 'logout'

echo "== 3. TMPDIR set, no override, non-zero UID =="
j="$(check1 TMPDIR=/var/folders/xyz)"
assert_contains "resolves via step 3 with real UID" "$j" '.message' '/var/folders/xyz/termlink-1000'

echo "== 4. bare fallback uses \$UID, not a hardcoded 0 =="
j="$(check1)"
assert_contains "step 4 is /tmp/termlink-<uid>" "$j" '.message' '/tmp/termlink-1000'

echo "== 5. empty override is surfaced, not papered over =="
j="$(check1 TERMLINK_RUNTIME_DIR=)"
assert_field "empty override fails" "$j" '.status' 'fail'
assert_contains "says EMPTY" "$j" '.message' 'EMPTY'

echo "== 6. volatility is decided by filesystem, not path spelling =="
# An unanticipated tmpfs location: nothing about this path looks like /tmp.
MOUNTS_ODD="$SCRATCH/mounts-odd"
cat > "$MOUNTS_ODD" <<'EOF'
/dev/sda1 on / type ext4 (rw,relatime)
tmpfs on /opt/scratch type tmpfs (rw,relatime)
EOF
j="$(env -u XDG_RUNTIME_DIR -u TMPDIR \
        TERMLINK_PREFLIGHT_TEST_MOUNT_OUTPUT="$MOUNTS_ODD" \
        TERMLINK_PREFLIGHT_TEST_UID=1000 \
        TERMLINK_RUNTIME_DIR=/opt/scratch/termlink \
        bash "$CHECK" --json 2>/dev/null | jq -c '.checks[] | select(.name=="runtime_dir")')"
assert_field "tmpfs anywhere fails" "$j" '.status' 'fail'
assert_contains "names the filesystem" "$j" '.message' 'tmpfs'

echo "== 7. macOS mount format parses (Directive #4) =="
j="$(env -u XDG_RUNTIME_DIR -u TMPDIR \
        TERMLINK_PREFLIGHT_TEST_MOUNT_OUTPUT="$MOUNTS_MAC" \
        TERMLINK_PREFLIGHT_TEST_UID=501 \
        TERMLINK_RUNTIME_DIR=/Users/x/Library/termlink \
        bash "$CHECK" --json 2>/dev/null | jq -c '.checks[] | select(.name=="runtime_dir")')"
assert_field "apfs root reads as persistent" "$j" '.status' 'pass'
assert_contains "parses fstype from parens" "$j" '.message' 'apfs'

echo "== 8. precedence: override beats XDG beats TMPDIR =="
j="$(check1 TERMLINK_RUNTIME_DIR=/var/lib/termlink XDG_RUNTIME_DIR=/run/user/1000 TMPDIR=/var/folders/xyz)"
assert_contains "step 1 beats steps 2 and 3" "$j" '.message' '/var/lib/termlink'
j="$(check1 XDG_RUNTIME_DIR=/run/user/1000 TMPDIR=/var/folders/xyz)"
assert_contains "step 2 beats step 3" "$j" '.message' '/run/user/1000/termlink'

echo ""
echo "fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
