#!/usr/bin/env bash
# T-2801 — fixtures for the stranded/stale auto-deferred envelope check.
#
# The load-bearing assertion is #9: age must come from the RECORDED timestamp, not
# file mtime. Every file in a fresh clone or worktree carries the checkout time, so
# an mtime-based age reports every envelope as brand new and STALE can never fire.
# The real P-043 is 73 days old and mtime called it 0.
#
# Host-independent (PL-213): a scratch pickup directory is built from nothing.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-pickup-deferred-freshness.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# envelope <dir> <name> <iso-timestamp> <summary>
envelope() {
    mkdir -p "$1"
    cat > "$1/$2.yaml" <<EOF
pickup_id: $2
version: 1
type: bug-report
source:
  project: "peer"
  timestamp: "$3"
payload:
  summary: "$4"
  detail: "body"
EOF
}

# crumb <dir> <name> <iso-deferred-at> <blocking-task>
crumb() {
    cat > "$1/$2.yaml.breadcrumb.yaml" <<EOF
reason: triple-dedup
blocking_task: $4
deferred_at: $3
envelope: $2.yaml
EOF
}

iso_days_ago() { date -u -d "@$(( $(date +%s) - $1 * 86400 ))" '+%Y-%m-%dT%H:%M:%SZ'; }

run() { d="$1"; shift; bash "$CHECK" --dir "$d" "$@" 2>&1; }

echo "T-2801 stranded/stale auto-deferred envelope fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1-2. No breadcrumb => STRANDED, fires. This is the real P-043 shape.
# ---------------------------------------------------------------------------
D="$TMP/stranded"; envelope "$D" "P-001" "$(iso_days_ago 73)" "Two framework bugs block inception decide"
out=$(run "$D"); rc=$?
if [ "$rc" = "1" ]; then ok "envelope with no breadcrumb fires (exit 1)"
else bad "no breadcrumb fires" "rc=$rc: $out"; fi
if echo "$out" | grep -q "STRANDED: P-001.yaml"; then ok "names the stranded envelope"
else bad "names the stranded envelope" "$out"; fi

# ---------------------------------------------------------------------------
# 3. The summary is printed — the whole failure mode is nobody knowing what is
#    in the file, so naming the file alone would repeat the original mistake.
# ---------------------------------------------------------------------------
if echo "$out" | grep -q "Two framework bugs block inception decide"; then
    ok "prints the envelope summary, not just the filename"
else bad "prints the envelope summary" "$out"; fi

# ---------------------------------------------------------------------------
# 4. A fresh, breadcrumbed envelope is healthy — the mechanism working normally
#    must not be reported as a problem.
# ---------------------------------------------------------------------------
D="$TMP/fresh"; envelope "$D" "P-002" "$(iso_days_ago 3)" "Recent filing"
crumb "$D" "P-002" "$(iso_days_ago 3)" "T-1234"
out=$(run "$D"); rc=$?
if [ "$rc" = "0" ]; then ok "fresh breadcrumbed envelope is healthy (exit 0)"
else bad "fresh breadcrumbed envelope is healthy" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 5. Breadcrumbed but old => STALE, fires, and names the blocking task.
# ---------------------------------------------------------------------------
D="$TMP/stale"; envelope "$D" "P-003" "$(iso_days_ago 90)" "Old but breadcrumbed"
crumb "$D" "P-003" "$(iso_days_ago 90)" "T-9999"
out=$(run "$D"); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "STALE: P-003.yaml"; then ok "old breadcrumbed envelope is STALE and fires"
else bad "old breadcrumbed envelope is STALE" "rc=$rc: $out"; fi
if echo "$out" | grep -q "blocked-by=T-9999"; then ok "STALE names the blocking task"
else bad "STALE names the blocking task" "$out"; fi

