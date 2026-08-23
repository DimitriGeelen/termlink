#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-platform-lock.sh (T-2693, T-2690 G3 — Directive #4 Portability)
#
# The fifth source-level static check (sibling of T-2527 alloc-sink, T-2531
# drain-sink, T-2666 silent-exit, T-2672 busy-spin), for the Constitutional
# Directive no review had ever examined: **#4 Portability — "no provider/language/
# environment lock-in"**.
#
# WHY THIS EXISTS
#
# README §Platform Support asserts `Yes` for macOS five times (core binary, PTY
# operations, Terminal.app spawn, tmux spawn, TCP hub) and names Homebrew the
# *recommended* macOS install. `release.yml` cross-builds two Darwin targets and
# publishes them to GitHub Releases, from which the Homebrew formula downloads.
#
# Yet T-2690 found `whoami`'s PID-ancestor fallback reading `/proc/<pid>/stat` with
# `.ok()?` on BOTH the CLI and MCP surfaces. `/proc` does not exist on macOS, so the
# ancestor chain collapsed to `[self]`, no session matched, and `whoami` reported
# "ambiguous — here are all candidates". Not a crash: a *plausible wrong answer*,
# indistinguishable from a genuine multi-session ambiguity, recommending an action
# (disambiguate) that could never help. That shipped with no check, and every
# existing guard was structurally blind to it — the alloc/drain/silent-exit/busy-spin
# checks all ask about resource safety, none asks "does this run off Linux?".
#
# WHAT IT FLAGS, in the product crates:
#   * `/proc/` and `/sys/` path literals  — procfs/sysfs are Linux-only
#   * `Command::new("<tool>")` for tools that do not exist on macOS:
#         ss · systemctl · journalctl · ufw · setsid · nproc · lsb_release
#
# WHAT IT DELIBERATELY DOES NOT FLAG
#   * cross-platform subprocesses: git, sh, bash, ssh, tmux, pgrep (pgrep is BSD too)
#   * `osascript` — macOS-only ON PURPOSE (the Terminal.app spawn backend)
#   * comment lines — a doc comment mentioning /proc is not a dependency on it
# The check is about UNDECLARED lock-in, not about using a subprocess at all.
#
# WHY AN ALLOWLIST RATHER THAN A HARD RULE
#
# A grep cannot tell "reads /proc and silently returns the wrong answer" from "reads
# /proc behind a runtime probe that names the limitation" or "unreachable off Linux
# because the enclosing block already required a Linux-only tool". Those are the three
# legitimate shapes, and they are distinguishable only by reading the code. So each
# confirmed site is acknowledged in `.context/checks/platform-lock-allowlist` with a
# cited reason that must state **how the non-Linux path behaves** — the acknowledgement
# is the portability documentation, in the one place that stays in sync with the code.
#
# Signatures are `<relpath>::<enclosing-fn>::<primitive>` — fn-name based, so they
# survive line moves; a fn RENAME re-fires the site, which is the intended re-review
# on meaningful change (same trade-off as the sibling checks).
#
# Exit codes: 0 clean · 1 unacknowledged platform-locked site · 2 tooling error
set -uo pipefail

ROOTS=()
ALLOWLIST=""
FORMAT=human
QUIET=0

_default_allowlist() {
    if [ -f ".context/checks/platform-lock-allowlist" ]; then
        printf '%s' ".context/checks/platform-lock-allowlist"
    else
        printf '%s' ".context/working/.platform-lock-allowlist"
    fi
}

usage() {
    cat <<'EOF'
check-platform-lock.sh — flag Linux-only primitives in the product crates
(Directive #4 Portability). macOS is a documented, recommended platform.

Flags: /proc/ and /sys/ path literals; Command::new for
  ss · systemctl · journalctl · ufw · setsid · nproc · lsb_release
Ignores: git/sh/bash/ssh/tmux/pgrep (cross-platform), osascript (macOS on purpose),
  and comment lines.

Usage: check-platform-lock.sh [OPTIONS]
  --root PATH       Scan root (repeatable; defaults to the product crates)
  --allowlist PATH  Acknowledged-site ledger
  --json            Emit {ok, firing:[{file,line,fn,primitive}], checked, allowlisted}
  --quiet           Print only on firing
  --no-heartbeat    Accepted for guard-layer parity; this check writes no heartbeat
  -h, --help        This help

Fixtures: bash tests/platform-lock-check-fixtures.sh
Exit: 0 clean · 1 unacknowledged site · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --root) ROOTS+=("$2"); shift 2 ;;
        --allowlist) ALLOWLIST="$2"; shift 2 ;;
        --json) FORMAT=json; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-platform-lock: unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(
        crates/termlink-cli/src
        crates/termlink-mcp/src
        crates/termlink-session/src
        crates/termlink-hub/src
        crates/termlink-bus/src
        crates/termlink-protocol/src
    )
fi
[ -n "$ALLOWLIST" ] || ALLOWLIST="${PLATFORM_LOCK_ALLOWLIST:-$(_default_allowlist)}"

EXIST=()
for r in "${ROOTS[@]}"; do [ -d "$r" ] && EXIST+=("$r"); done
if [ "${#EXIST[@]}" -eq 0 ]; then
    echo "check-platform-lock: no scan root exists (looked for: ${ROOTS[*]})" >&2
    exit 2
fi

