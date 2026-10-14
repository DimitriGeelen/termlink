#!/bin/sh
# T-3442 (arc-011 P9, R-39.e, exit 5) — runs INSIDE a clean Alpine container:
# no /opt/termlink checkout, no Rust. Installs termlink from a local "release"
# over HTTP with the real install.sh, then proves the sidecar starts, delivers,
# survives an upgrade, a rollback and a SIGKILL, with the release checksum
# re-verified by status. Prints PASS/FAIL per step; exit 0 only if all pass.
# Driven by tests/arc011-clean-host.sh (which serves the release and runs this).
set -u
REL="$1"   # http://127.0.0.1:<port>
FAILS=0
pass() { echo "PASS $*"; }
fail() { echo "FAIL $*"; FAILS=$((FAILS + 1)); }
die() { echo "FAIL $*"; echo "RESULT: aborted"; exit 1; }

# The tools need bash and a ps that understands -o (procps). These are the
# clean host's prerequisites, installed here like an operator would.
apk add --no-cache bash procps >/dev/null 2>&1 || die "cannot install bash/procps"

[ ! -e /opt/termlink ] && pass "0a no /opt/termlink checkout" || fail "0a a checkout is present"
command -v cargo >/dev/null 2>&1 && fail "0b Rust is present" || pass "0b no Rust toolchain"

# 1 install v1 from the release
TERMLINK_INSTALL_BASE_URL="$REL/v1" sh /work/install.sh > /tmp/install1.log 2>&1 || { cat /tmp/install1.log; die "1 install v1 failed"; }
V1=$(sha256sum /usr/local/bin/termlink | cut -d' ' -f1)
grep -q "$V1" /srv/expect-v1.sha && pass "1a installed binary is the v1 artifact" || fail "1a installed binary is not v1"
[ -x /usr/local/share/termlink/sidecar/sidecar-supervise.sh ] && [ -f /usr/local/share/termlink/sidecar/termlink-sidecar@.service ] \
    && pass "1b sidecar tools installed from the binary" || fail "1b sidecar tools missing"
grep -v '^[[:space:]]*#' /usr/local/share/termlink/sidecar/termlink-sidecar@.service /usr/local/share/termlink/sidecar/sidecar-supervise.sh | grep -q /opt/termlink \
    && fail "1c tools point into a checkout" || pass "1c tools point at the install, not a checkout"
[ -f /usr/local/share/termlink/checksums.txt ] && pass "1d release checksum record kept" || fail "1d no checksum record"

