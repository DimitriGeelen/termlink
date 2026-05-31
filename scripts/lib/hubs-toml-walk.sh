#!/usr/bin/env bash
# T-1892 — Shared helpers for hubs.toml-walking scripts.
#
# Extracted from chat-arc-broadcast.sh (T-1889 inline dedup) so the same
# correctness fix lands in every wrapper that iterates ~/.termlink/hubs.toml
# profiles. Two profiles can list the same physical hub under different
# addresses (canonical example: workstation-107-public at 192.168.10.107:9100
# AND local-test at 127.0.0.1:9100 both hit the same hub bound to 0.0.0.0:9100).
# Without dedup, every fleet walk visits that hub twice — at best wasted work,
# at worst double-counted alerts in canaries.
#
# Source via:
#   _self="${BASH_SOURCE[0]}"
#   _libdir="$(cd "$(dirname "$_self")" && pwd)/lib"
#   # shellcheck source=/dev/null
#   . "$_libdir/hubs-toml-walk.sh"
#
# Public API:
#   dedup_addrs_by_fp <log-prefix>     reads stdin, writes stdout, logs to stderr
#
# Required env (callers must set or accept the defaults below):
#   TERMLINK        — termlink binary (default: "termlink")
#   TIMEOUT_CMD     — pre-built "timeout <secs>" or "" (callers usually set this)
#
# Caveats:
#   - Un-probeable addresses (probe fails, no fingerprint) PASS THROUGH unchanged.
#     Fail-open is intentional — a wedged hub should be visible to the caller's
#     per-hub loop (which surfaces the real error) rather than silently dropped here.
#   - Order: the first address seen per fingerprint becomes the canonical one
#     that gets kept. Callers that need deterministic ordering should pre-sort.

# Guard against double-source.
[ -n "${_TERMLINK_HUBS_TOML_WALK_LOADED:-}" ] && return 0
_TERMLINK_HUBS_TOML_WALK_LOADED=1

# dedup_addrs_by_fp [log-prefix]
#
# Reads newline-separated records from stdin, where each record is either:
#   - a bare address (e.g. "192.168.10.107:9100")
#   - or address<TAB>extra-fields (e.g. "192.168.10.107:9100\tring20-management")
#
# Probes the address (first whitespace-delimited field) via
# `termlink hub probe --json`, groups by TLS leaf-cert fingerprint, and writes
# the WHOLE INPUT LINE for the canonical-per-fingerprint record to stdout.
# Each suppressed duplicate emits a one-line stderr message:
#   <prefix>: skipping duplicate <addr> (same hub as <canonical>, fingerprint=<8hex>)
#
# Bare-address callers (e.g. chat-arc-broadcast) get bare addresses back.
# TSV callers (e.g. canary with name+addr pairs) get the full TSV row back,
# so they can rebuild parallel arrays by walking the kept set.
#
# Probe failures (no fingerprint extractable) PASS THROUGH — see file header.
#
# Args:
#   $1 — log prefix for stderr lines (default: "hubs-toml-walk")
dedup_addrs_by_fp() {
    local prefix="${1:-hubs-toml-walk}"
    local termlink="${TERMLINK:-termlink}"
    local timeout_cmd="${TIMEOUT_CMD:-}"
    local line addr fp_out fp fp_short
    declare -A _fp_to_canonical=()
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        # First whitespace-delimited field is the address (works for bare
        # addresses and addr<TAB>extra rows alike).
        addr="${line%%[[:space:]]*}"
        [ -n "$addr" ] || continue
        fp_out="$($timeout_cmd "$termlink" hub probe "$addr" --json 2>/dev/null || true)"
        fp="$(printf '%s' "$fp_out" | jq -r '.fingerprint // empty' 2>/dev/null || true)"
        if [ -z "$fp" ]; then
            # Probe failed — fail-open. Caller's per-hub loop will surface
            # the real error.
            printf '%s\n' "$line"
            continue
        fi
        if [ -n "${_fp_to_canonical[$fp]:-}" ]; then
            fp_short="${fp#sha256:}"
            fp_short="${fp_short:0:8}"
            echo "$prefix: skipping duplicate $addr (same hub as ${_fp_to_canonical[$fp]}, fingerprint=$fp_short)" >&2
            continue
        fi
        _fp_to_canonical[$fp]="$addr"
        printf '%s\n' "$line"
    done
}
