#!/usr/bin/env bash
# Fixtures for scripts/gate-inception-voi.sh (T-3174).
#
# Weighted to the OVERRIDE cases. The operator's instruction was "a gate, but it must
# have a way through, otherwise it is just friction" — so the waiver path is the part
# most worth pinning, and the mutant proves the reason requirement is load-bearing.
set -uo pipefail
GATE="${GATE:-scripts/gate-inception-voi.sh}"
TMP="$(mktemp -d)"; trap 'rm -rf -- "$TMP"' EXIT
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad(){ FAIL=$((FAIL+1)); printf '  FAIL %s\n       %s\n' "$1" "${2:-}"; }
rc_is(){ if [ "$2" = "$3" ]; then ok "$1"; else bad "$1" "expected rc=$2 got rc=$3"; fi; }

mk(){ # mk <file> <id> <type> <created> <voi-line> [waiver-line]
  mkdir -p "$(dirname "$1")"
  { echo "---"; echo "id: $2"; echo "name: \"t\""; echo "workflow_type: $3"
    echo "created: $4"; [ -n "${5:-}" ] && echo "$5"; [ -n "${6:-}" ] && echo "$6"
    echo "---"; echo "body"; } > "$1"
}

D="$TMP/.tasks/active"; NEW="2026-10-01T00:00:00Z"; OLD="2026-01-01T00:00:00Z"
run(){ bash "$GATE" --task "$1" >/dev/null 2>&1; echo $?; }

echo "== Case 1: the firing cases =="
mk "$D/a.md" T-1 inception "$NEW" "voi_score: 0.5"
rc_is "new inception at the 0.5 template default -> BLOCK" 1 "$(run "$D/a.md")"
mk "$D/b.md" T-2 inception "$NEW" ""
rc_is "new inception with voi_score absent -> BLOCK" 1 "$(run "$D/b.md")"
mk "$D/c.md" T-3 inception "$NEW" "voi_score: not-a-number"
rc_is "malformed voi_score reads as unset -> BLOCK" 1 "$(run "$D/c.md")"

echo "== Case 2: the clear cases =="
mk "$D/d.md" T-4 inception "$NEW" "voi_score: 0.9"
rc_is "considered value -> clear" 0 "$(run "$D/d.md")"
mk "$D/e.md" T-5 inception "$NEW" "voi_score: 0.0"
rc_is "0.0 is a considered value, not 'empty' -> clear" 0 "$(run "$D/e.md")"
mk "$D/f.md" T-6 build "$NEW" ""
rc_is "non-inception never fires" 0 "$(run "$D/f.md")"
mk "$D/g.md" T-7 inception "$OLD" "voi_score: 0.5"
rc_is "grandfathered by created date -> clear" 0 "$(run "$D/g.md")"

echo "== Case 3: THE OVERRIDE (operator requirement) =="
mk "$D/h.md" T-8 inception "$NEW" "voi_score: 0.5" 'voi_score_waived: "throwaway spike, ranking irrelevant"'
rc_is "waiver WITH a reason -> clear" 0 "$(run "$D/h.md")"
mk "$D/i.md" T-9 inception "$NEW" "" 'voi_score_waived: "absent on purpose, one-off"'
rc_is "waiver clears an ABSENT value too" 0 "$(run "$D/i.md")"
mk "$D/j.md" T-10 inception "$NEW" "voi_score: 0.5" 'voi_score_waived: ""'
rc_is "EMPTY reason does NOT waive" 1 "$(run "$D/j.md")"
mk "$D/k.md" T-11 inception "$NEW" "voi_score: 0.5" 'voi_score_waived: true'
rc_is "bare 'true' states no reason -> does NOT waive" 1 "$(run "$D/k.md")"
mk "$D/l.md" T-12 inception "$NEW" "voi_score: 0.5" 'voi_score_waived: "   "'
rc_is "whitespace-only reason -> does NOT waive" 1 "$(run "$D/l.md")"

echo "== Case 4: the waiver is LOGGED, never silent =="
LED="$TMP/ledger"
FW_VOI_GATE_LEDGER="$LED" bash "$GATE" --task "$D/h.md" >/dev/null 2>&1
if [ -f "$LED" ] && grep -q 'throwaway spike' "$LED"; then ok "waiver appended to the ledger with its reason"; else bad "waiver logging" "$(cat "$LED" 2>&1)"; fi
if [ -f "$LED" ] && grep -q 'T-8' "$LED"; then ok "ledger records which task was waived"; else bad "ledger task id" ""; fi
FW_VOI_GATE_LEDGER="$LED" bash "$GATE" --task "$D/h.md" >/dev/null 2>&1
n=$(grep -c 'throwaway spike' "$LED" 2>/dev/null || echo 0)
if [ "$n" -ge 2 ]; then ok "ledger is append-only, so bypass VOLUME is countable"; else bad "append-only" "count=$n"; fi
FW_VOI_GATE_LEDGER="/nonexistent-dir-xyz/ledger" bash "$GATE" --task "$D/h.md" >/dev/null 2>&1
rc_is "an unwritable ledger never blocks the operator" 0 "$?"

echo "== Case 5: FAIL-OPEN (this guards a ranking number, not a safety property) =="
printf 'no frontmatter at all\n' > "$D/m.md"
rc_is "file with no frontmatter -> clear, not blocked" 0 "$(run "$D/m.md")"
printf -- '---\n: : bad yaml [\n---\nx\n' > "$D/n.md"
rc_is "unparseable frontmatter -> clear, not blocked" 0 "$(run "$D/n.md")"
rc_is "absent task file -> clear, not blocked" 0 "$(run "$D/does-not-exist.md")"

echo "== Case 6: scan mode and reporting =="
out="$(FW_VOI_GATE_TASKS_DIR="$D" bash "$GATE" --scan --json 2>&1)"
printf '%s' "$out" | grep -q '"firing_count"' && ok "scan --json reports firing_count" || bad "json firing_count" "$out"
printf '%s' "$out" | grep -q '"waived_count"' && ok "scan reports waived COUNT (never hidden)" || bad "json waived_count" "$out"
printf '%s' "$out" | grep -q '"grandfathered_count"' && ok "scan reports grandfathered count" || bad "json grandfathered" "$out"
printf '%s' "$out" | grep -q 'does NOT judge whether a set value' && ok "carries the scope disclaimer (T-2680)" || bad "scope" "$out"
FW_VOI_GATE_TASKS_DIR="$D" bash "$GATE" --scan >/dev/null 2>&1
rc_is "scan exits 1 while anything is firing" 1 "$?"
grep -q '^# guard-layer: source' "$GATE" && ok "carries the guard-layer marker (T-2683)" || bad "marker" ""

echo "== Case 7: MUTANT — drop the reason requirement =="
M="$TMP/mutant.sh"
sed 's|    if not r or r.lower() in ("true", "yes", "none", "null"):|    if False:|' "$GATE" > "$M"
bash "$M" --task "$D/j.md" >/dev/null 2>&1
rc_is "mutant lets an EMPTY-reason waiver through (proves the check is load-bearing)" 0 "$?"
bash "$GATE" --task "$D/j.md" >/dev/null 2>&1
rc_is "  ...while the real gate still blocks it" 1 "$?"

printf '\n%s\n' "inception-voi-gate fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
