#!/usr/bin/env bash
# scripts/lib/version-cmp.sh — one version comparator for every script (T-3774). Source it.
#
# TermLink versions (crates/version-derive.rs):
#   0.12.11                 a release (exactly its tag)
#   0.12.11+4.ga2fcb16f8    4 commits after v0.12.11, at commit a2fcb16f8
#   0.13.0-rc1              a pre-release tag
#
# Order: SemVer precedence (core numerically, a pre-release below its release, pre-release
# identifiers numeric-before-alphanumeric), then the commit count N at the head of the build
# metadata as a tie-break, so 0.12.11 < 0.12.11+4.g… < 0.12.11+10.g… < 0.12.12.
# Strict SemVer ignores build metadata; we read N because it is the only thing that orders two
# dev builds on the same release. Two builds with the same N and different commits compare
# equal (they are on different branches; use string equality to tell them apart).
#
# Why this file exists: the per-script comparators stripped every non-digit from a segment, so
# `11+4` read as 114 and `4.ga2fcb16f8` leaked the hex digits of the sha into the number.
#
# The pre-T-3774 dev scheme (0.12.N, N = commits since the tag) cannot be ordered against
# anything: 0.12.1610 is an old dev build, not a release newer than 0.12.11. No comparator can
# repair that; those numbers disappear as hosts move to new builds.

# _vc_num_cmp A B  -> prints -1/0/1 for two digit strings (empty = 0)
_vc_num_cmp() {
    local a="${1:-0}" b="${2:-0}"
    a="${a//[^0-9]/}"; b="${b//[^0-9]/}"; a="${a:-0}"; b="${b:-0}"
    # strip leading zeros so 10# never sees an empty string
    a="$((10#$a))"; b="$((10#$b))"
    if [ "$a" -lt "$b" ]; then echo -1; elif [ "$a" -gt "$b" ]; then echo 1; else echo 0; fi
}

# _vc_pre_cmp A B  -> SemVer pre-release identifier-list compare (both non-empty)
_vc_pre_cmp() {
    local IFS=. i x y r
    local -a pa pb
    read -r -a pa <<< "$1"; read -r -a pb <<< "$2"
    for ((i = 0; i < ${#pa[@]} || i < ${#pb[@]}; i++)); do
        [ "$i" -ge "${#pa[@]}" ] && { echo -1; return; }
        [ "$i" -ge "${#pb[@]}" ] && { echo 1; return; }
        x="${pa[i]}"; y="${pb[i]}"
        if [[ "$x" =~ ^[0-9]+$ && "$y" =~ ^[0-9]+$ ]]; then
            r="$(_vc_num_cmp "$x" "$y")"; [ "$r" != 0 ] && { echo "$r"; return; }
        elif [[ "$x" =~ ^[0-9]+$ ]]; then echo -1; return
        elif [[ "$y" =~ ^[0-9]+$ ]]; then echo 1; return
        elif [[ "$x" < "$y" ]]; then echo -1; return
        elif [[ "$x" > "$y" ]]; then echo 1; return
        fi
    done
    echo 0
}

# version_cmp A B  -> prints -1 (A<B), 0, or 1 (A>B). A leading 'v' is ignored.
version_cmp() {
    local a="${1#v}" b="${2#v}" am="" bm="" ap="" bp="" i r
    [[ "$a" == *+* ]] && { am="${a#*+}"; a="${a%%+*}"; }
    [[ "$b" == *+* ]] && { bm="${b#*+}"; b="${b%%+*}"; }
    [[ "$a" == *-* ]] && { ap="${a#*-}"; a="${a%%-*}"; }
    [[ "$b" == *-* ]] && { bp="${b#*-}"; b="${b%%-*}"; }
    local -a ac bc
    IFS=. read -r -a ac <<< "$a"; IFS=. read -r -a bc <<< "$b"
    for ((i = 0; i < ${#ac[@]} || i < ${#bc[@]}; i++)); do
        r="$(_vc_num_cmp "${ac[i]:-0}" "${bc[i]:-0}")"; [ "$r" != 0 ] && { echo "$r"; return 0; }
    done
    if [ -n "$ap" ] || [ -n "$bp" ]; then
        if [ -z "$ap" ]; then echo 1; return 0; fi
        if [ -z "$bp" ]; then echo -1; return 0; fi
        r="$(_vc_pre_cmp "$ap" "$bp")"; [ "$r" != 0 ] && { echo "$r"; return 0; }
    fi
    # Build metadata: only a leading numeric identifier (our commit count) orders.
    local an="${am%%.*}" bn="${bm%%.*}"
    [[ "$an" =~ ^[0-9]+$ ]] || an=0
    [[ "$bn" =~ ^[0-9]+$ ]] || bn=0
    _vc_num_cmp "$an" "$bn"
}

version_lt() { [ "$(version_cmp "$1" "$2")" = -1 ]; }
version_ge() { [ "$(version_cmp "$1" "$2")" != -1 ]; }

# version_commit V  -> prints the commit sha from a dev version (0.12.11+4.ga2fcb16f8 ->
# a2fcb16f8), nothing for a release. Lets a reader ask git instead of trusting the number.
version_commit() {
    local m="${1#*+}"
    [[ "$1" == *+* ]] || return 0
    m="${m#*.}"
    [[ "$m" =~ ^g([0-9a-f]{7,40})$ ]] && printf '%s\n' "${BASH_REMATCH[1]}"
    return 0
}
