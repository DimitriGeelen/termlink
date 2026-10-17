#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# count-hub-plaintext-fields.sh (T-3427, arc-011 FX-5 "disclosure does not grow")
#
# WHY: FX-5 says no feature may write content to a hub in a form a token holder or the hub
# operator can read beyond what today's rails already expose, probed by "count plaintext
# content fields on the hub per phase: never above the PH-0 baseline". No baseline existed.
# This script re-derives the count from the code so a later phase can compare against it.
#
# CLASSIFICATION RULE (applied identically everywhere): a CONTENT field carries message
# content or text derived from it (a body, a reply, a free-text evidence note, a topic
# description). Routing metadata (ids, conversation ids, offsets, timestamps, signatures,
# digests, roles, addresses, capability lists) is NOT counted. One count = one field of one
# rail that the hub stores readable (base64 is encoding, not encryption).
#
# HOW: (1) a manifest of known content fields, each counted only while its anchor text is
# still present in its file (a rail removed from the code lowers the count; a stale manifest
# cannot inflate it); (2) a drift scan of every metadata key written by the sender scripts
# and the sidecar crate: a key outside the routing allow-list is counted as an UNCLASSIFIED
# content field (conservative), so a new free-text metadata key raises the total and must be
# classified by a human before the baseline moves.
#
# SCOPE (T-2680): it counts the fields the code writes; it does not read a live hub, and it
# does not see rails written by vendored or third-party code (the AEF pickup rail).
#
# Exit: 0 = counted (and within --max if given) / 1 = total exceeds --max / 2 = tooling.
# Usage: count-hub-plaintext-fields.sh [--root DIR] [--max N] [--json] [--quiet]
set -uo pipefail

ROOT="" MAX="" JSON=0 QUIET=0
while [ $# -gt 0 ]; do
    case "$1" in
        --root) ROOT="${2:-}"; shift 2 ;;
        --max) MAX="${2:-}"; shift 2 ;;
        --json) JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) sed -n 2,26p "$0"; exit 0 ;;
        *) echo "count-hub-plaintext-fields: unknown arg '$1'" >&2; exit 2 ;;
    esac
done
[ -n "$ROOT" ] || ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[ -d "$ROOT" ] || { echo "count-hub-plaintext-fields: no such root '$ROOT'" >&2; exit 2; }
case "$MAX" in ''|*[!0-9]*) [ -z "$MAX" ] || { echo "count-hub-plaintext-fields: --max needs an integer" >&2; exit 2; } ;; esac

# rail | field | file (under ROOT) | anchor (fixed string that must be present)
MANIFEST='
channel.post|envelope.payload|crates/termlink-hub/src/channel.rs|let payload_b64 = match param_str(params, "payload_b64")
dm-turn|envelope.payload (agent-send turn)|scripts/agent-send.sh|--msg-type turn --payload "$message"
dm-turn|envelope.payload (agent-respond reply)|scripts/agent-respond.sh|--msg-type turn --payload "$reply"
chat-arc|envelope.payload (broadcast text)|scripts/chat-arc-broadcast.sh|--payload "$PAYLOAD"
receipt|metadata.evidence (free-text note)|scripts/notify-ack-read.sh|--metadata evidence="$EVIDENCE"
topic|metadata.description (topic title)|crates/termlink-cli/src/commands/channel.rs|format!("description={description}")
inbox-mirror|envelope.payload.payload (hub copy of a deposit)|crates/termlink-hub/src/channel.rs|"payload": payload,
sidecar|payload.body (message body, plaintext per PH-1 5.1)|crates/termlink-sidecar/src/wire.rs|m.insert("body".into(), self.body.clone().into());
artifact-store|artifact blob (file send, stored as sent; FX-5 ruling C′, T-3432)|crates/termlink-hub/src/artifact.rs|store.put_streaming(staging_id, offset, &chunk, is_final, expected_sha256)
'
# Routing metadata keys: never content. Anything else found by the scan is unclassified.
ROUTING_KEYS=" conversation_id up_to relay_hops to_circuit agent_id role listen_topics started_at interval_secs host pty_session capabilities addr cv_key _from _thread from_project stage sidecar_msg_id sidecar_digest sidecar_pk sidecar_sig in_reply_to description evidence "
# description/evidence are in the manifest above, so the scan must not count them twice.

declare -A RAIL_N; total=0; lines=""
while IFS='|' read -r rail field file anchor; do
    [ -n "$rail" ] || continue
    f="$ROOT/$file"
    [ -f "$f" ] || continue
    if grep -qF -- "$anchor" "$f"; then
        RAIL_N[$rail]=$(( ${RAIL_N[$rail]:-0} + 1 )); total=$((total + 1))
        n="$(grep -nF -- "$anchor" "$f" | head -1 | cut -d: -f1)"
        lines+="$rail|$field|$file:$n"$'\n'
    fi
done <<< "$MANIFEST"

# Drift scan: metadata keys written by sender scripts (non-test) and the sidecar crate.
unclassified=""
scan_files=()
while IFS= read -r p; do scan_files+=("$p"); done < <(find "$ROOT/scripts" -maxdepth 1 -name '*.sh' 2>/dev/null \
    | grep -Ev '/(check-|test-|demo-|bench-|.*selftest|.*smoke|.*fixtures)' | sort)
keys="$(for p in "${scan_files[@]}"; do
        grep -ohE -- '--metadata[ =]"?[A-Za-z_]+=' "$p" 2>/dev/null | sed -E 's/--metadata[ =]"?//; s/=$//'
    done; for p in "$ROOT"/crates/termlink-sidecar/src/{send,client,deliver}.rs; do
        [ -f "$p" ] && grep -ohE '^[[:space:]]*\("[a-z_]+"\.to_owned\(\)' "$p" | sed -E 's/^[[:space:]]*\("//; s/"\.to_owned.*//'
    done | sort -u)"
for k in $keys; do
    case "$ROUTING_KEYS" in *" $k "*) ;; *) unclassified+="$k "; total=$((total + 1)); RAIL_N[unclassified-metadata]=$(( ${RAIL_N[unclassified-metadata]:-0} + 1 )) ;; esac
done
[ "$total" -gt 0 ] || { echo "count-hub-plaintext-fields: counted 0 fields; the scan matched nothing (wrong --root?)" >&2; exit 2; }

if [ "$JSON" -eq 1 ]; then
    per="$(for r in "${!RAIL_N[@]}"; do printf '"%s":%s,' "$r" "${RAIL_N[$r]}"; done)"
    printf '{"total":%s,"per_rail":{%s},"unclassified_metadata_keys":"%s"}\n' "$total" "${per%,}" "${unclassified% }"
elif [ "$QUIET" -eq 0 ]; then
    for r in $(printf '%s\n' "${!RAIL_N[@]}" | sort); do printf '%-22s %s\n' "$r" "${RAIL_N[$r]}"; done
    [ -z "$unclassified" ] || echo "unclassified metadata keys: $unclassified"
    echo "total $total"
else
    echo "$total"
fi
if [ -n "$MAX" ] && [ "$total" -gt "$MAX" ]; then
    echo "count-hub-plaintext-fields: FX-5 FIRING: $total plaintext content fields > baseline $MAX" >&2
    exit 1
fi
exit 0
