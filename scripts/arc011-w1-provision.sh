#!/usr/bin/env bash
# T-3438 — arc-011 PH-2 wave W1: provision and start the W1 pair's sidecars
# (operator rulings P7-1 pair, P7-2 key register, P7-3 keep running, P7-4
# provisioned instance ids; OQ-f D+). Called by /opt/termlink/runme.sh.
#
#   --check   read-only readiness: key files present, register consistent, hub
#             reachable with an id, the installed termlink has the sidecar verbs.
#             Exit 0 ready, 1 not ready (each reason printed), 2 tooling.
#   --apply   idempotent provisioning, then start, then verify:
#               1. key register  /etc/termlink-sidecar/keys/<agent>.pub (0644)
#                  — written if absent; a DIFFERENT existing key is FAILED
#                  (a key changes only through a runme action naming it)
#               2. instance id   /etc/termlink-sidecar/instances/<agent>
#                  — minted once as <agent>.<stamp>, never rewritten
#               3. store root    /var/lib/termlink-sidecar/<agent>/ (0700) with
#                  the production marker .termlink-sidecar-root (0644)
#               4. config        /etc/termlink-sidecar/<agent>.json (roster = the
#                  partner's key; book = partner -> its instance; main pinned)
#               5. settings      /etc/termlink-sidecar/<project>--<agent>.env
#               6. admin list    <project>/.context/working/sidecar-admin-sockets.list
#               7. setting       SIDECAR_KEEP_RUNNING: on in <project>/.framework.yaml
#               8. start         sidecar-supervise.sh start <project>--<agent>
#               9. verify        each sidecar answers status in production mode
#                  with a fresh heartbeat and the expected supervisor
#             Prints OK / skip / FAILED per step. Exit 0 all good, 1 any FAILED.
#
# Seams (fixtures): W1_PROJECT, W1_PROJECT_DIR, W1_PAIR, W1_MAIN, W1_STAMP,
#   W1_IDENT_DIR, W1_ETC, W1_VARLIB, W1_HUB_RT, W1_BIN, W1_HOME, W1_SUPERVISE,
#   W1_SUPERVISE_ENV (extra env for the supervise call), W1_ADMIN_LIST,
#   W1_WAIT_SECS, W1_EXPECT_SUPERVISOR.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT="${W1_PROJECT:-010-termlink}"
PDIR="${W1_PROJECT_DIR:-$ROOT}"
PAIR="${W1_PAIR:-claude-termlink claude-termlink-alt}"
MAIN="${W1_MAIN:-claude-termlink}"
IDENT="${W1_IDENT_DIR:-${HOME:-/root}/.termlink/identities}"
ETC="${W1_ETC:-/etc/termlink-sidecar}"
VARLIB="${W1_VARLIB:-/var/lib/termlink-sidecar}"
HUB_RT="${W1_HUB_RT:-/var/lib/termlink}"
BIN="${W1_BIN:-$(command -v termlink || echo /root/.cargo/bin/termlink)}"
SHOME="${W1_HOME:-${HOME:-/root}}"
SUPERVISE="${W1_SUPERVISE:-$ROOT/scripts/sidecar-supervise.sh}"
ADMIN_LIST="${W1_ADMIN_LIST:-$PDIR/.context/working/sidecar-admin-sockets.list}"
WAIT="${W1_WAIT_SECS:-30}"
EXPECT_SUP="${W1_EXPECT_SUPERVISOR:-systemd}"
MAGIC="termlink-sidecar-production v1"
FAILED=0

ok()   { echo "  OK      $*"; }
skip() { echo "  skip    $*"; }
bad()  { echo "  FAILED  $*"; FAILED=$((FAILED + 1)); }

pubkey() {  # <key file> -> 64 hex, or nothing
    TERMLINK_IDENTITY_FILE="$1" "$BIN" identity show --resolve --no-create --json 2>/dev/null |
        python3 -c 'import json,sys; print(json.load(sys.stdin).get("public_key_hex",""))' 2>/dev/null
}
hub_id() { "$BIN" sidecar hub-id --hub-socket "$HUB_RT/hub.sock" 2>/dev/null; }

