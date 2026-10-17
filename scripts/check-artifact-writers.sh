#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-artifact-writers.sh — who may write to the hub's artifact store (T-3432, arc-011 FX-5 ruling C′)
#
# WHY: the hub's artifact store keeps files as they were sent, readable by the hub and any token
# holder. The operator ruled (FX-5 C′, T-3426, 2026-10-13) that it counts toward the FX-5 baseline and
# that FX-5 is a "workflow" floor: a NEW hub-readable copy of a content class the hub does not hold
# today is growth, however few fields it adds. So today's writers are allowed by name, and any new
# writer fails here unless it goes through the attachment encryption wrapper (PH-2 row P15), which
# will join the allowlist with that reason. A sidecar-only check would be one call site from bypass,
# so this scans every client crate.
#
# RULES (client crates only; the hub and protocol crates are the server side and the definition):
#   artifact-put-rpc   any use of `method::ARTIFACT_PUT` or the literal "artifact.put"
#   send-artifact      any call of `send_artifact_via_client(` (not its definition)
#   upload-chunked     any call of `upload_artifact_chunked(` (not its definition)
# Test code (from a file's first `#[cfg(test)]` on, and anything under tests/) is not scanned.
# Comment lines are skipped. A site is acknowledged by its signature in the git-tracked allowlist:
#   <relpath>::<enclosing-fn>::<rule>   # reason
# Fn-name based, so it survives line moves; renaming the function re-fires it on purpose.
#
# SCOPE (T-2680): it finds code that can reach the store's write RPC through these names. It does not
# prove a writer encrypts, and it does not see a writer outside the scanned crates (a script calling
# `termlink file send` goes through an allowlisted CLI writer).
#
# Exit: 0 clean · 1 an unacknowledged writer · 2 tooling (no files, unreadable allowlist).
# Usage: check-artifact-writers.sh [--root DIR]... [--allowlist F] [--json] [--quiet] [--no-heartbeat]
set -uo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOTS=() ALLOWLIST="" JSON=0 QUIET=0
while [ $# -gt 0 ]; do
    case "$1" in
        --root) ROOTS+=("${2:-}"); shift 2 ;;
        --allowlist) ALLOWLIST="${2:-}"; shift 2 ;;
        --json) JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) sed -n 2,26p "$0"; exit 0 ;;
        *) echo "check-artifact-writers: unknown arg '$1'" >&2; exit 2 ;;
    esac
done
cd "$PROJECT_ROOT" || exit 2
[ "${#ROOTS[@]}" -gt 0 ] || ROOTS=(crates/termlink-cli/src crates/termlink-mcp/src crates/termlink-session/src crates/termlink-sidecar/src)
[ -n "$ALLOWLIST" ] || ALLOWLIST=".context/checks/artifact-writer-allowlist"
if [ -e "$ALLOWLIST" ] && [ ! -r "$ALLOWLIST" ]; then echo "check-artifact-writers: allowlist unreadable: $ALLOWLIST" >&2; exit 2; fi

declare -A ALLOW=()
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        line="${line%%#*}"; line="$(printf '%s' "$line" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
        [ -n "$line" ] && ALLOW["$line"]=1
    done < "$ALLOWLIST"
fi

FILES="$(for r in "${ROOTS[@]}"; do find "$r" -type f -name '*.rs' -not -path '*/tests/*' 2>/dev/null; done | sort)"
[ -n "$FILES" ] || { echo "check-artifact-writers: no .rs files under: ${ROOTS[*]}" >&2; exit 2; }

checked=0 acked=0 firing=""
while IFS= read -r file; do
    # one awk pass: production lines only (before the first #[cfg(test)]), comments skipped,
    # each hit printed as "<line>\t<enclosing fn>\t<rule>"
    hits="$(awk '
        /^#\[cfg\(test\)\]/ { exit }
        { code = $0; sub(/\/\/.*$/, "", code) }
        match(code, /(^|[^A-Za-z0-9_])fn[ \t]+[A-Za-z0-9_]+/) {
            s = substr(code, RSTART, RLENGTH); sub(/.*fn[ \t]+/, "", s); fn = s
        }
        code ~ /method::ARTIFACT_PUT|"artifact\.put"/ { print NR "\t" fn "\tartifact-put-rpc" }
        code ~ /send_artifact_via_client\(/ && code !~ /fn[ \t]+send_artifact_via_client\(/ { print NR "\t" fn "\tsend-artifact" }
        code ~ /upload_artifact_chunked\(/ && code !~ /fn[ \t]+upload_artifact_chunked\(/ { print NR "\t" fn "\tupload-chunked" }
    ' "$file")"
    while IFS=$'\t' read -r ln fn rule; do
        [ -n "$ln" ] || continue
        checked=$((checked + 1))
        sig="${file}::${fn:--}::${rule}"
        if [ -n "${ALLOW[$sig]:-}" ]; then acked=$((acked + 1)); continue; fi
        firing+="${file}:${ln}|${fn:--}|${rule}"$'\n'
    done <<< "$hits"
done <<< "$FILES"

n_fire="$(printf '%s' "$firing" | grep -c . || true)"
if [ "$JSON" = 1 ]; then
    printf '%s' "$firing" | python3 -c '
import json, sys
rows = [l.split("|") for l in sys.stdin.read().splitlines() if l]
print(json.dumps({"ok": not rows, "checked": int(sys.argv[1]), "acknowledged": int(sys.argv[2]),
  "firing": [{"site": r[0], "fn": r[1], "rule": r[2]} for r in rows]}))' "$checked" "$acked"
elif [ "${n_fire:-0}" = 0 ]; then
    [ "$QUIET" = 1 ] || echo "check-artifact-writers: clean — $checked artifact-store write site(s), all $acked acknowledged in $ALLOWLIST"
else
    echo "check-artifact-writers: FIRING — $n_fire artifact-store write site(s) not in the allowlist:"
    printf '%s' "$firing" | grep . | while IFS='|' read -r site fn rule; do echo "  $site  $rule  (in fn $fn)"; done
    echo "  The hub keeps artifact files readable. FX-5 ruling C′: a new writer must go through the"
    echo "  attachment encryption wrapper (row P15). Route it there, or — if it is a writer the operator"
    echo "  accepts — add '<file>::<fn>::<rule>  # reason' to $ALLOWLIST."
fi
[ "${n_fire:-0}" = 0 ] || exit 1
[ "$QUIET" = 1 ] || [ "$JSON" = 1 ] || echo "  Scope: finds code that can reach the store's write RPC by these names; does not prove a writer encrypts."
exit 0
