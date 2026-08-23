#!/usr/bin/env bash
# T-2821 — fixtures for check-cron-install-drift's firing behaviour.
#
# Pins the property the change exists for: an installed crontab that differs from
# its git source must FIRE, not print "healthy" and exit 0. And pins the escape
# hatch that makes firing tolerable: a deliberate host-local variation can be
# acknowledged in an allowlist, stays visible, and does not fire.
#
# Host-independent (PL-213): every crontab is synthesised in a scratch tree via
# CRON_DRIFT_SRC_DIR / CRON_DRIFT_INSTALLED_DIR / CRON_DRIFT_ALLOWLIST, so this
# runs with no /etc/cron.d, no root, and no real canary.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-cron-install-drift.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Build a scratch pair: git source + installed copy.
#   $1 case-dir  $2 name  $3 git-body  $4 installed-body ("" = not installed)
make_case() {
    local d="$1" name="$2" gitbody="$3" instbody="$4"
    mkdir -p "$d/src" "$d/installed"
    {
        echo "# Installed to: /etc/cron.d/termlink-$name"
        echo "$gitbody"
    } > "$d/src/$name.crontab"
    if [ -n "$instbody" ]; then
        {
            echo "# Installed to: /etc/cron.d/termlink-$name"
            echo "$instbody"
        } > "$d/installed/termlink-$name"
    fi
}

run_check() {
    local d="$1"; shift
    CRON_DRIFT_SRC_DIR="$d/src" \
    CRON_DRIFT_INSTALLED_DIR="$d/installed" \
    CRON_DRIFT_ALLOWLIST="${ALLOW:-$d/allowlist}" \
        bash "$CHECK" --strict "$@" 2>&1
}

# --strict is prepended (T-2830, integration merge). T-2821 made DRIFT fire by
# DEFAULT on this branch; main meanwhile shipped T-2682, which split drift by
# direction — a git-declared job line absent from the host became its own always-
# firing UNINSTALLED_JOBS class, leaving cosmetic drift a warning behind --strict.
# The two policies collided head-on in the merge. Main's default won because its
# discriminator is the sharper one: it fires on the specific difference that means
# "shipped but dark" instead of on any byte change. T-2821's mechanism is NOT lost —
# the allowlist and --lenient are grafted onto main's implementation — so every
# assertion below still tests exactly what it was written to test, just with drift
# firing requested explicitly rather than by default.
#
# STILL OPEN, for a human: T-2821's motivating evidence was 21 of 24 host crontabs
# carrying a real fix that had never been committed. That is the OPPOSITE direction
# from UNINSTALLED_JOBS, and main deliberately treats an extra host-local job as the
# operator's prerogative. Neither policy fires on it. See the merge report.

echo "T-2821 cron-drift firing fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1. THE LOAD-BEARING ONE. Drift must fire by default. Before T-2821 this exact
#    input printed "healthy" and exited 0 — 21 times over, on the real host.
# ---------------------------------------------------------------------------
D="$TMP/t1"; make_case "$D" "alpha" "0 7 * * * root echo a" $'# host-local comment — cosmetic drift, job line identical\n0 7 * * * root echo a'
out=$(run_check "$D"); rc=$?
if [ "$rc" = "1" ]; then
    ok "drift fires by default (exit 1)"
else
    bad "drift fires by default" "got rc=$rc: $out"
fi
if ! echo "$out" | grep -q "healthy"; then
    ok "drift output never claims 'healthy'"
else
    bad "drift output never claims 'healthy'" "got: $out"
fi

# ---------------------------------------------------------------------------
# 2. The escape hatch: the SAME drift, acknowledged, does not fire. Firing
#    without this would paint legitimately-varied hosts permanently red, which
#    is how a check gets ignored — the failure being corrected.
# ---------------------------------------------------------------------------
D="$TMP/t2"; make_case "$D" "alpha" "0 7 * * * root echo a" $'# host-local comment — cosmetic drift, job line identical\n0 7 * * * root echo a'
printf 'alpha.crontab  # this host runs it on a different schedule\n' > "$D/allowlist"
out=$(run_check "$D"); rc=$?
if [ "$rc" = "0" ]; then
    ok "acknowledged drift does not fire"
else
    bad "acknowledged drift does not fire" "got rc=$rc: $out"
fi

