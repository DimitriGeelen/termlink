#!/usr/bin/env bash
# scripts/orchestrator-backlog-drain.sh
#
# T-2204 (T-2018 §6 arc-parallel-substrate): first in-tree consumer of the
# substrate primitives shipped over the last several sessions. Discovers
# agent-eligible work-units from .tasks/active/, finds idle workers on
# agent-presence via the find-idle (#2) primitive, dispatches each unit
# through claim (#1) + claim-transfer (#3) + DM (agent contact).
#
# --dry-run is the default; --live is required to actually dispatch. This
# script reads but never mutates source files. It writes only to the hub
# (post + claim + DM) and only in --live mode.
#
# Substrate primitives exercised:
#   #2 find-idle           discover LIVE-AND-IDLE workers on agent-presence
#   #1 claim               exclusive reservation on a work-queue offset
#   #3 claim-transfer      atomic handoff orchestrator → worker (T-2046, no race)
#      agent contact       DM dispatch with task brief
#   #1 release             worker --ack on completion (worker-side)
#   #10 governor (read)    pre-flight back-pressure check
#
# Pattern reference: docs/operations/substrate-orchestrator-recipe.md
# Coordination reference: .tasks/active/T-2204-aef-coordination*.md

set -euo pipefail

# ── defaults ────────────────────────────────────────────────────────────────
CAPABILITY="backlog-drain"
QUEUE_TOPIC="work-queue"
LIMIT=20
PER_WORKER_MAX=3
MODE="dry-run"
ORCHESTRATOR_ID=""
HUB=""

usage() {
  cat <<EOF
orchestrator-backlog-drain.sh — substrate parallel-worker orchestrator (T-2204)

Usage: $0 [--dry-run|--live] [options]

Modes:
  --dry-run                (default) print intended dispatches; no hub writes
  --live                   actually post-claim-transfer-DM; explicit opt-in

Options:
  --capability NAME        find-idle filter (default: backlog-drain)
  --queue-topic NAME       work-queue topic name (default: work-queue)
  --limit N                max work-units to dispatch this pass (default: 20)
  --per-worker-max N       max concurrent dispatches per worker (default: 3)
  --hub ADDR               hub override (default: local via be-reachable state)
  --orchestrator-id ID     override orchestrator identity (default: be-reachable state)
  -h, --help               show this help

Output (one line per work-unit):
  DISPATCH [DRY-RUN|LIVE|SKIP] worker=<id> unit=<T-XXX> classification=<…>
           claim_payload=<json>
           dm_body=<rendered brief>

Exit codes:
  0    dispatched (or dry-run completed successfully)
  1    --live attempted but post/claim/transfer/DM failed for one+ units
  2    flag error / no work-units / orchestrator identity unresolved

Substrate consumer pattern documented in:
  docs/operations/substrate-orchestrator-recipe.md
EOF
}

# ── flag parse ──────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)         MODE="dry-run"; shift ;;
    --live)            MODE="live"; shift ;;
    --capability)      CAPABILITY="$2"; shift 2 ;;
    --queue-topic)     QUEUE_TOPIC="$2"; shift 2 ;;
    --limit)           LIMIT="$2"; shift 2 ;;
    --per-worker-max)  PER_WORKER_MAX="$2"; shift 2 ;;
    --hub)             HUB="$2"; shift 2 ;;
    --orchestrator-id) ORCHESTRATOR_ID="$2"; shift 2 ;;
    -h|--help)         usage; exit 0 ;;
    *)                 echo "unknown flag: $1" >&2; usage >&2; exit 2 ;;
  esac
done

# ── orchestrator identity ───────────────────────────────────────────────────
if [ -z "$ORCHESTRATOR_ID" ]; then
  if [ -f ~/.termlink/be-reachable.state ] && command -v jq >/dev/null 2>&1; then
    ORCHESTRATOR_ID="$(jq -r .agent_id ~/.termlink/be-reachable.state 2>/dev/null || true)"
  fi
fi
[ -z "$ORCHESTRATOR_ID" ] && ORCHESTRATOR_ID="${TERMLINK_AGENT_ID:-}"

if [ -z "$ORCHESTRATOR_ID" ]; then
  echo "ERROR: cannot resolve orchestrator identity." >&2
  echo "  Set TERMLINK_AGENT_ID, pass --orchestrator-id, or run '/be-reachable start'." >&2
  exit 2
fi

# ── --live safety gate ──────────────────────────────────────────────────────
if [ "$MODE" = "live" ]; then
  if ! command -v termlink >/dev/null 2>&1; then
    echo "ERROR: --live requires termlink on PATH." >&2
    exit 2
  fi
fi

echo "# orchestrator-backlog-drain.sh — T-2204"
echo "# mode=$MODE capability=$CAPABILITY queue_topic=$QUEUE_TOPIC limit=$LIMIT per_worker_max=$PER_WORKER_MAX"
echo "# orchestrator=$ORCHESTRATOR_ID"
echo

