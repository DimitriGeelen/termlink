#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# T-3139 — the Tier-0 approval surface latches PENDING and never clears.
#
# WHY THIS EXISTS
# ---------------
# Tier-0 is the sovereignty boundary: the agent refuses a consequential command and
# the human authorises it. Two defects were measured on 2026-09-25, both in vendored
# `bin/fw`, both filed upstream at framework:pickup offset 154 (G-062 — vendored code
# is reported, never patched locally). This guard is the LOCAL DETECTION that survives
# a re-vendor.
#
#   DEFECT 1 (not detectable here): a Tier-0 block never WRITES the blocked command to
#   .tier0-approval.pending, so the approvals page the block points at is empty. The
#   operator's words: "There is nothing for me to approve in approval route." A guard
#   cannot see a write that never happened without knowing a block occurred, and the
#   block leaves no local trace. Upstream's to fix.
#
#   DEFECT 2 (what this detects): the pending record is never CLEARED on approval. The
#   same hash sat in BOTH files —
#       .tier0-approval.pending   "<hash> 1790324636 PENDING"
#       .tier0-approval           "<hash> 1790324677"
#   — approved 41 seconds after it was queued, and still reading PENDING 88 minutes
#   later. So `fw tier0 status` says "a command is waiting for approval" forever, and a
#   REAL pending block cannot be told from debris. The monotonic-latch class (T-2556),
#   applied to the approval surface.
#
# WHY THE LATCH MATTERS MORE THAN IT LOOKS: it is what makes DEFECT 1 hard to notice.
# An operator who checks `fw tier0 status` sees "pending" and assumes the channel works.
# The status verb is always busy, so it can never say anything.
#
# SCOPE — read a green narrowly (T-2680). This answers ONE question: is a hash marked
# PENDING while the SAME hash already appears in the approved file? It does NOT verify
# that blocks are being queued at all (that is defect 1, structurally invisible here),
# that approvals are legitimate, or that Watchtower renders them.
#
# Exit: 0 clean · 1 latched · 2 tooling (FAIL-CLOSED).
set -u

STATE_DIR="${TIER0_STATE_DIR:-.context/working}"
JSON=0; QUIET=0

usage() {
    cat <<'USAGE'
Usage: check-tier0-approval-latch.sh [options]

Fires when a hash reads PENDING in .tier0-approval.pending while the SAME hash is
already recorded approved in .tier0-approval — a stale latch that makes
`fw tier0 status` report a waiting command indefinitely.

Options:
  --state-dir DIR   directory holding the tier0 files (default .context/working)
  --json            machine-readable
  --quiet           print only when something fires
  --no-heartbeat    accepted and ignored (guard-layer runner compatibility)
  --help            this text

Exit: 0 clean · 1 latched · 2 tooling (fail-closed)
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --state-dir)    STATE_DIR="${2:-}"; shift 2 ;;
        --json)         JSON=1; shift ;;
        --quiet)        QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        --help|-h)      usage; exit 0 ;;
        *) echo "check-tier0-approval-latch: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

[ -d "$STATE_DIR" ] || {
    echo "check-tier0-approval-latch: no state dir at $STATE_DIR (fail-closed)" >&2
    exit 2
}

PENDING_F="$STATE_DIR/.tier0-approval.pending"
APPROVED_F="$STATE_DIR/.tier0-approval"

SCOPE="detects a hash reading PENDING while the SAME hash is already recorded approved. It does NOT verify that Tier-0 blocks are being queued at all (that is the severe half, filed upstream, and structurally invisible from here), nor that approvals are legitimate."

emit_clean() {
    if [ "$JSON" = "1" ]; then
        printf '{"ok":true,"firing":[],"firing_count":0,"scope":"%s"}\n' "$SCOPE"
    elif [ "$QUIET" != "1" ]; then
        echo "check-tier0-approval-latch: clean — no stale PENDING latch"
        echo "  SCOPE: $SCOPE"
    fi
    exit 0
}

# No pending file at all = nothing latched. Healthy.
[ -f "$PENDING_F" ] || emit_clean

# Unreadable is never clean.
[ -r "$PENDING_F" ] || {
    echo "check-tier0-approval-latch: $PENDING_F exists but is unreadable (fail-closed)" >&2
    exit 2
}

pend_line="$(cat "$PENDING_F" 2>/dev/null)" || {
    echo "check-tier0-approval-latch: cannot read $PENDING_F (fail-closed)" >&2; exit 2; }

# Empty pending file = nothing waiting.
[ -n "${pend_line// /}" ] || emit_clean

# Only a line ending PENDING is a claim that something waits.
case "$pend_line" in
    *PENDING*) : ;;
    *) emit_clean ;;
esac

pend_hash="${pend_line%% *}"
[ -n "$pend_hash" ] || {
    echo "check-tier0-approval-latch: pending record has no hash field (fail-closed)" >&2; exit 2; }

# If the approved file carries the SAME hash, the pending record is stale.
latched=0
if [ -f "$APPROVED_F" ]; then
    [ -r "$APPROVED_F" ] || {
        echo "check-tier0-approval-latch: $APPROVED_F unreadable (fail-closed)" >&2; exit 2; }
    appr_line="$(cat "$APPROVED_F" 2>/dev/null)"
    appr_hash="${appr_line%% *}"
    [ "$appr_hash" = "$pend_hash" ] && latched=1
fi

if [ "$latched" = "1" ]; then
    short="$(printf '%s' "$pend_hash" | cut -c1-12)"
    if [ "$JSON" = "1" ]; then
        printf '{"ok":false,"firing":[{"hash":"%s","why":"same hash is already recorded approved"}],"firing_count":1,"scope":"%s"}\n' \
            "$pend_hash" "$SCOPE"
    else
        echo "check-tier0-approval-latch: stale PENDING latch on $short…"
        echo "  pending:  $pend_line"
        echo "  approved: $appr_line"
        echo ""
        echo "  The same hash appears in both files, so this request was ALREADY granted"
        echo "  and the pending record was never cleared. While it stands,"
        echo "  'fw tier0 status' reports a command waiting for approval indefinitely and"
        echo "  a REAL pending block is indistinguishable from this debris."
        echo ""
        echo "  SCOPE: $SCOPE"
        echo ""
        echo "Remediation: clear the stale pending record. The WRITER is vendored bin/fw and"
        echo "must not be patched locally (G-062) — both defects are filed upstream at"
        echo "framework:pickup offset 154."
    fi
    exit 1
fi

emit_clean
