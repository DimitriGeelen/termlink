#!/usr/bin/env bash
# check-dashboard-deleted-root.sh (T-2848)
#
# Runtime check: is a Watchtower answering on this host serving a tree that is
# dead underneath it -- either a project_root gone from disk, or a process still
# executing from an unlinked directory inode?
#
# WHY THIS EXISTS. On 2026-08-28 the dashboard at :3003 answered HTTP 200, fast
# and well-formed, and said "Nothing needs your attention". Measured against a
# dashboard freshly started on the live tree, :3003 rendered 31 task IDs where
# the live one rendered 163 -- 132 items it could not see. A previous session
# read that page and reported "0 items need attention" to the operator as fact.
# That is the failure this check exists to make impossible.
#
# GET THE CAUSE RIGHT -- the first write-up of this (including a broadcast to
# peer projects) said the worktree had been DELETED. It had not. The worktree was
# alive; what had been unlinked was the .agentic-framework directory INSIDE it,
# replaced underneath a running server that kept serving from the dead inode:
#
#     readlink /proc/<pid>/cwd
#     .../worktrees/t2687-pickup-failopen/.agentic-framework (deleted)
#
# Detector 1 (declared root missing) is structurally blind to that, because the
# declared root existed. Hence two detectors. The correction matters more than
# the tidy story: a check written against the wrong cause would have shipped
# green over the very instance that motivated it.
#
# THE TAXONOMY THIS EXTENDS. 832-Workflow-designer (agent-chat-arc @607) named
# two states for an approvals queue -- an item is PRESENT, or it is ABSENT -- and
# observed that "absence and outage render identically". There is a fourth, and
# it is the worst of them:
#
#     PRESENT             the item shows
#     ABSENT              the item does not show (predicate gap)
#     OUTAGE              the page does not answer
#     SERVING-A-CORPSE    the page answers 200, fast, well-formed, and stale
#
# The fourth is indistinguishable from a genuinely empty queue AND from a healthy
# one. Absence fails loudly once someone counts. This one answers confidently,
# which is why it survived long enough to mislead a session.
#
# WHAT THIS DOES NOT COVER. A dashboard serving a DIFFERENT BUT PERFECTLY ALIVE
# worktree is not detectable here and is not a defect -- it is correct behaviour
# for that tree. Only the reader can know which tree they meant. That is a
# separate hazard, and the mitigation is the operator one: check /api/_identity's
# project_root before trusting a count.
#
# NOT a guard-layer member: it probes a live HTTP service, so it cannot run on a
# bare CI runner. The hermetic half is tests/dashboard-deleted-root-fixtures.sh,
# which IS a member. Same split as scripts/check-addressed-posts.sh (T-2793).
#
# Exit: 0 = no dashboard serving a dead root, 1 = FIRING, 2 = tooling error.

set -uo pipefail

JSON=0
PORTS=""
QUIET=0

while [ $# -gt 0 ]; do
    case "$1" in
        --json)   JSON=1; shift ;;
        --quiet)  QUIET=1; shift ;;
        --ports)  PORTS="$2"; shift 2 ;;
        --no-heartbeat) shift ;;
        -h|--help)
            echo "usage: check-dashboard-deleted-root.sh [--json] [--quiet] [--ports '3000 3003']"
            exit 0 ;;
        *)
            echo "check-dashboard-deleted-root: unknown flag: $1" >&2
            exit 2 ;;
    esac
done

# Test seams (PL-213): a fixture supplies a canned listener table and a canned
# identity response per port, so the whole classifier is exercisable with no
# live service and no /proc.
SEAM_PORTS="${DASHBOARD_TEST_PORTS:-}"
SEAM_ID_DIR="${DASHBOARD_TEST_IDENTITY_DIR:-}"
SEAM_ROOT_PREFIX="${DASHBOARD_TEST_ROOT_PREFIX:-}"
SEAM_PROC_DIR="${DASHBOARD_TEST_PROC_DIR:-}"

_identity_for_port() {
    local port="$1"
    if [ -n "$SEAM_ID_DIR" ]; then
        # Absent fixture file == port answers nothing (not a dashboard).
        [ -r "$SEAM_ID_DIR/$port.json" ] || return 1
        cat "$SEAM_ID_DIR/$port.json"
        return 0
    fi
    curl -sf --max-time 3 "http://localhost:${port}/api/_identity" 2>/dev/null
}

_discover_ports() {
    if [ -n "$SEAM_PORTS" ]; then printf '%s\n' $SEAM_PORTS; return 0; fi
    if [ -n "$PORTS" ];    then printf '%s\n' $PORTS;      return 0; fi
    command -v ss >/dev/null 2>&1 || return 2
    ss -tlnH 2>/dev/null | awk '{print $4}' | sed -E 's/.*:([0-9]+)$/\1/' \
        | grep -E '^[0-9]+$' | sort -un
}

_field() { # <json> <key>
    printf '%s' "$1" | grep -oE "\"$2\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" \
        | sed -E 's/.*"([^"]*)"$/\1/' | head -1
}

