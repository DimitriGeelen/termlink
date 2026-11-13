#!/usr/bin/env bash
# T-3774: fixtures for scripts/lib/version-cmp.sh, plus a negative control — the dotted-numeric
# comparator the scripts used before (strip non-digits per segment) gets these orderings wrong.
# Exit 0 = all held, 1 = an assertion failed.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=../scripts/lib/version-cmp.sh
. "$ROOT/scripts/lib/version-cmp.sh"
pass=0 fail=0
ok()  { pass=$((pass+1)); }
bad() { fail=$((fail+1)); echo "FAIL  $*"; }
expect() { # A op B   op in < = >
    local r; r="$(version_cmp "$1" "$3")"
    case "$2" in '<') [ "$r" = -1 ] ;; '=') [ "$r" = 0 ] ;; '>') [ "$r" = 1 ] ;; esac \
        && ok || bad "$1 $2 $3 (got $r)"
}

# The release / dev chain, strictly increasing.
chain=(0.12.10 0.12.11-rc1 0.12.11 0.12.11+4.ga2fcb16f8 0.12.11+10.g123456789 0.12.12 0.13.0-rc1 0.13.0-rc1+2.gdeadbeef0 0.13.0 1.0.0)
for ((i = 0; i < ${#chain[@]}; i++)); do
    for ((j = 0; j < ${#chain[@]}; j++)); do
        if   [ "$i" -lt "$j" ]; then expect "${chain[i]}" '<' "${chain[j]}"
        elif [ "$i" -gt "$j" ]; then expect "${chain[i]}" '>' "${chain[j]}"
        else expect "${chain[i]}" '=' "${chain[j]}"; fi
    done
done
# Sha digits never leak into the number (a sha full of 9s vs one with none).
expect 0.12.11+4.g999999999 '<' 0.12.11+5.gaaaaaaaaa
expect 0.12.11+4.g999999999 '<' 0.12.12
# Same count, different commit: equal for ordering.
expect 0.12.11+4.gaaaaaaaaa '=' 0.12.11+4.gbbbbbbbbb
# Leading v, numeric not lexical, pre-release identifier rules.
expect v0.12.11 '=' 0.12.11
expect 0.9.1591 '<' 0.11.2
expect 0.13.0-rc.2 '<' 0.13.0-rc.10
expect 0.13.0-1 '<' 0.13.0-alpha
expect 0.13.0-alpha '<' 0.13.0-alpha.1
# version_lt / version_ge wrappers
version_lt 0.12.11 0.12.11+1.gabcdef012 && ok || bad "version_lt dev"
version_ge 0.12.11+1.gabcdef012 0.12.11 && ok || bad "version_ge dev"
version_ge 0.12.11 0.12.11 && ok || bad "version_ge equal"
! version_lt 0.12.11 0.12.11 && ok || bad "version_lt equal"
# version_commit
[ "$(version_commit 0.12.11+4.ga2fcb16f8)" = a2fcb16f8 ] && ok || bad "version_commit dev"
[ -z "$(version_commit 0.12.11)" ] && ok || bad "version_commit release"
[ -z "$(version_commit 0.12.11+4.nosha)" ] && ok || bad "version_commit malformed"

# Negative control: the pre-T-3774 comparator (check-fleet-binary-freshness.sh version_lt).
old_lt() {
    local -a a b; IFS=. read -r -a a <<< "${1#v}"; IFS=. read -r -a b <<< "${2#v}"
    local i x y
    for i in 0 1 2 3; do
        x="${a[i]:-0}"; y="${b[i]:-0}"; x="${x//[!0-9]/}"; y="${y//[!0-9]/}"; x="${x:-0}"; y="${y:-0}"
        [ "$x" -lt "$y" ] && return 0; [ "$x" -gt "$y" ] && return 1
    done
    return 1
}
wrong=0
old_lt 0.12.11+4.ga2fcb16f8 0.12.12 || wrong=$((wrong+1))    # old reads 11+4 as 114
old_lt 0.12.11+4.g999999999 0.12.11+5.gaaaaaaaaa || wrong=$((wrong+1))
if [ "$wrong" -gt 0 ]; then ok; echo "PASS  control: old comparator wrong on $wrong of 2 orderings"
else bad "control: old comparator got every ordering right, the fixtures do not bite"; fi

echo "version-cmp-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
