#!/usr/bin/env bash
# scripts/sweep-test-debris.sh — T-2424: test-debris sweep via channel.delete (T-2421)
#
# Walks `termlink channel list`, selects topics matching a conservative DEBRIS
# ALLOWLIST, and deletes them one by one via `termlink channel delete <name> --yes`
# (exact-name only — the T-2421 primitive refuses wildcards by design; this script
# never passes one).
#
# SAFETY MODEL (deny-first):
#   1. A topic must FIRST survive the DENY guard (durable/operational topics and
#      whole namespaces are excluded structurally — even if a future allow pattern
#      would match them).
#   2. It must THEN match one of the explicit debris allow patterns.
#   3. Anything else is untouched. Unknown/novel topics are NOT debris.
#
# DRY-RUN BY DEFAULT. Nothing is deleted without --yes.
#
# Usage:
#   bash scripts/sweep-test-debris.sh                 # dry-run against local hub
#   bash scripts/sweep-test-debris.sh --yes           # actually delete
#   bash scripts/sweep-test-debris.sh --hub ADDR      # target another hub (their maintenance window!)
#   bash scripts/sweep-test-debris.sh --sleep-ms 100  # pacing between deletes (default 50)
#   bash scripts/sweep-test-debris.sh --list-only     # print candidate names only (for piping)
#
# Exit codes: 0 = ok (dry-run or all deletes succeeded), 1 = one or more deletes
# failed, 2 = tooling error (no termlink / hub unreachable / list failed).
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
yes=0
hub=""
sleep_ms=50
list_only=0

while [ $# -gt 0 ]; do
    case "$1" in
        --yes)       yes=1; shift ;;
        --hub)       hub="${2:?--hub needs an address}"; shift 2 ;;
        --sleep-ms)  sleep_ms="${2:?--sleep-ms needs a value}"; shift 2 ;;
        --list-only) list_only=1; shift ;;
        -h|--help)   sed -n '2,25p' "$0"; exit 0 ;;
        *) echo "sweep-test-debris: unknown arg '$1' (see --help)" >&2; exit 2 ;;
    esac
done

command -v "$TERMLINK" >/dev/null 2>&1 || { echo "sweep-test-debris: termlink not on PATH" >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "sweep-test-debris: jq not on PATH" >&2; exit 2; }

hub_args=()
[ -n "$hub" ] && hub_args=(--hub "$hub")

# The delete verb must exist in the installed binary (T-2421+).
if ! "$TERMLINK" channel delete --help >/dev/null 2>&1; then
    echo "sweep-test-debris: this termlink binary has no 'channel delete' (needs >= T-2421 build)" >&2
    exit 2
fi

list_json="$(timeout 30 "$TERMLINK" channel list "${hub_args[@]}" --json 2>/dev/null)" || {
    echo "sweep-test-debris: channel list failed — hub unreachable or auth failure" >&2
    exit 2
}

names="$(printf '%s' "$list_json" | jq -r '.topics[].name' 2>/dev/null)" || {
    echo "sweep-test-debris: could not parse channel list JSON" >&2
    exit 2
}

# ---- deny-first guard -------------------------------------------------------
# Durable/operational topics and namespaces. A topic matching ANY of these is
# never a candidate, regardless of the allowlist.
deny_topic() {
    case "$1" in
        channel:learnings|policy-decisions|framework:pickup|broadcast:global) return 0 ;;
        agent-presence|agent-chat-arc) return 0 ;;
        agent-listeners*|agent-conv-*) return 0 ;;
        dm:*) return 0 ;;
        ring20:*|health:*|inbox:*) return 0 ;;
    esac
    return 1
}

# ---- debris allowlist -------------------------------------------------------
# Only the test-debris classes named in the T-2419 §4 usage sweep. Conservative:
# a class not listed here is NOT debris, however junky it looks.
allow_topic() {
    case "$1" in
        t-[0-9]*|T-[0-9]*) return 0 ;;      # per-task smoke topics (t-1234-..., T-1234-...)
        xhub-*) return 0 ;;                  # cross-hub test topics
        stress-*) return 0 ;;                # stress-test topics
        scratch:*) return 0 ;;               # scratch namespace (incl. scratch:t2409-reachtest)
        smoke:*|smoke-*) return 0 ;;         # explicit smoke namespaces
    esac
    return 1
}

candidates=()
while IFS= read -r n; do
    [ -n "$n" ] || continue
    deny_topic "$n" && continue
    allow_topic "$n" || continue
    candidates+=("$n")
done <<< "$names"

total_topics="$(printf '%s\n' "$names" | grep -c . || true)"

if [ "$list_only" -eq 1 ]; then
    printf '%s\n' "${candidates[@]:-}"
    exit 0
fi

echo "sweep-test-debris: hub=${hub:-local}  topics=$total_topics  debris-candidates=${#candidates[@]}"

if [ "${#candidates[@]}" -eq 0 ]; then
    echo "nothing to sweep — no topics match the debris allowlist"
    exit 0
fi

if [ "$yes" -ne 1 ]; then
    echo "DRY-RUN (no --yes): would delete the following ${#candidates[@]} topic(s):"
    printf '  %s\n' "${candidates[@]}"
    echo "re-run with --yes to delete"
    exit 0
fi

deleted=0
failed=0
sleep_s="$(awk "BEGIN{print $sleep_ms/1000}")"
for n in "${candidates[@]}"; do
    if out="$(timeout 20 "$TERMLINK" channel delete "$n" --yes "${hub_args[@]}" 2>&1)"; then
        deleted=$((deleted+1))
    else
        failed=$((failed+1))
        echo "  ! delete failed: $n — $(printf '%s' "$out" | head -1)" >&2
    fi
    sleep "$sleep_s"
done

echo "sweep-test-debris: deleted=$deleted failed=$failed (of ${#candidates[@]} candidates)"
[ "$failed" -eq 0 ] || exit 1
exit 0
