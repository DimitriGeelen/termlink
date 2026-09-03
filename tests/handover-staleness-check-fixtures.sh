#!/usr/bin/env bash
# T-2883 — fixtures for scripts/check-handover-staleness.sh
#
# Weighted toward the FIRING cases and the false-positive guards. A register-style
# check is trivially green when nothing is wrong, and a green check that cannot go
# red is not a check (the T-2812 lesson, applied here).
#
# Case 12 is deliberately built from the REAL pre-fix corpus pulled out of git,
# not from synthetic mutants — a detector that only ever fires on hand-built inputs
# has not been shown to fire on the thing it was written for.
#
# Usage: bash tests/handover-staleness-check-fixtures.sh
# Exit:  0 = all assertions pass, 1 = a failure, 2 = tooling error.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="${CHECK:-$REPO_ROOT/scripts/check-handover-staleness.sh}"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n     expected: %s\n     actual:   %s\n' "$1" "$2" "$3"; }
die() { printf 'handover-staleness-fixtures: %s\n' "$1" >&2; exit 2; }

[ -f "$CHECK" ] || die "check not found: $CHECK"
command -v python3 >/dev/null 2>&1 || die "python3 not found"

WORK="$(mktemp -d)" || die "mktemp failed"
trap 'rm -rf "$WORK"' EXIT

EMPTY_ALLOW="$WORK/empty-allowlist"
: > "$EMPTY_ALLOW"

# make_handover <dir> <stamp> <suggested-action> [<decisions-body>]
make_handover() {
    local dir="$1" stamp="$2" action="$3" decisions="${4:-[TODO: unfilled]}"
    mkdir -p "$dir"
    cat > "$dir/S-$stamp.md" <<HEOF
---
session_id: S-$stamp
enrichment_status: pending
---

## Decisions Made This Session

$decisions

## Things Tried That Failed

[TODO: unfilled]

## Open Questions / Blockers

[TODO: unfilled]

## Gotchas / Warnings for Next Session

[TODO: unfilled]

## Suggested First Action

$action

## Recent Commits
HEOF
}

run() { bash "$CHECK" --allowlist "$EMPTY_ALLOW" "$@" 2>&1; }
rc()  { bash "$CHECK" --allowlist "$EMPTY_ALLOW" "$@" >/dev/null 2>&1; echo $?; }

echo "handover-staleness-check-fixtures: $CHECK"
echo

# ── Case 1: axis A fires on a constant suggested action ────────────────────────
D1="$WORK/d1"
for i in 1 2 3 4 5; do make_handover "$D1" "2026-0101-000$i" 'Continue T-1457: "stuck"'; done
out="$(run --handovers-dir "$D1" --window 5)"; r="$(rc --handovers-dir "$D1" --window 5)"
case "$out" in *"constant-suggested-action"*) ok "case 1: axis A fires on a constant suggested action" ;;
  *) bad "case 1: axis A fires" "constant-suggested-action" "$out" ;; esac
[ "$r" = "1" ] && ok "case 1b: firing exit code is 1" || bad "case 1b: firing exit code" "1" "$r"

# ── Case 2 (FALSE-POSITIVE GUARD): one differing value clears axis A ───────────
D2="$WORK/d2"
for i in 1 2 3 4; do make_handover "$D2" "2026-0101-000$i" 'Continue T-1457: "stuck"'; done
make_handover "$D2" "2026-0101-0005" 'Continue T-2871: "the real focus"'
out="$(run --handovers-dir "$D2" --window 5)"; r="$(rc --handovers-dir "$D2" --window 5)"
case "$out" in *"constant-suggested-action"*) bad "case 2: axis A self-clears" "no axis A finding" "$out" ;;
  *) ok "case 2: axis A self-clears as soon as one handover differs" ;; esac
[ "$r" = "0" ] && ok "case 2b: clean exit code is 0" || bad "case 2b: clean exit code" "0" "$r"

