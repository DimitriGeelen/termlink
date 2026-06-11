#!/usr/bin/env bash
# T-2146 — canonical substrate worker loop (hello-world for substrate users).
#
# Walks the T-2018 §6 #1 CLAIM primitive lifecycle for one work unit:
#
#     claim → run worker cmd (with background auto-renew) → release
#
# Composition of existing substrate verbs:
#   - termlink channel claim   (T-2032, substrate primitive #1)
#   - termlink channel renew   (T-2030, substrate primitive #1)
#   - termlink channel release (T-2032, substrate primitive #1)
#
# Read-side observability is unchanged — pair with /claims (T-2093),
# /queue-status (T-2094), and /governor (T-2095) to inspect the work
# in flight. See docs/operations/substrate-orchestrator-recipe.md for
# the master pattern this script implements.
#
# Usage:
#   substrate-worker-loop.sh --topic T --offset N --cmd 'COMMAND'
#                            [--claimer ID] [--ttl-ms N] [--renew-every-ms N]
#                            [--hub addr]
#
# Lifecycle:
#   1. claim(topic, offset, ttl_ms, claimer)
#      → fail (CLAIM_ALREADY_HELD, AUTH_FAIL, RATE_LIMITED, ...)
#        → exit 11, no work attempted, no claim held (loud-fail)
#   2. spawn background auto-renew loop (every renew_every_ms while
#      worker is alive). Each renew extends the lease by ttl_ms.
#      A CLAIM_LAPSED here is loud-but-not-fatal to the worker — the
#      worker may still finish in time; the release step will then
#      fail and we report that.
#   3. run --cmd. The worker sees TERMLINK_CLAIM_ID, TERMLINK_CLAIM_TOPIC,
#      TERMLINK_CLAIM_OFFSET, TERMLINK_CLAIMER in its env so it can
#      identify itself if it needs to post follow-ups on the substrate.
#   4. on worker exit:
#        cmd exit 0   → release(claim_id, ack=true)  — work completed
#        cmd exit !=0 → release(claim_id, ack=false) — slot reopens
#   5. on SIGTERM/SIGINT to wrapper → kill worker + release-without-ack,
#      exit 130 (so an outer supervisor can distinguish "killed mid-work"
#      from "worker exited nonzero on its own").
#
# Exit codes:
#   0   worker exited 0, release(ack=true) succeeded — work done
#   1   worker exited non-zero, release(ack=false) — slot reopened
#   2   usage / missing flag
#   4   preflight refused start — TERMLINK_RUNTIME_DIR on /tmp etc. (T-2163)
#   11  claim failed (CLAIM_ALREADY_HELD, AUTH_FAIL, ...) — no work attempted
#   12  release failed after worker — worker's own exit is masked by 12
#   130 SIGTERM/SIGINT during work (release-without-ack attempted)
#
# Adapt this script: change the `--cmd` invocation to whatever your
# worker actually does. The lifecycle wiring around it is what matters.

set -u

# ---- Configuration -------------------------------------------------------

TERMLINK="${TERMLINK_BIN:-termlink}"
STATE_DIR="${TERMLINK_STATE_DIR:-${HOME}/.termlink}"
BE_REACHABLE_STATE="${STATE_DIR}/be-reachable.state"

TOPIC=""
OFFSET=""
CMD=""
CLAIMER=""
TTL_MS=30000
RENEW_EVERY_MS=""
HUB=""
ADOPT_CLAIM_ID=""
SKIP_PREFLIGHT=0

# ---- Helpers -------------------------------------------------------------

