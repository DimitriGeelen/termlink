#!/usr/bin/env bash
# T-3472 — fixtures for scripts/check-arc-driver-scoring-persisted.sh. Hermetic:
# scratch arc files only.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CHK="$ROOT/scripts/check-arc-driver-scoring-persisted.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "FAIL $1"; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
run() { bash "$CHK" --arcs-dir "$1" >"$TMP/out" 2>&1; echo $?; }

entry_lost='  - name: LOST
    weight: 5
    reviewer:
      verdict: pass
      checks:
        a: {check: scorable, verdict: pass, reason: "inline scoring: validates (3 level(s))"}'
entry_kept='  - name: KEPT
    weight: 5
    scoring: {kind: signals, levels: {3: {keywords: [x]}}}
    reviewer:
      verdict: pass
      checks:
        a: {check: scorable, verdict: pass, reason: "inline scoring: validates (3 level(s))"}'
entry_human='  - name: HUMAN
    weight: 4
    approved_by: human'

mk() { mkdir -p "$TMP/$1"; printf 'id: a\nscoped_drivers:\n%s\n' "$2" > "$TMP/$1/a.yaml"; }

mk lost "$entry_lost";  [ "$(run "$TMP/lost")" = 1 ] && grep -q "FIRING  a.yaml: LOST" "$TMP/out" && ok "1 reviewed-scorable without a block fires" || bad "1 reviewed-scorable without a block fires"
mk kept "$entry_kept";  [ "$(run "$TMP/kept")" = 0 ] && ok "2 stored block is clean" || bad "2 stored block is clean"
mk human "$entry_human"; [ "$(run "$TMP/human")" = 0 ] && ok "3 human-approved driver without reviewer is out of scope" || bad "3 human-approved driver without reviewer is out of scope"
mkdir -p "$TMP/empty"; [ "$(run "$TMP/empty")" = 2 ] && ok "4 no arc files is tooling, never clean" || bad "4 no arc files is tooling, never clean"
mkdir -p "$TMP/broken"; printf 'id: [unclosed\n' > "$TMP/broken/a.yaml"; [ "$(run "$TMP/broken")" = 2 ] && ok "5 unparseable arc is tooling" || bad "5 unparseable arc is tooling"
[ "$(run "$TMP/nonexistent")" = 2 ] && ok "6 missing dir is tooling" || bad "6 missing dir is tooling"

echo "--- $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
