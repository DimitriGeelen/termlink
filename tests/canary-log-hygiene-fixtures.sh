#!/usr/bin/env bash
# canary-log-hygiene-fixtures.sh (T-2685)
#
# Hermetic proof for scripts/check-canary-log-hygiene.sh — no /etc/cron.d, no root,
# no host state. Builds a scratch crontab dir via the check's CANARY_HYGIENE_SRC_DIR
# seam and asserts the verdict on each classification branch.
#
# The load-bearing fixtures are 1 and 3. Fixture 1 is the T-2683 F2 defect itself:
# `2>&1` on a findings log, which put `error: origin HEAD empty` into the
# release-mirror canary's log and pointed an operator at a GitHub token rotation for
# a fault that did not exist. Fixture 3 guards the tempting wrong fix — `2>/dev/null`
# silences the false positive by making a canary that cannot run fail completely
# silently, trading a bad signal for no signal at all.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-canary-log-hygiene.sh"
[ -f "$CHECK" ] || { echo "FAIL: check not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
SRC="$SCRATCH/cron"
mkdir -p "$SRC"

pass=0; fail=0
ok()  { pass=$((pass+1)); echo "  ok: $1"; }
bad() { fail=$((fail+1)); echo "  FAIL: $1" >&2; }
assert_rc() { [ "$2" -eq "$3" ] && ok "$1 (rc=$3)" || bad "$1 — expected rc=$2 got rc=$3"; }
assert_eq() { [ "$2" = "$3" ] && ok "$1 ($3)" || bad "$1 — expected '$2' got '$3'"; }

run()      { CANARY_HYGIENE_SRC_DIR="$SRC" bash "$CHECK" >/dev/null 2>&1; echo $?; }
run_json() { CANARY_HYGIENE_SRC_DIR="$SRC" bash "$CHECK" --json 2>/dev/null; }
reset()    { rm -f "$SRC"/*.crontab 2>/dev/null; }

hdr() { printf '# Installed to: /etc/cron.d/termlink-%s\nSHELL=/bin/bash\n' "$1"; }

# --- fixture 1: THE DEFECT — 2>&1 merges tooling errors into the findings log ---
reset
{ hdr alpha
  echo '13 7 * * * root cd /opt/termlink && bash scripts/check-alpha.sh --quiet >> .context/working/.alpha-canary.log 2>&1'
} > "$SRC/alpha.crontab"
assert_rc "a 2>&1 findings-log job line fires" 1 "$(run)"
assert_eq "one firing entry" "1" "$(run_json | jq -r '.firing | length')"
assert_eq "reason names the merge" "true" \
    "$(run_json | jq -r '.firing[0].why | test("merges stderr")')"

# --- fixture 2: the correct split-stream idiom passes ---------------------------
reset
{ hdr beta
  echo '13 7 * * * root cd /opt/termlink && bash scripts/check-beta.sh --quiet >> .context/working/.beta-canary.log 2>> .context/working/.beta-canary.log.stderr'
} > "$SRC/beta.crontab"
assert_rc "split-stream job line is clean" 0 "$(run)"
assert_eq "still counted as checked" "1" "$(run_json | jq -r '.checked')"
assert_eq "envelope reports ok" "true" "$(run_json | jq -r '.ok')"

# --- fixture 3: 2>/dev/null is ALSO rejected — silent failure is not hygiene ----
reset
{ hdr gamma
  echo '13 7 * * * root cd /opt/termlink && bash scripts/check-gamma.sh --quiet >> .context/working/.gamma-canary.log 2>/dev/null'
} > "$SRC/gamma.crontab"
assert_rc "discarding stderr fires too" 1 "$(run)"
assert_eq "reason names the discard" "true" \
    "$(run_json | jq -r '.firing[0].why | test("discards stderr")')"

# --- fixture 4: a redirect that is not a findings log is out of scope -----------
# An operator appending stderr to a scratch/debug file is not this defect.
reset
{ hdr delta
  echo '13 7 * * * root cd /opt/termlink && bash scripts/something.sh >> /var/log/scratch.log 2>&1'
} > "$SRC/delta.crontab"
assert_rc "a non-findings-log redirect does not fire" 0 "$(run)"
assert_eq "and is not even counted as in-scope" "0" "$(run_json | jq -r '.checked')"

# --- fixture 5: comments and env assignments are never job lines ----------------
reset
{ hdr eps
  echo '# 13 7 * * * root bash scripts/x.sh >> .context/working/.eps-canary.log 2>&1'
  echo 'MAILTO=root'
  echo '13 7 * * * root cd /opt/termlink && bash scripts/eps.sh --quiet >> .context/working/.eps-canary.log 2>> .context/working/.eps-canary.log.stderr'
} > "$SRC/eps.crontab"
assert_rc "a commented-out bad line does not fire" 0 "$(run)"
assert_eq "only the real job line is in scope" "1" "$(run_json | jq -r '.checked')"

# --- fixture 6: multiple crontabs, mixed state, all firing lines reported -------
reset
{ hdr good; echo '1 1 * * * root bash scripts/g.sh >> .context/working/.g-canary.log 2>> .context/working/.g-canary.log.stderr'; } > "$SRC/good.crontab"
{ hdr bad1; echo '2 2 * * * root bash scripts/b1.sh >> .context/working/.b1-canary.log 2>&1'; } > "$SRC/bad1.crontab"
{ hdr bad2; echo '3 3 * * * root bash scripts/b2.sh >> .context/working/.b2-canary.log 2>&1'; } > "$SRC/bad2.crontab"
assert_rc "mixed tree fires" 1 "$(run)"
assert_eq "both bad lines reported, good one not" "2" "$(run_json | jq -r '.firing | length')"
assert_eq "crontab count is 3" "3" "$(run_json | jq -r '.crontabs')"

# --- fixture 7: meta-canary aliveness lines are in scope too --------------------
# These append to .canary-aliveness.log — a findings log like any other, and the
# one that detects when a canary stops firing at all.
reset
{ hdr meta
  echo '33 8 * * * root cd /opt/termlink && CANARY_NAME="x" bash scripts/check-canary-aliveness.sh --quiet >> .context/working/.canary-aliveness.log 2>&1'
} > "$SRC/meta.crontab"
assert_rc "meta-canary aliveness line is in scope" 1 "$(run)"

# --- fixture 8: absent source dir is a tooling error, never a clean bill --------
rc="$(CANARY_HYGIENE_SRC_DIR="$SCRATCH/nope" bash "$CHECK" >/dev/null 2>&1; echo $?)"
assert_rc "absent source dir exits 2 (fail-closed)" 2 "$rc"

# --- fixture 9: an empty crontab dir is clean, not an error ---------------------
reset
assert_rc "no crontabs is clean" 0 "$(run)"
assert_eq "zero checked" "0" "$(run_json | jq -r '.checked')"

echo
echo "canary-log-hygiene fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
