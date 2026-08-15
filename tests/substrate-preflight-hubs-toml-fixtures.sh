#!/usr/bin/env bash
# substrate-preflight-hubs-toml-fixtures.sh (T-2742)
#
# Hermetic proof for Check 2 of scripts/substrate-preflight.sh — the hubs.toml
# presence check. No host mutation: the config path comes from the
# TERMLINK_PREFLIGHT_HUBS_TOML seam and the hub socket from a scratch
# TERMLINK_RUNTIME_DIR, so the real $HOME and the real runtime dir are never
# read or written.
#
# The headline fixture is 1. Before T-2742 the check warned whenever
# hubs.toml was absent, full stop. But config.rs treats a missing file as an
# empty config and the local hub is reached at runtime_dir()/hub.sock with no
# profile — so an operator running purely locally was correctly configured and
# warned at anyway, on every single run. That is the PL-219 class the repo
# already names: a guard that fires on a healthy state trains its operator to
# stop reading it, which costs more than the warning was worth.
#
# Fixture 2 is the other half, and the reason this is a narrowing rather than a
# deletion: with no hubs.toml AND no local hub there really is nothing to talk
# to, and that must still warn.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# PREFLIGHT_SCRIPT lets the suite be pointed at another copy of the script —
# used to prove these fixtures are load-bearing against the pre-T-2742 version.
CHECK="${PREFLIGHT_SCRIPT:-$REPO_ROOT/scripts/substrate-preflight.sh}"
[ -f "$CHECK" ] || { echo "FAIL: preflight not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

RD_WITH_HUB="$SCRATCH/rt-with-hub"
RD_NO_HUB="$SCRATCH/rt-no-hub"
mkdir -p "$RD_WITH_HUB" "$RD_NO_HUB"

# A real AF_UNIX socket, so the check's `-S` test sees a socket rather than a
# regular file — a plain `touch` would pass `-e` and prove nothing.
if command -v python3 >/dev/null 2>&1; then
    python3 - "$RD_WITH_HUB/hub.sock" <<'PY'
import socket, sys
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.bind(sys.argv[1])
PY
else
    echo "SKIP: python3 required to create a test socket" >&2
    exit 0
fi
[ -S "$RD_WITH_HUB/hub.sock" ] || { echo "FAIL: could not create test socket" >&2; exit 1; }

MISSING_TOML="$SCRATCH/absent/hubs.toml"     # deliberately never created

POPULATED_TOML="$SCRATCH/populated.toml"
cat > "$POPULATED_TOML" <<'EOF'
[hubs.alpha]
address = "192.168.10.107:9100"
EOF

EMPTY_TOML="$SCRATCH/empty.toml"
printf '# no hub sections here\n' > "$EMPTY_TOML"

pass=0; fail=0

# Run preflight with a controlled environment and return Check 2's JSON object.
check2() { # <env assignments...>
    env -u XDG_RUNTIME_DIR -u TMPDIR \
        "$@" bash "$CHECK" --json 2>/dev/null \
        | jq -c '.checks[] | select(.name=="hubs.toml")'
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

echo "== 1. no hubs.toml but a local hub is serving — a valid local-only install =="
j="$(check2 TERMLINK_PREFLIGHT_HUBS_TOML="$MISSING_TOML" TERMLINK_RUNTIME_DIR="$RD_WITH_HUB")"
assert_field "verdict is pass, not warn" "$j" '.status' "pass"
assert_contains "message names the serving socket" "$j" '.message' "hub.sock"
assert_contains "message says fleet emptiness is by design" "$j" '.message' "by design"

echo "== 2. no hubs.toml and no local hub — genuinely nothing to talk to =="
j="$(check2 TERMLINK_PREFLIGHT_HUBS_TOML="$MISSING_TOML" TERMLINK_RUNTIME_DIR="$RD_NO_HUB")"
assert_field "still warns" "$j" '.status' "warn"
assert_contains "remediation offers a local hub" "$j" '.remediation' "hub start"
assert_contains "remediation still offers a profile" "$j" '.remediation' "profile add"

echo "== 3. populated hubs.toml passes regardless of local hub =="
j="$(check2 TERMLINK_PREFLIGHT_HUBS_TOML="$POPULATED_TOML" TERMLINK_RUNTIME_DIR="$RD_NO_HUB")"
assert_field "verdict is pass" "$j" '.status' "pass"
assert_contains "message counts the declared hubs" "$j" '.message' "1 hub(s) declared"

echo "== 4. present but no [hubs.NAME] sections still warns (unchanged branch) =="
j="$(check2 TERMLINK_PREFLIGHT_HUBS_TOML="$EMPTY_TOML" TERMLINK_RUNTIME_DIR="$RD_WITH_HUB")"
assert_field "warns even though a local hub is serving" "$j" '.status' "warn"
assert_contains "message names the empty-sections cause" "$j" '.message' "no [hubs.NAME] sections"

echo
echo "fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