# ── Step 1: governor pre-flight (informational) ─────────────────────────────
echo "# Step 1: governor pre-flight (#10)"
if command -v termlink >/dev/null 2>&1; then
  GOV_JSON="$(timeout 5 termlink hub status --governor --json 2>/dev/null || echo '{}')"
  CAP_HITS="$(echo "$GOV_JSON" | jq -r '.governor.capacity_hits_total // "?"' 2>/dev/null || echo "?")"
  RATE_HITS="$(echo "$GOV_JSON" | jq -r '.governor.rate_hits_total // "?"' 2>/dev/null || echo "?")"
  CONN_ACT="$(echo "$GOV_JSON" | jq -r '.governor.connections_active // "?"' 2>/dev/null || echo "?")"
  CONN_MAX="$(echo "$GOV_JSON" | jq -r '.governor.connections_max // "?"' 2>/dev/null || echo "?")"
  echo "#   conn_active=$CONN_ACT/$CONN_MAX cap_hits_total=$CAP_HITS rate_hits_total=$RATE_HITS"
  if [ "$MODE" = "live" ] && [ "$CAP_HITS" != "?" ] && [ "$CAP_HITS" -gt 0 ] 2>/dev/null; then
    echo "#   WARN: capacity_hits > 0 — hub has refused connections; live dispatch may fail." >&2
  fi
else
  echo "#   termlink not on PATH — skipping governor check"
fi
echo

# ── Step 2: enumerate agent-eligible work-units ─────────────────────────────
echo "# Step 2: enumerate agent-eligible work-units from .tasks/active/"
WORK_UNITS="$(python3 - <<'PY'
import os, re, yaml
ACTIVE = ".tasks/active"
out = []
if not os.path.isdir(ACTIVE):
    raise SystemExit(0)
for f in sorted(os.listdir(ACTIVE)):
    if not f.endswith(".md"): continue
    p = os.path.join(ACTIVE, f)
    try:
        with open(p, encoding="utf-8", errors="replace") as fh:
            content = fh.read()
    except OSError:
        continue
    m = re.match(r"^---\n(.*?)\n---", content, re.DOTALL)
    if not m: continue
    try: fm = yaml.safe_load(m.group(1)) or {}
    except Exception: continue
    tid     = fm.get("id","?")
    owner   = (fm.get("owner") or "").lower()
    horizon = (fm.get("horizon") or "now").lower()
    status  = (fm.get("status") or "").lower()
    wtype   = (fm.get("workflow_type") or "").lower()
    if owner == "human": continue
    if horizon == "later": continue
    if status not in ("captured", "started-work"): continue
    if wtype == "inception": continue
    # Count unchecked non-DEFERRED Agent ACs
    unchecked_agent = 0
    in_ac, in_human = False, False
    for line in content.split("\n"):
        if line.startswith("## Acceptance Criteria"): in_ac = True; continue
        if in_ac and line.startswith("## "): break
        if in_ac and line.startswith("### Human"): in_human = True; continue
        if in_ac and line.startswith("### "): in_human = False; continue
        if in_ac and not in_human and re.match(r"^- \[ \]", line):
            if not re.match(r"^- \[ \]\s+\*\*(DEFERRED|Deferred)", line):
                unchecked_agent += 1
    classification = "closure-ready" if unchecked_agent == 0 else "needs-work"
    name = (fm.get("name") or "").replace("|"," ")[:90]
    out.append(f"{tid}|{classification}|{unchecked_agent}|{name}")
print("\n".join(out))
PY
)"

TOTAL_UNITS="$(echo "$WORK_UNITS" | awk 'NF>0' | wc -l | tr -d ' ')"
echo "#   found $TOTAL_UNITS agent-eligible units"
if [ "$TOTAL_UNITS" = "0" ]; then
  echo
  echo "# nothing to dispatch — backlog is drained."
  exit 0
fi
echo

# ── Step 3: discover idle workers (#2) ──────────────────────────────────────
echo "# Step 3: discover idle workers via find-idle (#2)"
IDLE_JSON="$(timeout 5 termlink agent find-idle --capability "$CAPABILITY" --json 2>/dev/null || echo '{"idle":[]}')"
mapfile -t IDLE_AGENTS < <(
  echo "$IDLE_JSON" | jq -r '.idle[]?.agent_id' 2>/dev/null \
    | awk -v self="$ORCHESTRATOR_ID" '$0 != self && length($0) > 0'
)
IDLE_COUNT="${#IDLE_AGENTS[@]}"
echo "#   capability=$CAPABILITY  idle_workers=$IDLE_COUNT (excluding self=$ORCHESTRATOR_ID)"
if [ "$IDLE_COUNT" -gt 0 ]; then
  for w in "${IDLE_AGENTS[@]}"; do echo "#     - $w"; done
else
  echo "#   no idle workers — dispatch lines will use 'no-idle-worker' placeholder"
fi
echo

# ── Step 4: pair-and-dispatch ───────────────────────────────────────────────
echo "# Step 4: pair-and-dispatch"
declare -A worker_load
i=0
dispatched=0
no_worker=0
failures=0
emitted=0