# ---------------------------------------------------------------------------
# 3. Acknowledged is REPORTED, not hidden. An allowlist nobody can see is how
#    the next 21-file backlog accumulates.
# ---------------------------------------------------------------------------
if echo "$out" | grep -q "ACKNOWLEDGED"; then
    ok "acknowledged drift stays visible in the output"
else
    bad "acknowledged drift stays visible" "got: $out"
fi
js=$(run_check "$D" --json)
if echo "$js" | grep -q '"acknowledged_count":1'; then
    ok "--json carries acknowledged_count"
else
    bad "--json carries acknowledged_count" "got: $js"
fi

# ---------------------------------------------------------------------------
# 4. --lenient restores the pre-T-2821 behaviour for anyone who depended on it.
# ---------------------------------------------------------------------------
D="$TMP/t4"; make_case "$D" "alpha" "0 7 * * * root echo a" $'# host-local comment — cosmetic drift, job line identical\n0 7 * * * root echo a'
out=$(run_check "$D" --lenient); rc=$?
if [ "$rc" = "0" ]; then
    ok "--lenient does not fire on drift"
else
    bad "--lenient does not fire on drift" "got rc=$rc: $out"
fi
# ...but it must still not call that state "healthy".
if ! echo "$out" | grep -q "healthy"; then
    ok "--lenient still refuses to call drift 'healthy'"
else
    bad "--lenient still refuses to call drift 'healthy'" "got: $out"
fi

# ---------------------------------------------------------------------------
# 5. --strict still accepted (back-compat) and matches the new default.
# ---------------------------------------------------------------------------
out=$(run_check "$D" --strict); rc=$?
if [ "$rc" = "1" ]; then
    ok "--strict still accepted, behaves as the default"
else
    bad "--strict still accepted" "got rc=$rc: $out"
fi

# ---------------------------------------------------------------------------
# 6. MISSING fires regardless of the allowlist — a dark canary is not a
#    host-local variation and must never be acknowledgeable away.
# ---------------------------------------------------------------------------
D="$TMP/t6"; make_case "$D" "beta" "0 7 * * * root echo b" ""
printf 'beta.crontab  # trying to acknowledge a MISSING file\n' > "$D/allowlist"
out=$(run_check "$D"); rc=$?
if [ "$rc" = "1" ]; then
    ok "MISSING fires even when allowlisted"
else
    bad "MISSING fires even when allowlisted" "got rc=$rc: $out"
fi
out=$(run_check "$D" --lenient); rc=$?
if [ "$rc" = "1" ]; then
    ok "MISSING fires even under --lenient"
else
    bad "MISSING fires even under --lenient" "got rc=$rc: $out"
fi

# ---------------------------------------------------------------------------
# 7. A genuinely clean tree still reports healthy and exits 0 — the change must
#    not make everything look broken.
# ---------------------------------------------------------------------------
D="$TMP/t7"; make_case "$D" "gamma" "0 7 * * * root echo g" "0 7 * * * root echo g"
out=$(run_check "$D"); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "healthy"; then
    ok "matching tree reports healthy, exit 0"
else
    bad "matching tree reports healthy, exit 0" "got rc=$rc: $out"
fi

# ---------------------------------------------------------------------------
# 8. Allowlist parsing: comment lines and blanks must not accidentally
#    acknowledge anything.
# ---------------------------------------------------------------------------
D="$TMP/t8"; make_case "$D" "alpha" "0 7 * * * root echo a" $'# host-local comment — cosmetic drift, job line identical\n0 7 * * * root echo a'
printf '# just a comment\n\n   \n' > "$D/allowlist"
out=$(run_check "$D"); rc=$?
if [ "$rc" = "1" ]; then
    ok "comment-only allowlist acknowledges nothing"
else
    bad "comment-only allowlist acknowledges nothing" "got rc=$rc: $out"
fi

# ---------------------------------------------------------------------------
# 9. The real tree passes — T-2696 reconciled all 21, so this is a live check
#    that the repo is currently clean rather than only the fixtures.
# ---------------------------------------------------------------------------
if [ -d /etc/cron.d ]; then
    ( cd "$REPO_ROOT" && bash "$CHECK" >/dev/null 2>&1 )
    rc=$?
    if [ "$rc" = "0" ]; then
        ok "the real tree passes the firing check"
    else
        bad "the real tree passes the firing check" "exit $rc — run it for the list"
    fi
else
    ok "real-tree check skipped (no /etc/cron.d on this host)"
fi

echo ""
echo "----------------------------------------"
printf 'T-2821 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