# ---------------------------------------------------------------------------
# 6. Threshold is tunable — the same envelope is healthy under a wider window.
# ---------------------------------------------------------------------------
out=$(run "$D" --threshold-days 365); rc=$?
if [ "$rc" = "0" ]; then ok "--threshold-days widens the window"
else bad "--threshold-days widens the window" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 7. THE LOAD-BEARING ONE. Age comes from the recorded timestamp, not mtime.
#    The files here were written seconds ago; only the recorded dates are old.
#    If this ever regresses to mtime, STALE silently stops working in every
#    clone and worktree.
# ---------------------------------------------------------------------------
D="$TMP/mtime"; envelope "$D" "P-004" "$(iso_days_ago 200)" "Written now, dated long ago"
crumb "$D" "P-004" "$(iso_days_ago 200)" "T-1111"
out=$(run "$D"); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "STALE"; then
    ok "age is read from the recorded timestamp, not file mtime"
else bad "age from recorded timestamp not mtime" "rc=$rc (files are seconds old on disk): $out"; fi
if echo "$out" | grep -qE "deferred (19[0-9]|20[0-9]) days ago"; then
    ok "reports the true recorded age"
else bad "reports the true recorded age" "$out"; fi

# ---------------------------------------------------------------------------
# 8. An envelope with no timestamp at all falls back to mtime and SAYS so,
#    because an age derived from mtime is not trustworthy.
# ---------------------------------------------------------------------------
D="$TMP/nots"; mkdir -p "$D"
printf 'pickup_id: P-005\npayload:\n  summary: "No timestamp anywhere"\n' > "$D/P-005.yaml"
out=$(run "$D")
if echo "$out" | grep -q "age from mtime"; then ok "mtime fallback is labelled as unreliable"
else bad "mtime fallback is labelled" "$out"; fi

# ---------------------------------------------------------------------------
# 9. Breadcrumb files are not themselves treated as envelopes.
# ---------------------------------------------------------------------------
D="$TMP/nodouble"; envelope "$D" "P-006" "$(iso_days_ago 2)" "Fine"
crumb "$D" "P-006" "$(iso_days_ago 2)" "T-2222"
js=$(run "$D" --json)
if echo "$js" | grep -q '"total": *1'; then ok "breadcrumb sibling is not counted as an envelope"
else bad "breadcrumb not counted as envelope" "$js"; fi

# ---------------------------------------------------------------------------
# 10. JSON carries the classes and counts.
# ---------------------------------------------------------------------------
js=$(run "$TMP/stranded" --json)
if echo "$js" | grep -q '"stranded_count": *1'; then ok "--json carries stranded_count"
else bad "--json carries stranded_count" "$js"; fi
if echo "$js" | grep -q '"class": *"STRANDED"'; then ok "--json carries the per-envelope class"
else bad "--json carries per-envelope class" "$js"; fi

# ---------------------------------------------------------------------------
# 11. An absent auto-deferred dir is healthy (nothing was ever deferred), but a
#     missing pickup tree is a tooling error — never a false clean.
# ---------------------------------------------------------------------------
mkdir -p "$TMP/pickup"
out=$(run "$TMP/pickup/auto-deferred"); rc=$?
if [ "$rc" = "0" ]; then ok "absent auto-deferred dir is healthy"
else bad "absent auto-deferred dir is healthy" "rc=$rc: $out"; fi
out=$(run "$TMP/nonexistent-tree/auto-deferred" 2>&1); rc=$?
if [ "$rc" = "2" ]; then ok "missing pickup tree => exit 2 (never a false clean)"
else bad "missing pickup tree => exit 2" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 12. --quiet is silent when healthy, loud when firing.
# ---------------------------------------------------------------------------
out=$(run "$TMP/fresh" --quiet)
if [ -z "$out" ]; then ok "--quiet is silent when healthy"
else bad "--quiet is silent when healthy" "$out"; fi
out=$(run "$TMP/stranded" --quiet)
if echo "$out" | grep -q "STRANDED"; then ok "--quiet still reports a firing envelope"
else bad "--quiet still reports firing" "$out"; fi

# ---------------------------------------------------------------------------
# 13. Bad threshold is rejected rather than silently coerced.
# ---------------------------------------------------------------------------
out=$(run "$TMP/fresh" --threshold-days abc 2>&1); rc=$?
if [ "$rc" = "2" ]; then ok "non-integer threshold => exit 2"
else bad "non-integer threshold => exit 2" "rc=$rc: $out"; fi

echo ""
echo "----------------------------------------"
printf 'T-2801 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
