#!/usr/bin/env bash
# PreToolUse hook for SendMessage|Agent|Task (T-3710, arc-011 P-26, B-18 log half, K-20, AL-39).
# Reports one vendor-channel use to this session's sidecar, which writes an `_admin` chain row
# and a routine needs-attention entry coalesced per (session, day, tool). When the row cannot be
# written the sidecar hook appends the use to its error log and `status` shows
# `vendor_log: degraded` (FCW-146).
#
# NEVER blocks the call (ruled, ARC4 6.10.1): always exits 0 and prints nothing, even when
# termlink is missing or predates the `vendor-channel` event (an older binary exits non-zero;
# exit 2 from a PreToolUse hook would block the tool call). Sessions not launched with a
# sidecar socket are not seen (AL-39 2.23.f).
# Test seam: TERMLINK_VENDOR_HOOK_BIN replaces the `termlink` binary (tests only).
[ -n "${TERMLINK_SIDECAR_SOCKET:-}" ] || exit 0
bin="${TERMLINK_VENDOR_HOOK_BIN:-termlink}"
command -v "$bin" >/dev/null 2>&1 || exit 0
"$bin" sidecar hook vendor-channel >/dev/null 2>&1 || true
exit 0
