#!/usr/bin/env bash
# fleet-token-remint.sh — T-3615 (operator ruling Q3 = A, 2026-10-28): the fleet hubs reach
# .107 with an operator-minted Interact token instead of .107's full hub secret
# (T-3542 "Option 1-lite", built by T-3543). One script, run two ways:
#   runme.sh action 22b   first install (attended; the agent reads the runme log)
#   the .107 cron          daily: re-mints a host's token when it is due (see RENEW_BEFORE)
#
# Per host in FLEET_TOKEN_HUBS:
#   1. mint on .107: `termlink hub token --scope interact` with the largest TTL the ceiling
#      allows (TERMLINK_MAX_TOKEN_TTL_SECS, default 7 days), into a 0700 scratch dir;
#   2. install it on the host over remote exec at $R_TOKEN (0600, written to a temp name and
#      renamed), plus a TOKEN-ONLY config home ($R_HOME/.termlink/hubs.toml: one profile at
#      .107 with token_file and no secret), and add `token_file` to the host's own .107
#      profile in $R_HUBS (its secret_file line stays: a secret still wins there until
#      runme action 24 removes it);
#   3. verify on the host: sha256 of the installed file equals the minted one, mode 600 and
#      owner $R_OWNER, the profile carries the token_file line, and with the TOKEN-ONLY config
#      (so no secret can be what authenticates) `channel list --hub .107` succeeds and a post
#      to the scratch topic $PROBE_TOPIC is accepted by the hub ("Posted to ... offset=").
#   4. record the verified state on .107 ($STATE_DIR/<host>.state: expires_at, sha256 of the
#      token, verified_at). Runme action 24 removes a host's secret only after this record
#      predates the run, i.e. the token path was proven on an earlier run.
# A host is skipped when its recorded token is verified, installed unchanged, and not due.
# The token and the secret are never printed, logged or kept: the scratch dir is removed on exit.
# Residual: the token crosses as part of the remote-exec command (TLS on the wire, but visible
# in the host's process list while that command runs). The hub's rpc audit logs no params.
#
# Usage: fleet-token-remint.sh [--dry-run] [--force] [--cron] [--hub NAME]...
#   --dry-run  probe read-only and say what would happen; mint and write nothing
#   --force    re-mint even when the recorded token is not due
#   --cron     cron mode: print only FAILED lines (timestamped), so the findings log stays
#              empty while healthy; touch $HEARTBEAT on completion; on any FAILED line also
#              put a note on the operator's needs-attention list (each sidecar listed in
#              $SOCKETS_LIST). The cron only RENEWS: a host with no token on record is left
#              to runme action 22b. Without --cron every line is printed (runme reads them).
# Exit: 0 every host ok or skipped · 1 at least one FAILED · 2 tooling (bad args, no python3).
#
# Seams (PL-213, fixtures only): FLEET_TOKEN_HUBS, FLEET_TOKEN_REMOTE (command: <hub> <shell
# command>), FLEET_TOKEN_MINT, FLEET_TOKEN_LOCAL_TL, FLEET_TOKEN_RUNTIME_DIR, FLEET_TOKEN_STATE_DIR,
# FLEET_TOKEN_REMOTE_TOKEN, FLEET_TOKEN_REMOTE_HOME, FLEET_TOKEN_REMOTE_HUBS, FLEET_TOKEN_REMOTE_ID,
# FLEET_TOKEN_REMOTE_TL, FLEET_TOKEN_REMOTE_OWNER, FLEET_TOKEN_NOW, FLEET_TOKEN_RENEW_BEFORE,
# FLEET_TOKEN_NOTE (command: <kind> <why>), FLEET_TOKEN_SOCKETS_LIST, FLEET_TOKEN_HEARTBEAT.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY=0 FORCE=0 CRON=0 ONLY=""
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run) DRY=1 ;;
        --force) FORCE=1 ;;
        --cron) CRON=1 ;;
        --hub) shift; [ $# -ge 1 ] || { echo "fleet-token-remint: --hub needs a name" >&2; exit 2; }; ONLY="$ONLY $1" ;;
        -h|--help) sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "fleet-token-remint: unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done
command -v python3 >/dev/null || { echo "fleet-token-remint: python3 not found" >&2; exit 2; }