usage() {
    cat <<'EOF'
Usage: substrate-worker-loop.sh --topic T --offset N --cmd 'COMMAND' [options]

Required:
  --topic T              Topic to claim on (must exist — `channel create` first)
  --offset N             Offset within the topic to exclusively claim
  --cmd 'COMMAND'        Shell command to run while the claim is held.
                         Receives TERMLINK_CLAIM_ID, TERMLINK_CLAIM_TOPIC,
                         TERMLINK_CLAIM_OFFSET, TERMLINK_CLAIMER in env.

Options:
  --claimer ID           Worker identity. Default: $TERMLINK_AGENT_ID env,
                         then $HOME/.termlink/be-reachable.state (T-1841).
                         Refuses if neither is set — no implicit claimer.
  --claim-id ID          ADOPTED-CLAIM mode (T-2150): skip the self-claim
                         step and adopt an existing claim_id (typically
                         transferred from an orchestrator via
                         channel claim-transfer). The script still runs
                         auto-renew + work + release against this claim.
                         Without this flag (default): STANDALONE mode —
                         worker claims the offset itself. Use the
                         adopted-claim form when paired with
                         substrate-orchestrator-loop.sh (T-2148).
  --ttl-ms N             Lease length per claim/renew, ms. Default 30000.
                         Hub clamps to 1h max.
  --renew-every-ms N     Auto-renew cadence, ms. Default ttl_ms/2.
                         Smaller is safer but more chatty.
  --hub addr             Target hub. Default: local hub.
  --skip-preflight       Skip the startup substrate-preflight.sh call
                         (CI/test paths where preflight is already known
                         clean). T-2163. Default: run preflight.
  -h, --help             Print this help and exit 0.

Exit codes:
  0    worker exit 0  + release(--ack) ok       — work done
  1    worker exit !=0 + release(no --ack) ok    — slot reopened
  2    usage error
  4    preflight refused start (T-2163, e.g. TERMLINK_RUNTIME_DIR on /tmp)
  11   claim refused (held / auth / rate-limit)  — no work attempted
  12   release failed after worker — work outcome masked
  130  SIGTERM/SIGINT mid-work; release-without-ack attempted

Examples:
  # Treat offset 42 on work-queue as work to do; run a python worker:
  substrate-worker-loop.sh --topic work-queue --offset 42 \
                           --cmd 'python3 /opt/myapp/process.py 42'

  # Tight loop with explicit identity + 60s lease + 20s renew cadence:
  substrate-worker-loop.sh --topic aef:deploy --offset 7 \
                           --claimer worker-alpha --ttl-ms 60000 \
                           --renew-every-ms 20000 \
                           --cmd '/opt/aef/deploy.sh aef:deploy 7'

See: docs/operations/substrate-orchestrator-recipe.md (T-2124 master recipe)
EOF
}

die() {
    # one-line stderr + exit
    echo "substrate-worker-loop.sh: $1" >&2
    exit "${2:-2}"
}

resolve_claimer() {
    # CLI flag wins; then env; then be-reachable.state; else refuse.
    if [ -n "$CLAIMER" ]; then
        return 0
    fi
    if [ -n "${TERMLINK_AGENT_ID:-}" ]; then
        CLAIMER="$TERMLINK_AGENT_ID"
        return 0
    fi
    if [ -r "$BE_REACHABLE_STATE" ]; then
        local id
        id=$(grep -o '"agent_id"[[:space:]]*:[[:space:]]*"[^"]*"' "$BE_REACHABLE_STATE" \
             | sed 's/.*"agent_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
        if [ -n "$id" ]; then
            CLAIMER="$id"
            return 0
        fi
    fi
    die "claimer unresolved — pass --claimer, set \$TERMLINK_AGENT_ID, or run /be-reachable first" 2
}

extract_claim_id() {
    # Parse claim_id out of `termlink channel claim --json` output.
    # We use grep+sed instead of jq so the script has no hard runtime dep.
    grep -o '"claim_id"[[:space:]]*:[[:space:]]*"[^"]*"' \
        | sed 's/.*"claim_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/' \
        | head -n1
}

run_preflight() {
    # T-2163: run substrate-preflight.sh as a startup gate.
    #   exit 0 (PASS) → silent, continue
    #   exit 1 (WARN) → print body + WARN line to stderr, continue
    #   exit 2 (FAIL) → print body to stderr, refuse to start (exit 4)
    # Locates sibling script via $0 dirname so install-location doesn't matter.
    # Missing preflight script → warn + continue (defensive — don't block on
    # absent tooling; the canary covers the long-running drift case).
    [ "$SKIP_PREFLIGHT" -eq 1 ] && return 0

    local self_dir preflight pf_out pf_rc
    self_dir=$(cd "$(dirname "$0")" && pwd)
    preflight="$self_dir/substrate-preflight.sh"

    if [ ! -x "$preflight" ]; then
        echo "substrate-worker-loop.sh: preflight script not found at $preflight — continuing without check" >&2
        return 0
    fi

    pf_out=$("$preflight" 2>&1)
    pf_rc=$?

    case "$pf_rc" in
        0) return 0 ;;
        1)
            echo "substrate-worker-loop.sh: WARNING — substrate-preflight reported warnings:" >&2
            echo "$pf_out" >&2
            echo "substrate-worker-loop.sh: continuing despite WARN (use --skip-preflight to suppress)" >&2
            return 0
            ;;
        *)
            echo "substrate-worker-loop.sh: substrate-preflight FAILED (exit $pf_rc) — refusing to start:" >&2
            echo "$pf_out" >&2
            echo "substrate-worker-loop.sh: fix the failure above OR pass --skip-preflight if you accept the risk" >&2
            exit 4
            ;;
    esac
}