# Linux-only external tools. Kept short and defensible on purpose: every entry is a
# tool genuinely absent from a stock macOS, so a hit is real lock-in rather than noise.
LINUX_ONLY_CMDS='ss|systemctl|journalctl|ufw|setsid|nproc|lsb_release'
HIT_RE="(/proc/|/sys/|Command::new\(\"($LINUX_ONLY_CMDS)\"\))"
FN_RE='(^|[^A-Za-z0-9_])fn[[:space:]]+[A-Za-z0-9_]+'

fn_name_of() { printf '%s' "$1" | sed -E 's/.*[^A-Za-z0-9_]?fn[[:space:]]+([A-Za-z0-9_]+).*/\1/'; }

# Classify which primitive a (comment-stripped) line depends on.
primitive_of() {
    case "$1" in
        *"/proc/"*) printf 'proc-path' ;;
        *"/sys/"*)  printf 'sys-path' ;;
        *) printf '%s' "$1" | sed -nE "s/.*Command::new\\(\"($LINUX_ONLY_CMDS)\"\\).*/cmd:\\1/p" ;;
    esac
}

declare -A ALLOW=()
allow_n=0
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        entry="$(printf '%s' "$line" | sed -E 's/[[:space:]]*#.*$//; s/^[[:space:]]+//; s/[[:space:]]+$//')"
        [ -n "$entry" ] || continue
        ALLOW["$entry"]=1
        allow_n=$((allow_n + 1))
    done < "$ALLOWLIST"
fi

FILES="$(find "${EXIST[@]}" -type f -name '*.rs' 2>/dev/null | sort)"
[ -n "$FILES" ] || { echo "check-platform-lock: no .rs files under scan roots" >&2; exit 2; }

checked=0
firing_lines=""

while IFS= read -r file; do
    [ -n "$file" ] || continue
    fnmap="$(grep -nE "$FN_RE" "$file" 2>/dev/null | while IFS= read -r fl; do
        fln="${fl%%:*}"; fcode="${fl#*:}"
        printf '%s:%s\n' "$fln" "$(fn_name_of "$fcode")"
    done)"

    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        lineno="${hit%%:*}"
        code="${hit#*:}"
        # Strip line comments first: a doc comment naming /proc is documentation,
        # not a dependency. If nothing survives, the hit was comment-only.
        codestripped="$(printf '%s' "$code" | sed -E 's://.*$::')"
        prim="$(primitive_of "$codestripped")"
        [ -n "$prim" ] || continue

        checked=$((checked + 1))

        encfn="-"
        while IFS= read -r fm; do
            [ -n "$fm" ] || continue
            fln="${fm%%:*}"; fnm="${fm#*:}"
            [ "$fln" -le "$lineno" ] && encfn="$fnm"
            [ "$fln" -gt "$lineno" ] && break
        done <<< "$fnmap"

        sig="${file}::${encfn}::${prim}"
        [ -n "${ALLOW[$sig]:-}" ] && continue
        firing_lines="${firing_lines}${file}:${lineno}:${encfn}:${prim}"$'\n'
    done < <(grep -nE "$HIT_RE" "$file" 2>/dev/null)
done <<< "$FILES"

fire_count="$(printf '%s' "$firing_lines" | grep -c . || true)"

if [ "$FORMAT" = json ]; then
    printf '{"ok":%s,"firing":[' "$([ "$fire_count" -eq 0 ] && echo true || echo false)"
    first=1
    while IFS= read -r l; do
        [ -n "$l" ] || continue
        f="${l%%:*}"; rest="${l#*:}"
        ln="${rest%%:*}"; rest2="${rest#*:}"
        fnn="${rest2%%:*}"; prim="${rest2#*:}"
        [ $first -eq 1 ] || printf ','
        printf '{"file":%s,"line":%s,"fn":%s,"primitive":%s}' \
            "$(printf '%s' "$f" | jq -R .)" "$ln" \
            "$(printf '%s' "$fnn" | jq -R .)" "$(printf '%s' "$prim" | jq -R .)"
        first=0
    done <<< "$firing_lines"
    printf '],"checked":%s,"allowlisted":%s}\n' "$checked" "$allow_n"
    [ "$fire_count" -eq 0 ] && exit 0 || exit 1
fi

if [ "$fire_count" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "check-platform-lock: clean — 0 unacknowledged platform-locked site(s) ($checked scanned, $allow_n acknowledged)."
    exit 0
fi

echo "check-platform-lock: FIRING — $fire_count platform-locked site(s) with no acknowledgement:"
while IFS= read -r l; do
    [ -n "$l" ] || continue
    f="${l%%:*}"; rest="${l#*:}"
    ln="${rest%%:*}"; rest2="${rest#*:}"
    fnn="${rest2%%:*}"; prim="${rest2#*:}"
    echo "  ↳ $f:$ln  (in fn $fnn)  depends on: $prim"
done <<< "$firing_lines"
cat <<'EOF'
  macOS is a DOCUMENTED, RECOMMENDED platform (README §Platform Support asserts Yes
  five times; Homebrew is the recommended macOS install). A Linux-only primitive on a
  path a macOS user reaches must not fail silently — T-2690 found exactly that in
  whoami's /proc-based PID-ancestor walk, which returned a plausible wrong answer.
  Fix: guard it behind a runtime probe that NAMES the limitation to the user (see
  metadata.rs::procfs_available, T-2691), OR — if the site degrades gracefully or is
  unreachable off Linux — add its signature to .context/checks/platform-lock-allowlist
  with a reason stating HOW the non-Linux path behaves.
EOF
exit 1