HUBS="${ONLY:-${FLEET_TOKEN_HUBS:-ring20-management ring20-dashboard greenfield}}"
ADDR="192.168.10.107:9100"
PROBE_TOPIC="fleet-token-probe"
REMOTE="${FLEET_TOKEN_REMOTE:-}"
MINT="${FLEET_TOKEN_MINT:-termlink hub token}"
LOCAL_TL="${FLEET_TOKEN_LOCAL_TL:-termlink}"
RUNTIME_DIR="${FLEET_TOKEN_RUNTIME_DIR:-/var/lib/termlink}"
STATE_DIR="${FLEET_TOKEN_STATE_DIR:-/var/lib/termlink/fleet-tokens}"
R_TOKEN="${FLEET_TOKEN_REMOTE_TOKEN:-/root/.termlink/secrets/107-hub.token}"
R_HOME="${FLEET_TOKEN_REMOTE_HOME:-/root/.termlink/token-only-home}"
R_HUBS="${FLEET_TOKEN_REMOTE_HUBS:-/root/.termlink/hubs.toml}"
R_ID="${FLEET_TOKEN_REMOTE_ID:-/root/.termlink}"
R_TL="${FLEET_TOKEN_REMOTE_TL:-termlink}"
R_OWNER="${FLEET_TOKEN_REMOTE_OWNER:-root}"
R_PATH="/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
NOW="${FLEET_TOKEN_NOW:-$(date +%s)}"
# Re-mint when fewer than 3 days remain: with the 7-day ceiling a token is replaced every ~4
# days, and a failed run leaves three more daily attempts before the host's token expires.
RENEW_BEFORE="${FLEET_TOKEN_RENEW_BEFORE:-259200}"
SOCKETS_LIST="${FLEET_TOKEN_SOCKETS_LIST:-$ROOT/.context/working/sidecar-admin-sockets.list}"
HEARTBEAT="${FLEET_TOKEN_HEARTBEAT:-$ROOT/.context/working/.fleet-token-remint-canary.heartbeat}"

OKS=0 SKIPS=0 FAILS=0
say() {
    case "$1" in
        FAILED) FAILS=$((FAILS+1)) ;;
        OK|DRY) OKS=$((OKS+1)) ;;
        skip) SKIPS=$((SKIPS+1)) ;;
    esac
    local tag="$1"; shift
    [ "$tag" = DRY ] && tag="[DRY]"
    if [ "$CRON" = 1 ]; then
        [ "$tag" = FAILED ] && printf '%s FAILED  %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*"
    else
        printf '  %-7s %s\n' "$tag" "$*"
    fi
}

SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/.fleet-token.XXXXXX")" || { echo "fleet-token-remint: mktemp failed" >&2; exit 2; }
chmod 700 "$SCRATCH"
trap 'rm -rf "$SCRATCH"' EXIT

remote() {  # <hub> <shell command> -> stdout of the command on that hub's host
    if [ -n "$REMOTE" ]; then $REMOTE "$1" "$2"; return; fi
    local sid; sid="$(timeout 30 termlink remote list "$1" 2>/dev/null | awk 'NR==3 {print $1}')"
    [ -n "$sid" ] || return 3
    timeout 70 termlink remote exec --timeout 60 "$1" "$sid" "$2"
}

# The hubs.toml editor that runs ON the host (line-based, so comments and other profiles are
# kept byte for byte). Modes: probe | add-token <path> | drop-secret.
read -r -d '' EDITOR_PY <<'PY'
import os, re, sys
mode, path, addr = sys.argv[1], sys.argv[2], sys.argv[3]
arg = sys.argv[4] if len(sys.argv) > 4 else ""
try:
    lines = open(path).read().splitlines(True)
except FileNotFoundError:
    lines = []
hdr = re.compile(r'^\s*\[\s*hubs\.("?)([^"\]]+)\1\s*\]\s*(#.*)?$')
kv = re.compile(r'^\s*([A-Za-z_]+)\s*=\s*(.*?)\s*$')
def val(s):
    m = re.match(r'^"((?:[^"\\]|\\.)*)"', s)
    return m.group(1) if m else s.split('#')[0].strip()
secs, cur = [], None
for i, l in enumerate(lines):
    if l.lstrip().startswith('['):
        if cur: cur[2] = i; secs.append(cur); cur = None
        m = hdr.match(l)
        if m: cur = [m.group(2), i, len(lines)]