# ── Case 3: axis B fires on a fabricated literal in the newest handover ───────
D3="$WORK/d3"
make_handover "$D3" "2026-0101-0001" 'Continue T-1000: "a"'
make_handover "$D3" "2026-0101-0002" 'Continue T-2000: "b"' 'None'
out="$(run --handovers-dir "$D3" --window 2)"
case "$out" in *"fabricated-literal"*"Decisions Made This Session"*) ok "case 3: axis B fires on a bare 'None' narrative body" ;;
  *) bad "case 3: axis B fires on 'None'" "fabricated-literal finding" "$out" ;; esac

# ── Case 4 (FALSE-POSITIVE GUARD): [TODO] markers do not fire ─────────────────
D4="$WORK/d4"
make_handover "$D4" "2026-0101-0001" 'Continue T-1000: "a"'
make_handover "$D4" "2026-0101-0002" 'Continue T-2000: "b"' '[TODO: decisions taken this session]'
out="$(run --handovers-dir "$D4" --window 2)"; r="$(rc --handovers-dir "$D4" --window 2)"
case "$out" in *"fabricated-literal"*) bad "case 4: [TODO] must not fire" "no axis B finding" "$out" ;;
  *) ok "case 4: an explicit [TODO] marker does not fire axis B" ;; esac
[ "$r" = "0" ] && ok "case 4b: fixed-shape corpus is clean" || bad "case 4b: fixed corpus clean" "0" "$r"

# ── Case 5: axis C is counted, never fired on ─────────────────────────────────
# Every handover in D4 carries enrichment_status: pending, yet the corpus is clean.
case "$out" in *"enrichment_status: pending"*"counted, not a finding"*)
    ok "case 5: unenriched count is reported on the CLEAN path, and does not fire" ;;
  *) bad "case 5: axis C counted on clean path" "text noting counted-not-a-finding" "$out" ;; esac

# ── Case 6: axis C is also reported on the FIRING path ────────────────────────
out1="$(run --handovers-dir "$D1" --window 5)"
case "$out1" in *"counted, not a finding"*) ok "case 6: unenriched count is reported on the FIRING path too" ;;
  *) bad "case 6: axis C counted on firing path" "counted-not-a-finding note" "$out1" ;; esac

# ── Case 7: scope disclaimer on BOTH paths (T-2680) ───────────────────────────
c7=0
case "$out"  in *"does NOT verify"*) c7=$((c7+1)) ;; esac
case "$out1" in *"does NOT verify"*) c7=$((c7+1)) ;; esac
[ "$c7" = "2" ] && ok "case 7: scope disclaimer present on both clean and firing paths" \
  || bad "case 7: scope on both paths" "2 paths carrying it" "$c7"

# ── Case 8 (FAIL-CLOSED): missing directory exits 2, never 0 ──────────────────
r="$(rc --handovers-dir "$WORK/does-not-exist" --window 3)"
[ "$r" = "2" ] && ok "case 8: missing handovers dir exits 2 (fail-closed)" \
  || bad "case 8: missing dir fail-closed" "2" "$r"

# ── Case 9 (FAIL-CLOSED): an empty corpus exits 2, never a vacuous clean ──────
mkdir -p "$WORK/empty-dir"
r="$(rc --handovers-dir "$WORK/empty-dir" --window 3)"
[ "$r" = "2" ] && ok "case 9: empty corpus exits 2, not a vacuous clean" \
  || bad "case 9: empty corpus fail-closed" "2" "$r"

# ── Case 10 (FAIL-CLOSED): a nonsensical window is a tooling error ────────────
r="$(rc --handovers-dir "$D1" --window 1)"
[ "$r" = "2" ] && ok "case 10: --window 1 exits 2 (nothing to compare against)" \
  || bad "case 10: window validation" "2" "$r"
r="$(rc --handovers-dir "$D1" --window abc)"
[ "$r" = "2" ] && ok "case 10b: non-numeric --window exits 2" || bad "case 10b: window validation" "2" "$r"

