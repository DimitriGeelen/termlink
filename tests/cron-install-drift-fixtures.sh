#!/usr/bin/env bash
# cron-install-drift-fixtures.sh (T-2682)
#
# Hermetic proof for scripts/check-cron-install-drift.sh — no /etc/cron.d access, no
# root, no host state. Builds a scratch git-source dir + a scratch "installed" dir via
# the check's CRON_DRIFT_SRC_DIR / CRON_DRIFT_INSTALLED_DIR test hooks and asserts the
# verdict on each classification branch.
#
# The headline fixtures are 2 and 3: T-2561 collapsed every content difference into one
# non-firing "DRIFT (warning)" bucket, which let two never-installed meta-canary job
# lines (T-2175, T-2176) hide behind a line that read like cosmetic churn. T-2682 splits
# drift by DIRECTION — a git job line absent from the installed file is scheduled work
# that is not scheduled, so it fires; comment/env churn still only warns.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-cron-install-drift.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
SRC="$SCRATCH/src"; INST="$SCRATCH/installed"
mkdir -p "$SRC" "$INST"

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)"
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
assert_json() { # <desc> <jq-filter> <expected>
    local got; got="$(run_json | jq -r "$2" 2>/dev/null)"
    if [ "$got" = "$3" ]; then pass=$((pass+1)); echo "  ok: $1 ($2 = $got)"
    else fail=$((fail+1)); echo "  FAIL: $1 — $2 expected '$3' got '$got'" >&2; fi
}
run() {
    CRON_DRIFT_SRC_DIR="$SRC" CRON_DRIFT_INSTALLED_DIR="$INST" \
        bash "$CHECK" "$@" >/dev/null 2>&1; echo $?
}
run_json() {
    CRON_DRIFT_SRC_DIR="$SRC" CRON_DRIFT_INSTALLED_DIR="$INST" \
        bash "$CHECK" --json 2>/dev/null
}
reset() { rm -f "$SRC"/*.crontab "$INST"/* 2>/dev/null; }

# --- fixture 1: declared path absent entirely (T-2561 MISSING class) ----------------
reset
cat > "$SRC/alpha.crontab" <<'EOF'
# Installed to: /etc/cron.d/termlink-alpha
17 3 * * * root cd /opt/termlink && bash scripts/alpha.sh
EOF
assert_rc "missing installed file fires" 1 "$(run)"
assert_json "missing_count is 1" '.missing_count' "1"

# --- fixture 2: THE REGRESSION — installed file lacks a git job line ----------------
# This is the T-2175/T-2176 shape: the canary line is installed, the meta-canary line
# that watches it is not. Pre-T-2682 this was rc=0 with a "DRIFT (warning)".
reset
cat > "$SRC/beta.crontab" <<'EOF'
# Installed to: /etc/cron.d/termlink-beta
23 6 * * * root cd /opt/termlink && bash scripts/beta.sh --quiet
# meta-canary — watches the job above
43 6 * * * root cd /opt/termlink && bash scripts/check-canary-aliveness.sh --quiet
EOF
cat > "$INST/termlink-beta" <<'EOF'
# Installed to: /etc/cron.d/termlink-beta
23 6 * * * root cd /opt/termlink && bash scripts/beta.sh --quiet
EOF
assert_rc "uninstalled job line fires (was a warning pre-T-2682)" 1 "$(run)"
assert_json "uninstalled_jobs_count is 1" '.uninstalled_jobs_count' "1"
assert_json "drift_count stays 0 — reclassified, not double-counted" '.drift_count' "0"
missing_line="$(run_json | jq -r '.uninstalled_jobs[0]' | cut -d'|' -f3-)"
if printf '%s' "$missing_line" | grep -q "check-canary-aliveness.sh"; then
    pass=$((pass+1)); echo "  ok: firing output quotes the exact missing job line"
else
    fail=$((fail+1)); echo "  FAIL: missing line not reported: '$missing_line'" >&2
fi

# --- fixture 3: uninstalled job fires even WITHOUT --strict ------------------------
assert_rc "uninstalled job fires without --strict" 1 "$(run)"
assert_rc "uninstalled job fires with --strict too" 1 "$(run --strict)"

# --- fixture 4: comment-only difference stays a non-firing warning -----------------
reset
cat > "$SRC/gamma.crontab" <<'EOF'
# Installed to: /etc/cron.d/termlink-gamma
# A comment that was reworded in git only.
23 6 * * * root cd /opt/termlink && bash scripts/gamma.sh
EOF
cat > "$INST/termlink-gamma" <<'EOF'
# Installed to: /etc/cron.d/termlink-gamma
# An older wording of the same comment.
23 6 * * * root cd /opt/termlink && bash scripts/gamma.sh
EOF
assert_rc "comment-only drift warns, does not fire" 0 "$(run)"
assert_json "comment-only drift counted as drift" '.drift_count' "1"
assert_json "comment-only drift is NOT an uninstalled job" '.uninstalled_jobs_count' "0"
assert_rc "comment-only drift DOES fire under --strict" 1 "$(run --strict)"

# --- fixture 5: env-assignment difference is not a job line ------------------------
reset
cat > "$SRC/delta.crontab" <<'EOF'
# Installed to: /etc/cron.d/termlink-delta
MAILTO=ops@example.com
23 6 * * * root cd /opt/termlink && bash scripts/delta.sh
EOF
cat > "$INST/termlink-delta" <<'EOF'
# Installed to: /etc/cron.d/termlink-delta
MAILTO=root
23 6 * * * root cd /opt/termlink && bash scripts/delta.sh
EOF
assert_rc "env-var-only drift warns, does not fire" 0 "$(run)"
assert_json "env-var drift is not an uninstalled job" '.uninstalled_jobs_count' "0"

# --- fixture 6: whitespace reformatting is not a difference ------------------------
reset
cat > "$SRC/eps.crontab" <<'EOF'
# Installed to: /etc/cron.d/termlink-eps
23  6  *  *  *   root   cd /opt/termlink && bash scripts/eps.sh
EOF
cat > "$INST/termlink-eps" <<'EOF'
# Installed to: /etc/cron.d/termlink-eps
23 6 * * * root cd /opt/termlink && bash scripts/eps.sh
EOF
assert_rc "whitespace-only reformat does not fire" 0 "$(run)"
assert_json "whitespace reformat is not an uninstalled job" '.uninstalled_jobs_count' "0"

# --- fixture 7: an EXTRA job present only on the host does not fire ----------------
# An operator adding a local job is their prerogative; only git-declared work that is
# absent counts as shipped-dark.
reset
cat > "$SRC/zeta.crontab" <<'EOF'
# Installed to: /etc/cron.d/termlink-zeta
23 6 * * * root cd /opt/termlink && bash scripts/zeta.sh
EOF
cat > "$INST/termlink-zeta" <<'EOF'
# Installed to: /etc/cron.d/termlink-zeta
23 6 * * * root cd /opt/termlink && bash scripts/zeta.sh
59 23 * * * root /usr/local/bin/operator-local-job.sh
EOF
assert_rc "extra installed-only job does not fire" 0 "$(run)"
assert_json "extra installed job is not an uninstalled job" '.uninstalled_jobs_count' "0"

# --- fixture 8: byte-identical is OK ----------------------------------------------
reset
cat > "$SRC/eta.crontab" <<'EOF'
# Installed to: /etc/cron.d/termlink-eta
23 6 * * * root cd /opt/termlink && bash scripts/eta.sh
EOF
cp "$SRC/eta.crontab" "$INST/termlink-eta"
assert_rc "byte-identical is healthy" 0 "$(run)"
assert_json "ok_count is 1" '.ok_count' "1"
assert_json "envelope reports ok" '.ok' "true"

# --- fixture 9: no 'Installed to:' header is skipped, not fired --------------------
reset
cat > "$SRC/theta.crontab" <<'EOF'
23 6 * * * root cd /opt/termlink && bash scripts/theta.sh
EOF
assert_rc "crontab without an Installed-to header is skipped" 0 "$(run)"
assert_json "skipped_count is 1" '.skipped_count' "1"

# --- fixture 10: missing source dir is a tooling error, never a clean bill ---------
rc="$(CRON_DRIFT_SRC_DIR="$SCRATCH/does-not-exist" CRON_DRIFT_INSTALLED_DIR="$INST" \
    bash "$CHECK" >/dev/null 2>&1; echo $?)"
assert_rc "absent source dir exits 2 (fail-closed)" 2 "$rc"

echo
echo "cron-install-drift fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