if cur: secs.append(cur)
def fields(s):
    out = {}
    for j in range(s[1] + 1, s[2]):
        m = kv.match(lines[j])
        if m: out[m.group(1)] = (j, val(m.group(2)))
    return out
def exp(p):
    return os.path.expanduser(p) if p else ""
match = [s for s in secs if fields(s).get("address", (0, None))[1] == addr]
def save():
    if os.path.exists(path) and not os.path.exists(path + ".t3615-bak"):
        with open(path + ".t3615-bak", "w") as b: b.write(open(path).read())
        os.chmod(path + ".t3615-bak", 0o600)
    mode_bits = os.stat(path).st_mode & 0o777 if os.path.exists(path) else 0o600
    tmp = path + ".t3615-new"
    with open(tmp, "w") as f: f.write("".join(lines))
    os.chmod(tmp, mode_bits)
    os.replace(tmp, path)
if mode == "probe":
    print(f"PROFILES={len(match)}")
    if len(match) == 1:
        f = fields(match[0])
        sf = exp(f.get("secret_file", (0, ""))[1])
        print(f"PROFILE={match[0][0]}")
        print(f"TOKEN_FILE={exp(f.get('token_file', (0, ''))[1])}")
        print(f"SECRET_FILE={sf}")
        print(f"INLINE_SECRET={1 if 'secret' in f else 0}")
        others = [s[0] for s in secs if s is not match[0] and sf and exp(fields(s).get('secret_file', (0, ''))[1]) == sf]
        print(f"SECRET_SHARED={','.join(others)}")
elif mode == "add-token":
    if len(match) > 1:
        print("EDIT=refused-ambiguous"); sys.exit(0)
    line = f'token_file = "{arg}"\n'
    if not match:
        names = {s[0] for s in secs}
        name = "hub-107" if "hub-107" not in names else "hub-107-token"
        if lines and not lines[-1].endswith("\n"): lines[-1] += "\n"
        lines += ["\n", f"[hubs.{name}]\n", f'address = "{addr}"\n', line, 'scope = "interact"\n']
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        save(); print(f"EDIT=created:{name}"); sys.exit(0)
    f = fields(match[0])
    if "token_file" in f:
        j = f["token_file"][0]
        if exp(f["token_file"][1]) == arg:
            print("EDIT=unchanged"); sys.exit(0)
        lines[j] = line; save(); print("EDIT=replaced"); sys.exit(0)
    lines.insert(f["address"][0] + 1, line); save(); print("EDIT=added")
elif mode == "drop-secret":
    if len(match) != 1:
        print("EDIT=refused-ambiguous"); sys.exit(0)
    f = fields(match[0])
    if "secret" in f:
        print("EDIT=refused-inline"); sys.exit(0)
    if "secret_file" not in f:
        print("EDIT=absent"); sys.exit(0)
    del lines[f["secret_file"][0]]; save(); print("EDIT=dropped")
PY

# Shell helpers the host runs. Values are baked in with %q, so nothing is re-parsed.
remote_header() {  # <hub>
    printf 'R_TOKEN=%q R_HOME=%q R_HUBS=%q R_ID=%q R_TL=%q ADDR=%q TOPIC=%q HUBNAME=%q\n' \
        "$R_TOKEN" "$R_HOME" "$R_HUBS" "$R_ID" "$R_TL" "$ADDR" "$PROBE_TOPIC" "$1"
    printf 'PY_B64=%q\nexport PATH=%q\n' "$(printf '%s' "$EDITOR_PY" | base64 -w0)" "$R_PATH"
    cat <<'SH'
umask 077
edit() { python3 -c "$(printf %s "$PY_B64" | base64 -d)" "$@" </dev/null; }
tl_tok() { env -u TERMLINK_AGENT_ID -u TERMLINK_IDENTITY_FILE HOME="$R_HOME" TERMLINK_IDENTITY_DIR="$R_ID" timeout 30 "$R_TL" "$@" </dev/null; }
state() {
    echo "SHA=$(sha256sum < "$R_TOKEN" 2>/dev/null | cut -d' ' -f1)"
    echo "MODE=$(stat -c '%a:%U' "$R_TOKEN" 2>/dev/null)"
    echo "HOMECFG=$(sha256sum < "$R_HOME/.termlink/hubs.toml" 2>/dev/null | cut -d' ' -f1)"
    command -v python3 >/dev/null && echo HAVE-PY
    command -v "$R_TL" >/dev/null && echo HAVE-TL
    echo "FP=$(env -u TERMLINK_AGENT_ID -u TERMLINK_IDENTITY_FILE TERMLINK_IDENTITY_DIR="$R_ID" "$R_TL" identity show --json --resolve --no-create </dev/null 2>/dev/null | sed -n 's/.*"fingerprint":"\([0-9a-f]*\)".*/\1/p')"
    command -v python3 >/dev/null && edit probe "$R_HUBS" "$ADDR"
}
prove_token() {
    tl_tok channel list --hub "$ADDR" >/dev/null 2>&1 && echo AUTH-OK
    o="$(tl_tok channel post "$TOPIC" --msg-type token-probe --payload "{\"t3615\":\"token-probe\",\"host\":\"$HUBNAME\",\"ts\":$(date +%s)}" --hub "$ADDR" 2>&1)"
    case "$o" in *"Posted to "*"offset="*) echo POST-OK ;; esac
}
SH
}

