#!/usr/bin/env bash
# scripts/substrate-drain-demo.sh
#
# T-2211 (arc-parallel-substrate, T-2018 §6): self-contained operator-facing
# proof of the arc's HEADLINE MECHANIC — "multiple agents execute disjoint
# work-units concurrently and merge cleanly without conflicts."
#
# N synthetic workers race to drain an M-unit work-queue using ONLY the
# shipped substrate claim primitive (#1). The demo asserts the exclusive-
# delivery guarantee at shell level: every unit is won by EXACTLY ONE
# worker (disjoint union of per-worker win-sets == the full unit set), with
# zero double-claims. This is the shell-level companion to the Rust unit
# test `concurrent_n_way_race_each_offset_won_exactly_once`
# (crates/termlink-session/tests/claim_client_integration.rs) and to the
# single-unit composition proof scripts/substrate-smoke.sh (T-2151).
#
# Composes ONLY existing verbs — no new primitive, no hub change:
#   channel create / channel post / channel claim / channel release /
#   channel claims-summary
#
# Workers HOLD their claims through the race (no mid-race release), so a
# released-then-reclaimed slot can never inflate a unit's winner count —
# the assertion is purely about claim exclusivity under contention.
#
# Exit codes:
#   0  clean drain — every unit won exactly once, no double-claim, no gap
#   1  FAIL — a double-claim, an unclaimed unit, or a worker error
#   2  usage / missing dependency (termlink, jq)
#
# Usage:
#   substrate-drain-demo.sh [--workers N] [--units M] [--hub addr]
#                           [--topic NAME] [--ttl-ms N] [--json] [--keep] [--help]

set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
WORKERS=3
UNITS=9
HUB=""
TTL_MS=60000
JSON_MODE=0
KEEP=0
TOPIC="substrate-drain-demo"

die2() { echo "substrate-drain-demo: $*" >&2; exit 2; }

while [ $# -gt 0 ]; do
  case "$1" in
    --workers) WORKERS="$2"; shift 2;;
    --units)   UNITS="$2"; shift 2;;
    --hub)     HUB="$2"; shift 2;;
    --ttl-ms)  TTL_MS="$2"; shift 2;;
    --json)    JSON_MODE=1; shift;;
    --keep)    KEEP=1; shift;;
    --topic)   TOPIC="$2"; shift 2;;
    -h|--help)
      sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) die2 "unknown arg: $1";;
  esac
done

command -v "$TERMLINK" >/dev/null 2>&1 || die2 "termlink not on PATH (set TERMLINK_BIN)"
command -v jq >/dev/null 2>&1 || die2 "jq required"
[ "$WORKERS" -ge 2 ] 2>/dev/null || die2 "--workers must be >= 2"
[ "$UNITS" -ge "$WORKERS" ] 2>/dev/null || die2 "--units must be >= --workers"

HUB_ARG=(); [ -n "$HUB" ] && HUB_ARG=(--hub "$HUB")

# Reused, bounded topic (hubs are append-only — no channel-delete verb, so a
# fixed retention-capped topic avoids accumulating a throwaway topic per run).
WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/drain-demo.XXXXXX")"
CLAIMIDS="$WORKDIR/claim-ids"
: > "$CLAIMIDS"

