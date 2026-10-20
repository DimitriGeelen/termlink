# T-3499 — remote-side scripts for scripts/hub-binary-swap.sh.
#
# The swap helper used to relaunch with only TERMLINK_RUNTIME_DIR and the hub's
# arguments pasted unquoted into a shell string (T-3497's bug, in the manual
# helper). These generators emit bash scripts that run on the hub host, with the
# T-3497 reader (hub-termlink-env.sh) embedded verbatim, since the host has no
# checkout. The probe captures the hub's TERMLINK_* environment and argv BEFORE
# anything is stopped (fail-closed: unreadable -> the probe fails and nothing
# moves); the swap and rollback scripts receive them as base64 of NUL-joined
# entries, so no value or argument ever passes through a shell word-split.
#
#   write_swap_probe  <out>                       prints HUB_PID=.. HUB_ENV_B64=.. HUB_ARGV_B64=..
#   write_swap_relaunch <out> <swap|rollback> <staged> <hub_pid> <env_b64> <argv_b64> [bin] [log]
_hub_swap_envlib() { echo "$(dirname "${BASH_SOURCE[0]}")/hub-termlink-env.sh"; }

write_swap_probe() {
    local out="$1" envlib; envlib="$(_hub_swap_envlib)"
    [ -r "$envlib" ] || { echo "write_swap_probe: $envlib missing" >&2; return 1; }
    cat > "$out" <<'EOS'
set -e
# @@HUB_TERMLINK_ENV_LIB@@
HUB_PID=$(pgrep -f '[t]ermlink hub start' | head -1)
test -n "$HUB_PID" || { echo 'ERR: no termlink hub process found' >&2; exit 1; }
hub_termlink_env "$HUB_PID" || { echo "ERR: cannot read the running hub's environment; refusing (T-3499)" >&2; exit 1; }
mapfile -d '' PROBE_ARGV < "${HUB_PROC_ROOT:-/proc}/$HUB_PID/cmdline" 2>/dev/null || PROBE_ARGV=()
[ "${#PROBE_ARGV[@]}" -gt 0 ] || { echo "ERR: cannot read the running hub's cmdline; refusing (T-3499)" >&2; exit 1; }
echo "HUB_PID=$HUB_PID"
echo "HUB_ENV_B64=$(printf '%s\0' "${HUB_TL_ENV[@]}" | base64 -w0)"
echo "HUB_ARGV_B64=$(printf '%s\0' "${PROBE_ARGV[@]:1}" | base64 -w0)"
EOS
    sed -i "/^# @@HUB_TERMLINK_ENV_LIB@@\$/r $envlib" "$out"
    grep -q '^hub_termlink_env() {' "$out" || { echo "write_swap_probe: env reader not embedded" >&2; return 1; }
}

write_swap_relaunch() {
    local out="$1" mode="$2" staged="$3" pid="$4" envb="$5" argvb="$6"
    local bin="${7:-/usr/local/bin/termlink}" log="${8:-/var/log/termlink-hub.log}" envlib
    envlib="$(_hub_swap_envlib)"
    [ -r "$envlib" ] || { echo "write_swap_relaunch: $envlib missing" >&2; return 1; }
    case "$mode" in swap|rollback) ;; *) echo "write_swap_relaunch: bad mode $mode" >&2; return 1;; esac
    {
        echo "set -e"
        printf 'MODE=%q STAGED=%q HUB_PID=%q ENV_B64=%q ARGV_B64=%q BIN=%q HUB_LOG=%q\n' \
            "$mode" "$staged" "$pid" "$envb" "$argvb" "$bin" "$log"
        echo "# @@HUB_TERMLINK_ENV_LIB@@"
        cat <<'EOS'
# The hub's TERMLINK_* environment and argv, decoded BEFORE anything changes.
ENVF=$(mktemp); trap 'rm -f "$ENVF"' EXIT
printf '%s' "$ENV_B64" | base64 -d > "$ENVF"
hub_termlink_env "" "$ENVF" || { echo "ERR: cannot restore the hub's TERMLINK_* environment; nothing changed (T-3499)" >&2; exit 1; }
mapfile -d '' RUN_ARGS < <(printf '%s' "$ARGV_B64" | base64 -d)
relaunch_hub() {
  env "${HUB_ENV_ARGS[@]}" setsid nohup "$BIN" "${RUN_ARGS[@]}" </dev/null >>"$HUB_LOG" 2>&1 &
}
hub_up() { pgrep -f '[t]ermlink hub start' >/dev/null; }
if [ "$MODE" = swap ]; then
  cp "$BIN" "$BIN.bak"
  mv "$STAGED" "$BIN"
  chmod +x "$BIN"
  kill "$HUB_PID" 2>/dev/null || true
  for i in 1 2 3 4 5; do hub_up || break; sleep 1; done
  if hub_up; then echo 'ERR: hub did not exit within 5s' >&2; exit 1; fi
  relaunch_hub
  disown
  sleep 1
  for i in 1 2 3 4 5 6 7 8 9 10; do
    if hub_up && ss -tlnp 2>/dev/null | grep -q ':9100'; then UP=1; break; fi
    sleep 1
  done
  echo "HUB_UP=${UP:-0}"
  echo "HUB_PID_NEW=$(pgrep -f '[t]ermlink hub start' | head -1)"
  echo "POST_SECRET_SHA=$(sha256sum /var/lib/termlink/hub.secret | awk '{print $1}')"
  echo "POST_CERT_SHA=$(sha256sum /var/lib/termlink/hub.cert.pem | awk '{print $1}')"
  echo "POST_BIN_VERSION=$("$BIN" --version)"
else
  test -f "$BIN.bak" || { echo 'ERR: backup missing — manual recovery required' >&2; exit 1; }
  pkill -f '[t]ermlink hub start' 2>/dev/null || true
  sleep 2
  mv "$BIN.bak" "$BIN"
  relaunch_hub
  disown
  sleep 2
  echo "ROLLBACK_VERSION=$("$BIN" --version)"
  echo "ROLLBACK_HUB_UP=$(pgrep -f '[t]ermlink hub start' | head -1)"
fi
EOS
    } > "$out"
    sed -i "/^# @@HUB_TERMLINK_ENV_LIB@@\$/r $envlib" "$out"
    grep -q '^hub_termlink_env() {' "$out" || { echo "write_swap_relaunch: env reader not embedded" >&2; return 1; }
}