home_cfg() {  # the token-only config home's hubs.toml (deterministic: its sha256 is compared)
    printf '# T-3615: token-only profile for .107, written by /opt/termlink/scripts/fleet-token-remint.sh.\n'
    printf '# No secret here: whatever authenticates through this file is the token.\n'
    printf '[hubs.hub-107-token]\naddress = "%s"\ntoken_file = "%s"\nscope = "interact"\n' "$ADDR" "$R_TOKEN"
}
WANT_HOMECFG="$(home_cfg | sha256sum | cut -d' ' -f1)"

run_remote() {  # <hub> <tag> <body> -> tagged output, CR stripped
    local script; script="$( { remote_header "$1"; printf '%s\n' "$3"; } | base64 -w0)"
    remote "$1" "echo RUNME-TOKEN-$2; bash -c \"\$(echo $script | base64 -d)\"" 2>/dev/null | tr -d '\r'
}
field() { printf '%s\n' "$1" | sed -n "s/^$2=//p" | tail -1; }
has() { printf '%s\n' "$1" | grep -qx "$2"; }

state_get() {  # <hub> <key>
    sed -n "s/^$2=//p" "$STATE_DIR/$1.state" 2>/dev/null | tail -1
}
state_put() {  # <hub> <expires_at> <sha> <profile>
    mkdir -p "$STATE_DIR" && chmod 700 "$STATE_DIR" || return 1
    local f="$STATE_DIR/$1.state" mints first
    # mints counts verified installs (1 = the first, 2+ = re-mints); runme action 24 can wait
    # for a re-mint before it removes a host's secret (T-3615, greenfield ruling B). A state
    # written before this field existed counts as one verified mint.
    mints="$(state_get "$1" mints)"; first="$(state_get "$1" first_verified_at)"
    [ -z "$mints" ] && [ -n "$(state_get "$1" verified_at)" ] && mints=1
    mints=$(( ${mints:-0} + 1 ))
    [ -n "$first" ] || first="$(state_get "$1" verified_at)"
    printf 'expires_at=%s\ntoken_sha256=%s\nverified_at=%s\nscope=interact\nprofile=%s\nmints=%s\nfirst_verified_at=%s\n' \
        "$2" "$3" "$NOW" "$4" "$mints" "${first:-$NOW}" > "$f.new" \
        && chmod 600 "$f.new" && mv -f "$f.new" "$f"
}

ensure_probe_topic() {  # the scratch topic on .107 (an Interact token cannot create topics)
    [ "$DRY" = 1 ] && return 0
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" timeout 20 $LOCAL_TL channel create "$PROBE_TOPIC" --retention days:7 \
        --reason "T-3615 token-path probes from the fleet hubs (one post per re-mint)" >/dev/null 2>&1 || true
}

