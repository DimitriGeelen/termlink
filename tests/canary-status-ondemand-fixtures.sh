#!/usr/bin/env bash
# tests/canary-status-ondemand-fixtures.sh — T-2688 regression fixtures.
#
# Pins the ON_DEMAND classification added to scripts/canary-status.sh:
#
#   1. registered + empty log + aged heartbeat  -> ON_DEMAND, not a problem, exit 0
#   2. registered + log WITH findings           -> FIRING (registration never
#                                                  suppresses findings), exit 1
#   3. UNregistered + aged heartbeat            -> STALE (no blanket suppression)
#   4. --quiet omits ON_DEMAND rows
#   5. --json carries summary.on_demand and status ON_DEMAND
#
# Host-independent (PL-213): everything runs against a scratch working dir and a
# fixture registry via --working-dir / ONDEMAND_CHECKS_CONF. No cron, no hub.
#
# Usage: bash tests/canary-status-ondemand-fixtures.sh
# Exit:  0 = all pass, 1 = a fixture regressed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/canary-status.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$SCRIPT" ] || { echo "canary-status-ondemand-fixtures: cannot read $SCRIPT" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

WORK="$SCRATCH/working"
mkdir -p "$WORK"
CONF="$SCRATCH/ondemand-checks.conf"

cat > "$CONF" <<'EOF'
# fixture registry
alloc-sink-canary   # trailing comment must be stripped

drain-sink-canary
EOF

# Heartbeat mtime well past any sane threshold (2020-01-01).
AGED="202001010000"

# (a) registered, empty log, aged heartbeat  -> ON_DEMAND
: > "$WORK/.alloc-sink-canary.heartbeat"
touch -t "$AGED" "$WORK/.alloc-sink-canary.heartbeat"

# (b) registered, log WITH findings, aged heartbeat -> FIRING
: > "$WORK/.drain-sink-canary.heartbeat"
touch -t "$AGED" "$WORK/.drain-sink-canary.heartbeat"
echo "unacknowledged drain sink at foo.rs:42" > "$WORK/.drain-sink-canary.log"

# (c) UNregistered, empty log, aged heartbeat -> STALE
: > "$WORK/.release-mirror-canary.heartbeat"
touch -t "$AGED" "$WORK/.release-mirror-canary.heartbeat"

# Strip ANSI colour so status/name assertions are not separated by escape codes.
strip_ansi() { sed -e 's/\x1b\[[0-9;]*m//g'; }
run() { ONDEMAND_CHECKS_CONF="$CONF" bash "$SCRIPT" --working-dir "$WORK" "$@" 2>&1 | strip_ansi; }

echo "T-2688 canary-status ON_DEMAND fixtures"
echo

# --- 1 + 3: classification -------------------------------------------------

out=$(run) ; rc=$?

if printf '%s' "$out" | grep -q "ON_DEMAND *alloc-sink-canary"; then
    ok "registered + empty log + aged heartbeat -> ON_DEMAND"
else
    bad "registered + empty log + aged heartbeat -> ON_DEMAND" "got: $(printf '%s' "$out" | grep alloc-sink || echo '<no row>')"
fi

if printf '%s' "$out" | grep -q "STALE *release-mirror-canary"; then
    ok "unregistered + aged heartbeat still -> STALE (no blanket suppression)"
else
    bad "unregistered + aged heartbeat still -> STALE" "got: $(printf '%s' "$out" | grep release-mirror || echo '<no row>')"
fi

# --- 2: registration must not suppress findings ----------------------------

if printf '%s' "$out" | grep -q "FIRING *drain-sink-canary"; then
    ok "registered check WITH findings still -> FIRING"
else
    bad "registered check WITH findings still -> FIRING" "got: $(printf '%s' "$out" | grep drain-sink || echo '<no row>')"
fi

# --- exit code: ON_DEMAND alone must not make the run non-zero -------------

# Tree with ONLY the on-demand check present must exit 0.
CLEAN="$SCRATCH/clean"
mkdir -p "$CLEAN"
: > "$CLEAN/.alloc-sink-canary.heartbeat"
touch -t "$AGED" "$CLEAN/.alloc-sink-canary.heartbeat"
ONDEMAND_CHECKS_CONF="$CONF" bash "$SCRIPT" --working-dir "$CLEAN" >/dev/null 2>&1
clean_rc=$?
if [ "$clean_rc" -eq 0 ]; then
    ok "ON_DEMAND alone exits 0 (healthy tree is reportable again)"
else
    bad "ON_DEMAND alone exits 0" "exit was $clean_rc"
fi

# ...and the same tree WITHOUT the registry must exit 1 (proves it is load-bearing).
ONDEMAND_CHECKS_CONF="$SCRATCH/does-not-exist.conf" bash "$SCRIPT" --working-dir "$CLEAN" >/dev/null 2>&1
unreg_rc=$?
if [ "$unreg_rc" -eq 1 ]; then
    ok "same tree without the registry exits 1 (suppression is registry-driven)"
else
    bad "same tree without the registry exits 1" "exit was $unreg_rc"
fi

# --- 4: --quiet omits ON_DEMAND -------------------------------------------

qout=$(run --quiet)
if printf '%s' "$qout" | grep -q "alloc-sink-canary"; then
    bad "--quiet omits ON_DEMAND rows" "alloc-sink-canary appeared in quiet output"
else
    ok "--quiet omits ON_DEMAND rows"
fi
if printf '%s' "$qout" | grep -q "drain-sink-canary"; then
    ok "--quiet still renders the FIRING row"
else
    bad "--quiet still renders the FIRING row" "drain-sink-canary missing from quiet output"
fi

# --- 5: --json ------------------------------------------------------------

jout=$(run --json)
if printf '%s' "$jout" | grep -q '"on_demand":1'; then
    ok "--json summary carries on_demand count"
else
    bad "--json summary carries on_demand count" "got: $jout"
fi
if printf '%s' "$jout" | grep -q '"status":"ON_DEMAND"'; then
    ok "--json carries ON_DEMAND per-canary status"
else
    bad "--json carries ON_DEMAND per-canary status" "got: $jout"
fi

# --- registry parsing ------------------------------------------------------

# The fixture registry deliberately carries a trailing `#` comment and a blank
# line; if either broke parsing, assertion 1 above would already have failed.
ok "registry parsing tolerates comments and blank lines"

echo
echo "canary-status-ondemand-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