# ── Case 11: the allowlist suppresses a finding but still counts it ───────────
AL="$WORK/al"
printf 'Continue T-1457: "stuck"  # acknowledged for this fixture\n' > "$AL"
out="$(bash "$CHECK" --handovers-dir "$D1" --window 5 --allowlist "$AL" 2>&1)"
r=$?; bash "$CHECK" --handovers-dir "$D1" --window 5 --allowlist "$AL" >/dev/null 2>&1; r=$?
case "$out" in *"constant-suggested-action"*) bad "case 11: allowlist suppresses axis A" "no finding" "$out" ;;
  *) ok "case 11: an allowlisted suggested action does not fire" ;; esac
[ "$r" = "0" ] && ok "case 11b: allowlisted corpus exits 0" || bad "case 11b: allowlisted exit" "0" "$r"
case "$out" in *"1 acknowledged"*) ok "case 11c: the acknowledged count is reported, not hidden" ;;
  *) bad "case 11c: acknowledged count reported" "'1 acknowledged'" "$out" ;; esac

# ── Case 12 (REAL CORPUS): fires on the actual pre-fix handovers from git ─────
# Not a synthetic mutant — three handovers as they really stood at 35affce76.
D12="$WORK/d12"; mkdir -p "$D12"; got=0
for f in S-2026-0902-0846.md S-2026-0902-0849.md S-2026-0902-0850.md; do
    if git -C "$REPO_ROOT" show "35affce76:.context/handovers/$f" > "$D12/$f" 2>/dev/null && [ -s "$D12/$f" ]; then
        got=$((got+1))
    else
        rm -f "$D12/$f"
    fi
done
if [ "$got" -eq 3 ]; then
    out="$(run --handovers-dir "$D12" --window 3)"; r="$(rc --handovers-dir "$D12" --window 3)"
    c=0
    case "$out" in *"constant-suggested-action"*) c=$((c+1)) ;; esac
    case "$out" in *"fabricated-literal"*)        c=$((c+1)) ;; esac
    if [ "$c" -eq 2 ] && [ "$r" = "1" ]; then
        ok "case 12: fires on the REAL pre-fix corpus (both axis A and axis B)"
        printf '       axis A caught: %s\n' \
          "$(printf '%s' "$out" | grep -m1 'constant-suggested-action' | cut -c1-88)"
    else
        bad "case 12: real pre-fix corpus fires" "axis A + axis B, exit 1" "axes=$c exit=$r"
    fi
else
    bad "case 12: real pre-fix corpus available" "3 handovers from 35affce76" "$got retrieved"
fi

# ── Case 13: --json is parseable and carries the axes + scope ────────────────
jout="$(bash "$CHECK" --handovers-dir "$D1" --window 5 --allowlist "$EMPTY_ALLOW" --json 2>/dev/null)"
if printf '%s' "$jout" | python3 -c "
import json,sys
d=json.load(sys.stdin)
assert d['ok'] is False, 'ok should be False when firing'
assert d['firing_count'] >= 1
assert 'scope' in d and 'axes' in d
assert d['unenriched_in_window'] == 5
print('json-ok')
" >/dev/null 2>&1; then
    ok "case 13: --json parses and carries firing/axes/scope/unenriched"
else
    bad "case 13: --json shape" "parseable envelope with axes+scope" "$(printf '%s' "$jout" | head -3)"
fi

# ── Case 14: --quiet prints nothing on the clean path ────────────────────────
qout="$(bash "$CHECK" --handovers-dir "$D4" --window 2 --allowlist "$EMPTY_ALLOW" --quiet 2>&1)"
[ -z "$qout" ] && ok "case 14: --quiet is silent when clean" || bad "case 14: --quiet silent" "no output" "$qout"

# ── Case 15: the guard-layer marker is present ──────────────────────────────
if sed -n '2p' "$CHECK" | grep -q '^# guard-layer: source'; then
    ok "case 15: carries the '# guard-layer: source' marker so CI runs it"
else
    bad "case 15: guard-layer marker" "'# guard-layer: source' on line 2" "$(sed -n '2p' "$CHECK")"
fi

echo
printf 'passed: %d   failed: %d\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then echo "ALL PASS"; exit 0; fi
echo "FAILURES PRESENT"; exit 1