while IFS='|' read -r tid classification ac_count name; do
  [ -z "$tid" ] && continue
  i=$((i+1))
  emitted=$((emitted+1))
  if [ "$emitted" -gt "$LIMIT" ]; then
    echo "# limit reached at $LIMIT — stopping enumeration"
    break
  fi

  # Pick a worker round-robin, skipping any that are already at PER_WORKER_MAX
  target="no-idle-worker"
  if [ "$IDLE_COUNT" -gt 0 ]; then
    for offset in $(seq 0 $((IDLE_COUNT-1))); do
      idx=$(( (i - 1 + offset) % IDLE_COUNT ))
      cand="${IDLE_AGENTS[$idx]}"
      cur="${worker_load[$cand]:-0}"
      if [ "$cur" -lt "$PER_WORKER_MAX" ]; then
        target="$cand"
        worker_load["$cand"]=$((cur+1))
        break
      fi
    done
  fi

  unit_payload="$(printf '{"task_id":"%s","classification":"%s","ac_count":%s,"dispatched_by":"%s"}' \
                  "$tid" "$classification" "$ac_count" "$ORCHESTRATOR_ID")"

  if [ "$classification" = "closure-ready" ]; then
    work_brief="Run the task's ## Verification block. If all pass, commit with 'T-XXX: …', then 'fw task update $tid --status work-completed'."
  else
    work_brief="$ac_count unchecked Agent AC(s). Read .tasks/active/$tid-*.md, satisfy each AC, commit, then 'fw task update $tid --status work-completed'."
  fi
  dm_body="T-2204 dispatch [$classification] $tid — $name. $work_brief Reply when done (release the claim)."

  if [ "$target" = "no-idle-worker" ]; then
    no_worker=$((no_worker+1))
    echo "DISPATCH [SKIP] worker=no-idle-worker unit=$tid classification=$classification ac_count=$ac_count"
    echo "         claim_payload=$unit_payload"
    echo "         dm_body=\"$dm_body\""
    continue
  fi

  if [ "$MODE" = "dry-run" ]; then
    dispatched=$((dispatched+1))
    echo "DISPATCH [DRY-RUN] worker=$target unit=$tid classification=$classification ac_count=$ac_count"
    echo "         claim_payload=$unit_payload"
    echo "         dm_body=\"$dm_body\""
    continue
  fi

  # ── LIVE PATH ────────────────────────────────────────────────────────────
  # 4a: post the unit to the work-queue topic
  post_resp="$(termlink channel post "$QUEUE_TOPIC" --payload "$unit_payload" --json 2>&1 || echo '{"error":"post-failed"}')"
  queue_offset="$(echo "$post_resp" | jq -r '.offset // empty' 2>/dev/null || true)"
  if [ -z "$queue_offset" ]; then
    failures=$((failures+1))
    echo "DISPATCH [POST-FAIL] worker=$target unit=$tid response=$post_resp"
    continue
  fi

  # 4b: claim the offset as orchestrator (TTL 10min — plenty for transfer + DM)
  claim_resp="$(termlink channel claim "$QUEUE_TOPIC" "$queue_offset" \
                  --claimer "$ORCHESTRATOR_ID" --ttl-ms 600000 --json 2>&1 \
                  || echo '{"error":"claim-failed"}')"
  claim_id="$(echo "$claim_resp" | jq -r '.claim_id // empty' 2>/dev/null || true)"
  if [ -z "$claim_id" ]; then
    failures=$((failures+1))
    echo "DISPATCH [CLAIM-FAIL] worker=$target unit=$tid offset=$queue_offset response=$claim_resp"
    continue
  fi

  # 4c: atomic claim-transfer to worker (T-2046 #3 — no race window)
  xfer_resp="$(termlink channel claim-transfer "$claim_id" "$target" \
                  --by "$ORCHESTRATOR_ID" --reason "T-2204 dispatch of $tid" --json 2>&1 \
                  || echo '{"ok":false}')"
  xfer_ok="$(echo "$xfer_resp" | jq -r '.ok // false' 2>/dev/null || echo "false")"
  if [ "$xfer_ok" != "true" ]; then
    failures=$((failures+1))
    echo "DISPATCH [TRANSFER-FAIL] worker=$target unit=$tid claim_id=$claim_id response=$xfer_resp"
    continue
  fi

  # 4d: DM the worker with the brief (fire-and-forget — claim is what's load-bearing)
  if [ -x scripts/agent-send.sh ]; then
    bash scripts/agent-send.sh --to "$target" \
      --message "$dm_body (claim_id=$claim_id offset=$queue_offset topic=$QUEUE_TOPIC)" \
      >/dev/null 2>&1 || true
  fi

  dispatched=$((dispatched+1))
  echo "DISPATCH [LIVE] worker=$target unit=$tid claim_id=$claim_id offset=$queue_offset classification=$classification"
done <<< "$WORK_UNITS"

# ── Summary ────────────────────────────────────────────────────────────────
echo
echo "# Summary: total=$TOTAL_UNITS dispatched=$dispatched no_worker=$no_worker failures=$failures mode=$MODE"

if [ "$MODE" = "live" ] && [ "$failures" -gt 0 ]; then
  exit 1
fi
exit 0
