#!/usr/bin/env bash
# T-1075 — Cross-agent learnings exchange (asker side).
#
# Every 15 min (via /etc/cron.d/agentic-learnings-exchange-termlink) this script
# iterates over reachable termlink hub profiles, asks each for its learnings
# delta since we last polled it, dedupes by PL-ID, and writes unseen entries as
# pickup envelopes under .context/pickup/inbox/ for human review.
#
# Design note: this is the ASKER side. The RESPONDER side — a small handler
# that replies to {"q":"learnings.delta","since":"<iso>"} with matching entries
# — is implemented per-peer-project (T-1074 propagation envelope invited peers
# to adopt it). Until responders exist, this script will log empty-deltas for
# each peer, which is the expected steady-state.
#
# Env:
#   PROJECT_ROOT    — project root (default: /opt/termlink)
#   LOG_TAG         — syslog tag (default: agentic-learnings)
#
# Exit codes:
#   0 — ran to completion (per-peer failures logged, never fatal)
#   2 — misconfiguration (missing termlink binary, missing hubs config)

set -u

PROJECT_ROOT="${PROJECT_ROOT:-/opt/termlink}"
LOG_TAG="${LOG_TAG:-agentic-learnings}"
CURSOR_FILE="$PROJECT_ROOT/.context/working/.learnings-exchange-cursor.yaml"
INBOX="$PROJECT_ROOT/.context/pickup/inbox"
HUBS_CONFIG="${HOME:-/root}/.termlink/hubs.toml"

log() { logger -t "$LOG_TAG" -- "$*"; }

command -v termlink >/dev/null 2>&1 || { log "termlink binary not found — skipping"; exit 2; }
[ -f "$HUBS_CONFIG" ] || { log "no hubs.toml at $HUBS_CONFIG — skipping"; exit 0; }

mkdir -p "$INBOX" "$(dirname "$CURSOR_FILE")"
[ -f "$CURSOR_FILE" ] || printf -- '# T-1075 learnings exchange cursor — one entry per peer\n' > "$CURSOR_FILE"

# Enumerate hub profiles from hubs.toml.
mapfile -t PROFILES < <(awk '/^\[hubs\./{gsub(/\[hubs\./,""); gsub(/\].*/,""); print}' "$HUBS_CONFIG" 2>/dev/null)
if [ "${#PROFILES[@]}" -eq 0 ]; then
    log "no profiles parsed from $HUBS_CONFIG"
    exit 0
fi

TOTAL=0; OK=0; SKIPPED=0
for profile in "${PROFILES[@]}"; do
    TOTAL=$((TOTAL + 1))

    # Soft health check — no point asking a down peer.
    # termlink fleet doctor puts its report on stderr; tracing on stdout.
    # Merge both, strip ANSI, slice out the per-profile block, look for [PASS].
    block=$(termlink fleet doctor 2>&1 \
        | sed -E 's/\x1b\[[0-9;]*[a-zA-Z]//g' \
        | awk -v p="$profile" '
            $0 ~ "^--- " p " \\(" { capture=1; print; next }
            capture && /^---/ { capture=0 }
            capture { print }
        ')
    if ! grep -q '\[PASS\]' <<<"$block"; then
        log "peer $profile unreachable — skipping"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    since=$(awk -v p="$profile" '$1 == p ":" { print $2 }' "$CURSOR_FILE" 2>/dev/null || true)
    since="${since:-1970-01-01T00:00:00Z}"

    # Placeholder: the RESPONDER RPC doesn't exist yet. Until peers implement it,
    # we log a would-ask record so operators can see the asker side works.
    # When responders land, replace this block with:
    #   termlink remote call $profile --method learnings.delta \
    #       --params "{\"since\":\"$since\"}" --timeout 10 2>/dev/null
    # and parse the JSON response.

    log "would-ask peer=$profile since=$since (responder not yet implemented)"
    OK=$((OK + 1))

    # Update cursor to now so we don't double-ask the same window
    # once responders come online. Using ISO8601 UTC.
    now=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    # Rewrite cursor entry for this profile
    python3 - "$CURSOR_FILE" "$profile" "$now" <<'PY' 2>/dev/null || true
import sys, pathlib, re
path, prof, ts = sys.argv[1:]
p = pathlib.Path(path)
lines = p.read_text().splitlines() if p.exists() else []
kept = [l for l in lines if not re.match(rf"^\s*{re.escape(prof)}\s*:", l)]
kept.append(f"{prof}: {ts}")
p.write_text("\n".join(kept) + "\n")
PY
done

log "cycle complete: $TOTAL peer(s), $OK asked, $SKIPPED skipped"