one_hub() {
    local hub="$1" probe sha mode homecfg profiles tokfile exp_at rec_sha remain due=""
    probe="$(run_remote "$hub" PROBE 'echo PROBE-BEGIN; state')"
    if ! has "$probe" PROBE-BEGIN; then
        say FAILED "$hub: no answer over remote exec (no session, or the hub is down); nothing minted or written"; return
    fi
    if ! has "$probe" HAVE-PY || ! has "$probe" HAVE-TL; then
        say FAILED "$hub lacks python3 or $R_TL on $R_PATH; nothing minted or written"; return
    fi
    profiles="$(field "$probe" PROFILES)"
    if [ "${profiles:-0}" -gt 1 ]; then
        say FAILED "$hub has $profiles profiles at $ADDR in $R_HUBS: which one the CLI uses is not defined; merge them by hand first"; return
    fi
    sha="$(field "$probe" SHA)"; mode="$(field "$probe" MODE)"; homecfg="$(field "$probe" HOMECFG)"
    tokfile="$(field "$probe" TOKEN_FILE)"
    exp_at="$(state_get "$hub" expires_at)"; rec_sha="$(state_get "$hub" token_sha256)"
    remain=$(( ${exp_at:-0} - NOW ))
    # The cron only renews: a first install edits the host's hubs.toml and is runme's (attended).
    if [ "$CRON" = 1 ] && [ -z "$exp_at" ]; then
        say skip "$hub: no token on record; the first install is runme.sh action 22b"; return
    fi
    [ -n "$exp_at" ] || due="no verified token on record"
    [ -z "$due" ] && [ "$remain" -lt "$RENEW_BEFORE" ] && due="token expires in $((remain / 3600))h"
    [ -z "$due" ] && [ "$sha" != "$rec_sha" ] && due="installed token differs from the one on record"
    [ -z "$due" ] && [ "$mode" != "600:$R_OWNER" ] && due="token file mode/owner is '${mode:-missing}'"
    [ -z "$due" ] && [ "$homecfg" != "$WANT_HOMECFG" ] && due="token-only config home missing or changed"
    [ -z "$due" ] && [ "$tokfile" != "$R_TOKEN" ] && due="the $ADDR profile has no token_file line"
    [ -z "$due" ] && [ "$FORCE" = 1 ] && due="--force"
    if [ -z "$due" ]; then
        say skip "$hub: verified token on record, installed unchanged, expires in $((remain / 3600))h (re-mint under $((RENEW_BEFORE / 3600))h)"
        return
    fi
    if [ "$DRY" = 1 ]; then
        say DRY "$hub: would mint an Interact token on .107 ($due), install it at $R_TOKEN, add token_file to the $ADDR profile (profiles there now: ${profiles:-0}), then prove it with a token-only config"
        return
    fi
    # 1. mint
    local tok="$SCRATCH/$hub.token" mj exp_new scope b64 want_sha out edit
    if ! mj="$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" $MINT --out "$tok" --scope interact --ttl 315360000 --force --json 2>/dev/null)"; then
        say FAILED "$hub: minting on .107 failed (TERMLINK_RUNTIME_DIR=$RUNTIME_DIR: is the hub secret readable?); nothing written"; return
    fi
    exp_new="$(printf '%s' "$mj" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("expires_at",""))' 2>/dev/null)"
    scope="$(printf '%s' "$mj" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("scope",""))' 2>/dev/null)"
    if [ ! -s "$tok" ] || [ -z "$exp_new" ] || [ "$(printf '%s' "$scope" | tr 'A-Z' 'a-z')" != interact ]; then
        say FAILED "$hub: the mint did not produce an Interact token with an expiry (scope '${scope:-?}'); nothing written"; return
    fi
    want_sha="$(sha256sum < "$tok" | cut -d' ' -f1)"
    b64="$(base64 -w0 < "$tok")"
    # 2. install + 3. verify, in one remote call. The body never echoes the token.
    out="$(run_remote "$hub" INSTALL "TOK_B64=$b64