# 2 a hub and two production sidecars, started from the deployment
export TERMLINK_RUNTIME_DIR=/var/lib/termlink
mkdir -p "$TERMLINK_RUNTIME_DIR"
# A hub has an id (FX-2, which every sidecar verifies) only with its TLS cert, which
# the TCP listener creates: an arc-011 host's hub runs with --tcp. 127.0.0.1 on a
# spare port (the container shares the host network).
setsid nohup termlink hub start --tcp 127.0.0.1:19199 > /tmp/hub.log 2>&1 < /dev/null &
i=0; while [ ! -S "$TERMLINK_RUNTIME_DIR/hub.sock" ] && [ $i -lt 60 ]; do sleep 0.5; i=$((i + 1)); done
HID=$(termlink sidecar hub-id --hub-socket "$TERMLINK_RUNTIME_DIR/hub.sock") || die "2 hub did not start: $(tail -3 /tmp/hub.log)"
pass "2a hub up, id $HID"
mkdir -p /etc/termlink-sidecar/keys /srv/proj /keys
echo "SIDECAR_KEEP_RUNNING: on" > /srv/proj/.framework.yaml
pub() { TERMLINK_IDENTITY_FILE="/keys/$1.key" termlink identity show --resolve --json | sed -n 's/.*"public_key_hex": *"\([0-9a-f]*\)".*/\1/p'; }
PA=$(pub agent-a); PB=$(pub agent-b)
[ ${#PA} -eq 64 ] && [ ${#PB} -eq 64 ] || die "2 cannot mint test keys"
for a in agent-a agent-b; do
    if [ "$a" = agent-a ]; then peer=agent-b; ppub=$PB; mypub=$PA; else peer=agent-a; ppub=$PA; mypub=$PB; fi
    st="/var/lib/termlink-sidecar/$a"; mkdir -p "$st"; chmod 700 "$st"
    printf 'termlink-sidecar-production v1\n' > "$st/.termlink-sidecar-root"; chmod 644 "$st/.termlink-sidecar-root"
    printf '%s' "$mypub" > "/etc/termlink-sidecar/keys/$a.pub"
    cat > "/etc/termlink-sidecar/$a.json" <<EOF
{"agent":"$a","instance":"$a.ch-1","hub":{"type":"unix","path":"$TERMLINK_RUNTIME_DIR/hub.sock"},"hub_id":"$HID",
 "key_file":"/keys/$a.key","roster":["$ppub"],"queue_path":"$st/outbox.sqlite",
 "key_register":"/etc/termlink-sidecar/keys/$a.pub","book":{"peers":{"$peer":"$peer.ch-1"},"main":"agent-a"}}
EOF
    cat > "/etc/termlink-sidecar/test--$a.env" <<EOF
PROJECT_DIR=/srv/proj
AGENT=$a
SIDECAR_CONFIG=/etc/termlink-sidecar/$a.json
SIDECAR_BIN=/usr/local/bin/termlink
TERMLINK_SIDECAR_STORE=$st
TERMLINK_RUNTIME_DIR=$TERMLINK_RUNTIME_DIR
HOME=/root
EOF
done
SUP=/usr/local/share/termlink/sidecar/sidecar-supervise.sh
export SIDECAR_SYSTEMCTL=none SIDECAR_BACKSTOP_HEARTBEAT_FILE=/tmp/backstop.hb
startall() { for a in agent-a agent-b; do bash "$SUP" start "test--$a" >> /tmp/sup.log 2>&1 || return 1; done; }
stopall() { bash "$SUP" stop-project test >> /tmp/sup.log 2>&1; }
startall && pass "2b both sidecars started from the installed tools (cron supervises: no systemd)" || { cat /tmp/sup.log; die "2b sidecars did not start"; }
status() { a="$1"; shift; TERMLINK_SIDECAR_STATUS_SOCKET="$TERMLINK_RUNTIME_DIR/sidecar/$a.status.sock" termlink sidecar status --json "$@" 2>/dev/null; }

# 3 delivery between them, and SI-27 in status
MSG=$(TERMLINK_SIDECAR_SOCKET="$TERMLINK_RUNTIME_DIR/sidecar/agent-a.send.sock" termlink sidecar send --to agent-b --body "clean host" | sed -n 's/.*"msg_id":"\([0-9a-f]*\)".*/\1/p')
[ -n "$MSG" ] || die "3 send failed"
stored() { status agent-b --msg-id "$MSG" | grep -q '"stage":"STORED"'; }
i=0; until stored || [ $i -ge 40 ]; do sleep 0.5; i=$((i + 1)); done
stored && pass "3a message reached STORED at agent-b" || fail "3a message did not reach STORED"
status agent-a | grep -q '"mode":"production"' && pass "3b sidecar runs in production mode" || fail "3b not production mode"
status agent-a | grep -q '"verified":"yes"' && pass "3c status verifies the binary against the release checksum (SI-27)" || fail "3c binary not verified: $(status agent-a | grep -o '"binary":{[^}]*}')"

# 4 upgrade to v2, restart, store survives
TERMLINK_INSTALL_BASE_URL="$REL/v2" sh /work/install.sh > /tmp/install2.log 2>&1 || { cat /tmp/install2.log; die "4 install v2 failed"; }
V2=$(sha256sum /usr/local/bin/termlink | cut -d' ' -f1)
[ "$V2" != "$V1" ] && grep -q "$V2" /srv/expect-v2.sha && [ -e /usr/local/bin/termlink.prev ] && pass "4a upgraded to v2, previous binary kept" || fail "4a upgrade did not install v2"
stopall; startall || fail "4b restart after upgrade failed"
stored && pass "4b after upgrade and restart the stored message is still there" || fail "4b message lost across upgrade"
status agent-a | grep -q '"verified":"yes"' && pass "4c v2 verifies against its own release record" || fail "4c v2 not verified"

# 5 rollback to v1
sh /work/install.sh --rollback > /tmp/rollback.log 2>&1 || { cat /tmp/rollback.log; die "5 rollback failed"; }
[ "$(sha256sum /usr/local/bin/termlink | cut -d' ' -f1)" = "$V1" ] && pass "5a rolled back to the v1 binary" || fail "5a rollback did not restore v1"
stopall; startall || fail "5b restart after rollback failed"
stored && pass "5b after rollback the stored message is still there" || fail "5b message lost across rollback"
status agent-a | grep -q '"verified":"yes"' && pass "5c v1 verifies again against the restored record" || fail "5c v1 not verified after rollback"

# 6 SIGKILL, then the backstop (cron's role here) restarts it with a fresh heartbeat (R-8.e)
P=$(cat /var/lib/termlink-sidecar/test--agent-a/sidecar.pid 2>/dev/null || cat "${SIDECAR_STATE_DIR:-/var/lib/termlink-sidecar}/test--agent-a/sidecar.pid")
kill -9 "$P"; sleep 1
bash "$SUP" check --quiet >> /tmp/sup.log 2>&1
NP=$(cat /var/lib/termlink-sidecar/test--agent-a/sidecar.pid 2>/dev/null)
[ -n "$NP" ] && [ "$NP" != "$P" ] && kill -0 "$NP" 2>/dev/null && status agent-a | grep -q '"heartbeat_age_ms"' \
    && pass "6a a SIGKILLed sidecar is restarted by the backstop with a fresh heartbeat" || fail "6a not restarted after SIGKILL ($(tail -3 /tmp/sup.log))"

if [ "$FAILS" != 0 ]; then
    echo "--- diagnostics"; tail -15 /tmp/sup.log
    for a in agent-a agent-b; do echo "--- sidecar log test--$a"; tail -6 "/var/lib/termlink-sidecar/test--$a/sidecar.log" 2>/dev/null; done
    ps -eo pid,args | grep -E 'sidecar|hub' | grep -v grep
fi
echo "RESULT: $FAILS failed"
[ "$FAILS" = 0 ]