check() {
    local a k p reg rc=0 hid
    [ -x "$BIN" ] || { echo "  NOT READY  termlink binary not found at $BIN"; return 2; }
    local usage; usage=$("$BIN" sidecar 2>&1)   # usage exits 2; captured, never piped (pipefail)
    grep -q 'hub-id' <<< "$usage" || { echo "  NOT READY  $BIN has no 'sidecar hub-id' verb (install the current build first)"; rc=1; }
    hid=$(hub_id)
    [ -n "$hid" ] && echo "  ready      hub $HUB_RT/hub.sock answers with id $hid" || { echo "  NOT READY  hub at $HUB_RT/hub.sock does not answer with an id"; rc=1; }
    for a in $PAIR; do
        k="$IDENT/$a.key"
        if [ ! -f "$k" ]; then echo "  NOT READY  $a: key file $k is missing (W1 never mints a key)"; rc=1; continue; fi
        p=$(pubkey "$k")
        [ ${#p} -eq 64 ] || { echo "  NOT READY  $a: cannot read the public key of $k"; rc=1; continue; }
        reg="$ETC/keys/$a.pub"
        if [ -f "$reg" ] && [ "$(tr -d '[:space:]' < "$reg")" != "$p" ]; then
            echo "  NOT READY  $a: $reg holds a different key than $k (a key change needs its own named runme action)"; rc=1
        else
            echo "  ready      $a: key ${p:0:16}…${reg:+ (register $([ -f "$reg" ] && echo matches || echo to be written))}"
        fi
    done
    return $rc
}

write_if_changed() {  # <path> <mode> <content> <label>
    # Byte-for-byte (cmp), never $(cat …): command substitution strips trailing
    # newlines, so a file ending in one would never compare equal.
    local path="$1" mode="$2" content="$3" label="$4" tmp
    if [ -f "$path" ] && cmp -s <(printf '%s' "$content") "$path"; then skip "$label unchanged: $path"; return 0; fi
    tmp="$path.tmp.$$"
    printf '%s' "$content" > "$tmp" && chmod "$mode" "$tmp" && mv -f "$tmp" "$path" || { bad "$label: cannot write $path"; return 1; }
    cmp -s <(printf '%s' "$content") "$path" && ok "$label written and verified: $path" || bad "$label: $path does not read back"
}

apply() {
    local a k p reg inst hid peer
    declare -A PUB INST
    check >/dev/null || { echo "  FAILED  not ready; run the check:"; check; FAILED=$((FAILED + 1)); return; }
    hid=$(hub_id)
    install -d -m 0755 "$ETC" "$ETC/keys" "$ETC/instances" || { bad "cannot create $ETC"; return; }
    install -d -m 0755 "$VARLIB" || { bad "cannot create $VARLIB"; return; }
    local stamp="${W1_STAMP:-w1-$(date -u +%Y%m%d)}"
    # 1-3: register, instance, store + marker
    for a in $PAIR; do
        k="$IDENT/$a.key"; p=$(pubkey "$k"); PUB[$a]="$p"
        reg="$ETC/keys/$a.pub"
        if [ -f "$reg" ]; then
            [ "$(tr -d '[:space:]' < "$reg")" = "$p" ] && skip "$a: key register matches" || { bad "$a: $reg differs from $k; not changed"; continue; }
        else
            write_if_changed "$reg" 0644 "$p" "$a key register"
        fi
        inst="$ETC/instances/$a"
        if [ -s "$inst" ]; then skip "$a: instance id kept ($(cat "$inst"))"
        else write_if_changed "$inst" 0644 "$a.$stamp" "$a instance id"; fi
        INST[$a]="$(cat "$inst" 2>/dev/null)"
        install -d -m 0700 "$VARLIB/$a" || bad "$a: cannot create $VARLIB/$a"
        write_if_changed "$VARLIB/$a/.termlink-sidecar-root" 0644 "$MAGIC
" "$a production marker"
    done
    [ "$FAILED" = 0 ] || return
    # 4-5: config and settings, each needs the partner's key and instance
    for a in $PAIR; do
        local roster="" peers="" sep=""
        for peer in $PAIR; do
            [ "$peer" = "$a" ] && continue
            roster="$roster$sep\"${PUB[$peer]}\""; peers="$peers$sep\"$peer\": \"${INST[$peer]}\""; sep=", "
        done
        local cfg
        cfg=$(python3 - "$a" "${INST[$a]}" "$HUB_RT/hub.sock" "$hid" "$IDENT/$a.key" "$VARLIB/$a/outbox.sqlite" "$ETC/keys/$a.pub" "$MAIN" "[$roster]" "{$peers}" <<'EOF'
import json, sys
a, inst, sock, hid, key, queue, reg, main, roster, peers = sys.argv[1:11]
print(json.dumps({
    "agent": a, "instance": inst,
    "hub": {"type": "unix", "path": sock}, "hub_id": hid,
    "key_file": key, "roster": json.loads(roster), "queue_path": queue,
    "key_register": reg,
    "book": {"peers": json.loads(peers), "main": main},
}, indent=2, sort_keys=True))
EOF
)
        [ -n "$cfg" ] || { bad "$a: cannot build the config"; continue; }
        write_if_changed "$ETC/$a.json" 0644 "$cfg" "$a sidecar config"
        write_if_changed "$ETC/$PROJECT--$a.env" 0644 "# Written by scripts/arc011-w1-provision.sh (T-3438, arc-011 W1)
PROJECT_DIR=$PDIR
AGENT=$a
SIDECAR_CONFIG=$ETC/$a.json
SIDECAR_BIN=$BIN
TERMLINK_SIDECAR_STORE=$VARLIB/$a
TERMLINK_RUNTIME_DIR=$HUB_RT
HOME=$SHOME
" "$a settings"
    done
    # 6: admin-socket list (the fleet kill switch and the alerts read it)
    local list="" ; for a in $PAIR; do list="$list$HUB_RT/sidecar/$a.admin.sock
"; done
    write_if_changed "$ADMIN_LIST" 0644 "$list" "admin-socket list"
    # 7: the project setting (P7-3 B)
    local y="$PDIR/.framework.yaml"
    if grep -qE '^SIDECAR_KEEP_RUNNING:[[:space:]]*(on|true|yes|1)[[:space:]]*$' "$y" 2>/dev/null; then skip "SIDECAR_KEEP_RUNNING already on in $y"
    elif grep -q '^SIDECAR_KEEP_RUNNING:' "$y" 2>/dev/null; then
        sed -i 's/^SIDECAR_KEEP_RUNNING:.*/SIDECAR_KEEP_RUNNING: on/' "$y" && grep -q '^SIDECAR_KEEP_RUNNING: on$' "$y" && ok "SIDECAR_KEEP_RUNNING set on in $y" || bad "cannot set SIDECAR_KEEP_RUNNING in $y"
    else
        printf '\n# arc-011 OQ-f D+ / P7-3: keep this project'"'"'s sidecars running under supervision\nSIDECAR_KEEP_RUNNING: on\n' >> "$y" && grep -q '^SIDECAR_KEEP_RUNNING: on$' "$y" && ok "SIDECAR_KEEP_RUNNING added (on) to $y" || bad "cannot add SIDECAR_KEEP_RUNNING to $y"
    fi
    [ "$FAILED" = 0 ] || return
    # 8-9: start and verify
    for a in $PAIR; do
        if env ${W1_SUPERVISE_ENV:-} bash "$SUPERVISE" start "$PROJECT--$a" >/tmp/.w1-start-"$a".log 2>&1; then :; else
            bad "$a: sidecar did not start (see /tmp/.w1-start-$a.log)"; continue
        fi
        local hb="$VARLIB/$a/sidecar/$a/heartbeat.json" i good=0
        for ((i = 0; i < WAIT; i++)); do
            if [ -r "$hb" ] && python3 - "$hb" "$EXPECT_SUP" <<'EOF'
import json, sys, time
h = json.load(open(sys.argv[1]))
sys.exit(0 if time.time() * 1000 - h["ts_ms"] < 120000 and h["supervisor"] == sys.argv[2] and h["keep_running"] == "on" and not h["stale"] else 1)
EOF
            then good=1; break; fi
            sleep 1
        done
        [ "$good" = 1 ] || { bad "$a: no fresh heartbeat under supervisor $EXPECT_SUP within ${WAIT}s ($hb)"; continue; }
        local st
        st=$(TERMLINK_SIDECAR_STATUS_SOCKET="$HUB_RT/sidecar/$a.status.sock" "$BIN" sidecar status --json 2>/dev/null)
        if python3 -c 'import json,sys; d=json.loads(sys.argv[1]); sys.exit(0 if d.get("mode")=="production" and d.get("agent")==sys.argv[2] else 1)' "$st" "$a" 2>/dev/null; then
            ok "$a: running in production mode under $EXPECT_SUP, heartbeat fresh, instance ${INST[$a]}"
        else
            bad "$a: status does not answer in production mode (got: ${st:0:160})"
        fi
    done
}

case "${1:-}" in
    --check) check; exit $? ;;
    --apply) apply; [ "$FAILED" = 0 ]; exit $? ;;
    *) sed -n '2,32p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