mapfile -t CAND < <(_discover_ports)
rc=$?
if [ $rc -eq 2 ]; then
    echo "check-dashboard-deleted-root: cannot enumerate listeners (no 'ss'); pass --ports" >&2
    exit 2
fi

checked=0; dashboards=0
FIRING_PORTS=(); FIRING_ROOTS=()
OK_PORTS=(); OK_ROOTS=()

for port in "${CAND[@]}"; do
    [ -n "$port" ] || continue
    checked=$((checked + 1))
    ident="$(_identity_for_port "$port")" || continue
    svc="$(_field "$ident" service)"
    [ "$svc" = "watchtower" ] || continue
    dashboards=$((dashboards + 1))
    proot="$(_field "$ident" project_root)"
    if [ -z "$proot" ]; then
        # A dashboard that will not name its root cannot be verified. Fail closed:
        # treat it as firing rather than silently passing it -- an unverifiable
        # dashboard is exactly the condition this check exists to refuse.
        FIRING_PORTS+=("$port"); FIRING_ROOTS+=("(identity names no project_root)")
        continue
    fi
    if [ ! -d "${SEAM_ROOT_PREFIX}${proot}" ]; then
        # Detector 1 — declared root is gone from disk.
        FIRING_PORTS+=("$port"); FIRING_ROOTS+=("$proot (root NOT on disk)")
        continue
    fi
    # Detector 2 — the root still exists, but the PROCESS is executing from an
    # unlinked directory inode: /proc/<pid>/cwd resolves with a " (deleted)"
    # suffix. This is the :3003 shape measured 2026-08-28. The worktree was
    # alive; its .agentic-framework had been unlinked and replaced underneath a
    # running server, which kept serving from the dead inode.
    #
    # Detector 1 alone is structurally blind to this: it asks whether the
    # declared root exists, and it did. Two detectors, because one shape of
    # "serving a corpse" does not imply the other -- the same reason the
    # charter-drift canary needed a category detector alongside its name one
    # (T-2680).
    pid="$(_field "$ident" pid)"
    [ -n "$pid" ] || pid="$(printf '%s' "$ident" | grep -oE '"pid"[[:space:]]*:[[:space:]]*[0-9]+' | grep -oE '[0-9]+$')"
    cwd=""
    if [ -n "$pid" ]; then
        if [ -n "$SEAM_PROC_DIR" ]; then
            [ -r "$SEAM_PROC_DIR/$pid.cwd" ] && cwd="$(cat "$SEAM_PROC_DIR/$pid.cwd")"
        elif [ -e "/proc/$pid" ]; then
            cwd="$(readlink "/proc/$pid/cwd" 2>/dev/null || true)"
        fi
    fi
    case "$cwd" in
        *" (deleted)")
            FIRING_PORTS+=("$port"); FIRING_ROOTS+=("$proot (serving from a DELETED inode: $cwd)")
            ;;
        *)
            OK_PORTS+=("$port"); OK_ROOTS+=("$proot")
            ;;
    esac
done

SCOPE="SCOPE: two detectors -- declared project_root missing from disk, and process executing from a deleted directory inode. It does NOT verify the page serves the tree YOU care about, nor that its contents are correct"

if [ "$JSON" -eq 1 ]; then
    printf '{"ok":%s,"checked":%d,"dashboards":%d,"firing":[' \
        "$([ ${#FIRING_PORTS[@]} -eq 0 ] && echo true || echo false)" "$checked" "$dashboards"
    for i in "${!FIRING_PORTS[@]}"; do
        [ "$i" -gt 0 ] && printf ','
        printf '{"port":%s,"project_root":"%s"}' "${FIRING_PORTS[$i]}" "${FIRING_ROOTS[$i]}"
    done
    printf '],"healthy":['
    for i in "${!OK_PORTS[@]}"; do
        [ "$i" -gt 0 ] && printf ','
        printf '{"port":%s,"project_root":"%s"}' "${OK_PORTS[$i]}" "${OK_ROOTS[$i]}"
    done
    printf '],"scope":"%s"}\n' "$SCOPE"
    [ ${#FIRING_PORTS[@]} -eq 0 ] && exit 0 || exit 1
fi

if [ ${#FIRING_PORTS[@]} -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || {
        echo "check-dashboard-deleted-root: clean — ${dashboards} dashboard(s) on ${checked} listener(s), all roots live"
        for i in "${!OK_PORTS[@]}"; do echo "  :${OK_PORTS[$i]} -> ${OK_ROOTS[$i]}"; done
        echo "$SCOPE"
    }
    exit 0
fi

echo "check-dashboard-deleted-root: FIRING — ${#FIRING_PORTS[@]} dashboard(s) serving a tree that is dead underneath them"
for i in "${!FIRING_PORTS[@]}"; do
    echo "  :${FIRING_PORTS[$i]}  ${FIRING_ROOTS[$i]}"
done
echo
echo "  This page answers 200 and looks healthy. Its counts are meaningless:"
echo "  it is reading a checkout that was deleted out from under it."
echo "  Action: stop that process, then restart the dashboard from a live tree."
echo "$SCOPE"
exit 1