# ---- Arg parsing ---------------------------------------------------------

while [ $# -gt 0 ]; do
    case "$1" in
        --topic) TOPIC="$2"; shift 2 ;;
        --offset) OFFSET="$2"; shift 2 ;;
        --cmd) CMD="$2"; shift 2 ;;
        --claimer) CLAIMER="$2"; shift 2 ;;
        --claim-id) ADOPT_CLAIM_ID="$2"; shift 2 ;;
        --ttl-ms) TTL_MS="$2"; shift 2 ;;
        --renew-every-ms) RENEW_EVERY_MS="$2"; shift 2 ;;
        --hub) HUB="$2"; shift 2 ;;
        --skip-preflight) SKIP_PREFLIGHT=1; shift ;;
        -h|--help) usage; exit 0 ;;
        --) shift; break ;;
        *) die "unknown flag: $1" 2 ;;
    esac
done

[ -n "$TOPIC" ] || die "Usage: --topic required (see --help)" 2
[ -n "$OFFSET" ] || die "Usage: --offset required (see --help)" 2
[ -n "$CMD" ] || die "Usage: --cmd required (see --help)" 2

# Default renew cadence = ttl/2 so we have one safety margin before lapse.
if [ -z "$RENEW_EVERY_MS" ]; then
    RENEW_EVERY_MS=$(( TTL_MS / 2 ))
    [ "$RENEW_EVERY_MS" -gt 0 ] || RENEW_EVERY_MS=1000
fi

resolve_claimer

HUB_ARGS=()
if [ -n "$HUB" ]; then
    HUB_ARGS=(--hub "$HUB")
fi

# ---- Step 0 — preflight gate (T-2163) -----------------------------------
# Catch PL-021 (volatile /tmp) before any hub call. Silent on PASS; warn
# and continue on WARN; refuse to start on FAIL (exit 4). Bypass via
# --skip-preflight for CI/test paths where preflight is already clean.
run_preflight

# ---- Step 1 — claim (or adopt existing) ----------------------------------

if [ -n "$ADOPT_CLAIM_ID" ]; then
    # ADOPTED-CLAIM mode (T-2150): orchestrator already claimed + transferred
    # ownership to this worker. Skip the channel.claim call (it would fail
    # CLAIM_ALREADY_HELD since the worker IS the current holder) and use the
    # passed claim_id for the rest of the lifecycle.
    CLAIM_ID="$ADOPT_CLAIM_ID"
    echo "substrate-worker-loop.sh: adopting existing claim_id=$CLAIM_ID topic=$TOPIC offset=$OFFSET claimer=$CLAIMER" >&2