cleanup() {
  if [ "$KEEP" -eq 0 ]; then
    # Release every claim we hold (best-effort) so the topic frees cleanly.
    while IFS= read -r cid; do
      [ -n "$cid" ] && "$TERMLINK" "${HUB_ARG[@]}" channel release "$cid" --ack >/dev/null 2>&1
    done < "$CLAIMIDS"
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

log() { [ "$JSON_MODE" -eq 0 ] && echo "$*"; }

log "# substrate-drain-demo (T-2211) — proving the arc-001 headline mechanic"
log "#   topic=$TOPIC workers=$WORKERS units=$UNITS ttl_ms=$TTL_MS"

# ── Stage 1: create the queue topic ──────────────────────────────────────────
if ! "$TERMLINK" "${HUB_ARG[@]}" channel create "$TOPIC" \
      --retention messages --retention-value "$UNITS" >/dev/null 2>&1; then
  # create may already exist or not support retention flags on older hubs; retry plain
  "$TERMLINK" "${HUB_ARG[@]}" channel create "$TOPIC" >/dev/null 2>&1 \
    || die2 "channel create failed for $TOPIC"
fi

# ── Stage 2: seed M work-units, capturing each offset ─────────────────────────
OFFSETS=()
for u in $(seq 1 "$UNITS"); do
  out=$("$TERMLINK" "${HUB_ARG[@]}" channel post "$TOPIC" \
        --msg-type work --payload "unit-$u" --json 2>/dev/null)
  off=$(echo "$out" | jq -r '.delivered.offset // .offset // empty' 2>/dev/null)
  [ -n "$off" ] || die2 "could not read offset from post (unit $u): $out"
  OFFSETS+=("$off")
done
log "# seeded ${#OFFSETS[@]} units at offsets: ${OFFSETS[*]}"

# ── Stage 3: race N workers to claim disjoint units ───────────────────────────
# Each worker iterates the offsets in a ROTATED order (start index = worker id)
# so different workers hit different offsets first → real contention. Each
# offset is attempted at most once per worker; on conflict the worker moves on.
worker() {
  local wid="$1"; shift
  local order=("$@")
  local n=${#order[@]}
  local start=$(( (wid - 1) % n ))
  local wins="$WORKDIR/wins-$wid"; : > "$wins"
  local conflicts=0
  local i idx off out cid
  for ((i=0; i<n; i++)); do
    idx=$(( (start + i) % n ))
    off="${order[$idx]}"
    out=$("$TERMLINK" "${HUB_ARG[@]}" channel claim "$TOPIC" "$off" \
          --claimer "worker-$wid" --ttl-ms "$TTL_MS" --json 2>/dev/null)
    cid=$(echo "$out" | jq -r '.claim_id // empty' 2>/dev/null)
    if [ -n "$cid" ]; then
      echo "$off" >> "$wins"
      echo "$cid" >> "$CLAIMIDS"
      # simulate a small unit of work so this worker yields — lets the
      # steal spread across workers (proof is exclusivity, not the delay)
      sleep "0.0$(( (RANDOM % 8) + 1 ))"
    else
      conflicts=$((conflicts+1))
    fi
  done
  echo "$conflicts" > "$WORKDIR/conflicts-$wid"
}

for w in $(seq 1 "$WORKERS"); do
  worker "$w" "${OFFSETS[@]}" &
done
wait

# ── Stage 4: assert exclusive delivery ────────────────────────────────────────
ALLWINS="$WORKDIR/all-wins"
cat "$WORKDIR"/wins-* 2>/dev/null | sort -n > "$ALLWINS"
total_wins=$(wc -l < "$ALLWINS")
distinct_wins=$(sort -un "$ALLWINS" | wc -l)
total_conflicts=$(cat "$WORKDIR"/conflicts-* 2>/dev/null | awk '{s+=$1} END{print s+0}')

# double-claim = same offset in more than one worker's win-set
dupes=$(sort -n "$ALLWINS" | uniq -d | tr '\n' ' ')
# gaps = seeded offsets never won
missing=""
for off in "${OFFSETS[@]}"; do
  grep -qx "$off" "$ALLWINS" || missing="$missing $off"
done

PASS=1
[ -n "$dupes" ] && PASS=0
[ -n "$missing" ] && PASS=0
[ "$distinct_wins" -eq "$UNITS" ] || PASS=0

# per-worker distribution
dist=""
for w in $(seq 1 "$WORKERS"); do
  c=$(wc -l < "$WORKDIR/wins-$w" 2>/dev/null || echo 0)
  dist="$dist worker-$w=$c"
done

if [ "$JSON_MODE" -eq 1 ]; then
  jq -n --arg topic "$TOPIC" --argjson workers "$WORKERS" --argjson units "$UNITS" \
        --argjson total_wins "$total_wins" --argjson distinct "$distinct_wins" \
        --argjson conflicts "$total_conflicts" --arg dupes "$dupes" --arg missing "$missing" \
        --argjson pass "$PASS" \
    '{ok:($pass==1), topic:$topic, workers:$workers, units:$units,
      total_wins:$total_wins, distinct_units_won:$distinct,
      conflicts_observed:$conflicts, double_claims:($dupes|gsub("^ +| +$";"")),
      missing_units:($missing|gsub("^ +| +$";"")),
      verdict:(if $pass==1 then "clean-drain" else "FAIL" end)}'
else
  echo "# distribution:$dist"
  echo "# total_wins=$total_wins distinct_units_won=$distinct_wins/$UNITS conflicts_observed=$total_conflicts"
  [ -n "$dupes" ]   && echo "# DOUBLE-CLAIM offsets: $dupes"
  [ -n "$missing" ] && echo "# UNCLAIMED units: $missing"
  if [ "$PASS" -eq 1 ]; then
    echo "PASS: clean drain — $UNITS units, each won by exactly one of $WORKERS workers, zero double-claims (conflicts under contention: $total_conflicts)"
  else
    echo "FAIL: exclusive-delivery guarantee violated"
  fi
fi

[ "$PASS" -eq 1 ] && exit 0 || exit 1