CFG_B64=$(home_cfg | base64 -w0)
echo INSTALL-BEGIN
mkdir -p \"\$(dirname \"\$R_TOKEN\")\" \"\$R_HOME/.termlink\" && chmod 700 \"\$R_HOME\" \"\$R_HOME/.termlink\" || echo ERR=mkdir
printf %s \"\$TOK_B64\" | base64 -d > \"\$R_TOKEN.new\" && chmod 600 \"\$R_TOKEN.new\" && mv -f \"\$R_TOKEN.new\" \"\$R_TOKEN\" || echo ERR=write-token
printf %s \"\$CFG_B64\" | base64 -d > \"\$R_HOME/.termlink/hubs.toml.new\" && chmod 600 \"\$R_HOME/.termlink/hubs.toml.new\" && mv -f \"\$R_HOME/.termlink/hubs.toml.new\" \"\$R_HOME/.termlink/hubs.toml\" || echo ERR=write-home
edit add-token \"\$R_HUBS\" \"\$ADDR\" \"\$R_TOKEN\"
state
prove_token")"
    rm -f "$tok"
    edit="$(field "$out" EDIT)"
    local bad=""
    has "$out" INSTALL-BEGIN || bad="no answer to the install command"
    [ -z "$bad" ] && [ -n "$(field "$out" ERR)" ] && bad="write step '$(field "$out" ERR)' failed"
    [ -z "$bad" ] && [ "$edit" = refused-ambiguous ] && bad="profile edit refused (several profiles at $ADDR)"
    [ -z "$bad" ] && [ "$(field "$out" SHA)" != "$want_sha" ] && bad="installed token sha256 differs from the minted one"
    [ -z "$bad" ] && [ "$(field "$out" MODE)" != "600:$R_OWNER" ] && bad="token file is '$(field "$out" MODE)', want 600:$R_OWNER"
    [ -z "$bad" ] && [ "$(field "$out" HOMECFG)" != "$WANT_HOMECFG" ] && bad="token-only config home did not verify"
    [ -z "$bad" ] && [ "$(field "$out" TOKEN_FILE)" != "$R_TOKEN" ] && bad="the $ADDR profile in $R_HUBS does not carry token_file = $R_TOKEN"
    [ -z "$bad" ] && ! has "$out" AUTH-OK && bad="channel list --hub $ADDR with the token-only config did not authenticate (remote termlink older than T-3543? action 6 upgrades it)"
    [ -z "$bad" ] && ! has "$out" POST-OK && bad="token authenticated but a post to $PROBE_TOPIC on .107 was not accepted"
    if [ -n "$bad" ]; then
        say FAILED "$hub: $bad; the state on record is unchanged"; return
    fi
    if ! state_put "$hub" "$exp_new" "$want_sha" "$(field "$out" PROFILE)"; then
        say FAILED "$hub: token installed and proven, but the state could not be written to $STATE_DIR"; return
    fi
    say OK "$hub: Interact token installed and proven (sha256 matches, 600:$R_OWNER, profile $(field "$out" PROFILE) ${edit#EDIT=}, token-only list + post to $PROBE_TOPIC accepted), expires $(date -u -d "@$exp_new" +%Y-%m-%dT%H:%MZ 2>/dev/null || echo "$exp_new")"
    local fp; fp="$(field "$out" FP)"
    [ -n "$fp" ] && [ "$CRON" = 0 ] && printf '  %-7s %s\n' note "$hub signs its posts as $fp (the fleet view's sender inventory: scripts/telemetry-senders.txt)"
}

notify() {  # <kind> <why>: best effort, the findings log already carries the line
    if [ -n "${FLEET_TOKEN_NOTE:-}" ]; then $FLEET_TOKEN_NOTE "$1" "$2" >/dev/null 2>&1 || true; return; fi
    [ -r "$SOCKETS_LIST" ] || { echo "fleet-token-remint: no sidecar list at $SOCKETS_LIST: needs-attention note not recorded" >&2; return; }
    local s
    grep -v '^[[:space:]]*#' "$SOCKETS_LIST" | grep -v '^[[:space:]]*$' | while read -r s; do
        termlink sidecar admin note --socket "$s" --kind "$1" --why "$2" >/dev/null 2>&1 \
            || echo "fleet-token-remint: needs-attention note not recorded on $s" >&2
    done
}

ensure_probe_topic
for h in $HUBS; do one_hub "$h"; done

if [ "$CRON" = 1 ] && [ "$FAILS" -gt 0 ]; then
    kind=token-remint soonest=""
    for h in $HUBS; do
        e="$(state_get "$h" expires_at)"; [ -n "$e" ] || continue
        [ -z "$soonest" ] || [ "$e" -lt "$soonest" ] && soonest="$e"
    done
    [ -n "$soonest" ] && [ $((soonest - NOW)) -lt 86400 ] && kind=urgent-token-expiring
    notify "$kind" "T-3615: $FAILS fleet token re-mint(s) failed; see .context/working/.fleet-token-remint-canary.log. A host whose token expires falls back to nothing (mail and telemetry to .107 stop)."
fi
[ "$CRON" = 1 ] && [ "$DRY" = 0 ] && touch "$HEARTBEAT" 2>/dev/null
[ "$CRON" = 0 ] && printf '  %-7s %s\n' total "$OKS ok, $SKIPS skipped, $FAILS failed"
[ "$FAILS" -gt 0 ] && exit 1
exit 0