else
    echo "substrate-worker-loop.sh: claiming topic=$TOPIC offset=$OFFSET claimer=$CLAIMER ttl=${TTL_MS}ms" >&2

    CLAIM_OUT=$("$TERMLINK" channel claim --json "${HUB_ARGS[@]}" \
                --claimer "$CLAIMER" --ttl-ms "$TTL_MS" \
                "$TOPIC" "$OFFSET" 2>&1) || {
        echo "$CLAIM_OUT" >&2
        die "claim failed — slot still held by someone else, or auth/rate-limit (exit 11)" 11
    }

    CLAIM_ID=$(echo "$CLAIM_OUT" | extract_claim_id)
    [ -n "$CLAIM_ID" ] || {
        echo "$CLAIM_OUT" >&2
        die "claim returned no claim_id — malformed response (exit 11)" 11
    }

    echo "substrate-worker-loop.sh: claim_id=$CLAIM_ID — running worker" >&2
fi

# ---- Step 2 — background auto-renew --------------------------------------

# Convert ms→seconds for `sleep` (bash sleep supports fractional).
RENEW_EVERY_S=$(awk -v ms="$RENEW_EVERY_MS" 'BEGIN { printf "%.3f", ms/1000 }')

(
    # Quiet auto-renew — failures land on stderr but don't propagate up
    # (the main path's release will surface CLAIM_LAPSED if it matters).
    while true; do
        sleep "$RENEW_EVERY_S"
        "$TERMLINK" channel renew --json "${HUB_ARGS[@]}" \
            --claim-id "$CLAIM_ID" --claimer "$CLAIMER" \
            --additional-ttl-ms "$TTL_MS" >/dev/null 2>&1 || {
            echo "substrate-worker-loop.sh: renew failed (claim may have lapsed) — continuing" >&2
            # Don't exit the renew loop — worker might still be in time.
        }
    done
) &
RENEW_PID=$!

cleanup_renew() {
    if [ -n "${RENEW_PID:-}" ] && kill -0 "$RENEW_PID" 2>/dev/null; then
        kill "$RENEW_PID" 2>/dev/null || true
        wait "$RENEW_PID" 2>/dev/null || true
    fi
}

# ---- Step 3 — run worker -------------------------------------------------

# Trap so SIGTERM/SIGINT during work releases-without-ack and exits 130.
WORKER_PID=""
trap '
    echo "substrate-worker-loop.sh: caught signal — releasing without ack" >&2
    cleanup_renew
    if [ -n "$WORKER_PID" ] && kill -0 "$WORKER_PID" 2>/dev/null; then
        kill "$WORKER_PID" 2>/dev/null || true
        wait "$WORKER_PID" 2>/dev/null || true
    fi
    "$TERMLINK" channel release --json "${HUB_ARGS[@]}" \
        --claim-id "$CLAIM_ID" --claimer "$CLAIMER" >/dev/null 2>&1 || true
    exit 130
' INT TERM

# Run worker in foreground but capture PID for trap.
TERMLINK_CLAIM_ID="$CLAIM_ID" \
TERMLINK_CLAIM_TOPIC="$TOPIC" \
TERMLINK_CLAIM_OFFSET="$OFFSET" \
TERMLINK_CLAIMER="$CLAIMER" \
    bash -c "$CMD" &
WORKER_PID=$!
wait "$WORKER_PID"
WORKER_EXIT=$?

cleanup_renew

# ---- Step 4 — release ----------------------------------------------------

if [ "$WORKER_EXIT" -eq 0 ]; then
    echo "substrate-worker-loop.sh: worker ok — release(--ack)" >&2
    RELEASE_OUT=$("$TERMLINK" channel release --json "${HUB_ARGS[@]}" \
                  --claim-id "$CLAIM_ID" --claimer "$CLAIMER" --ack 2>&1) || {
        echo "$RELEASE_OUT" >&2
        die "release(--ack) failed — work may have completed but cursor did not advance (exit 12)" 12
    }
    exit 0
else
    echo "substrate-worker-loop.sh: worker exit=$WORKER_EXIT — release (no --ack), slot reopens" >&2
    RELEASE_OUT=$("$TERMLINK" channel release --json "${HUB_ARGS[@]}" \
                  --claim-id "$CLAIM_ID" --claimer "$CLAIMER" 2>&1) || {
        echo "$RELEASE_OUT" >&2
        die "release failed after worker fail — slot may stay claimed until TTL lapse (exit 12)" 12
    }
    exit 1
fi
