#!/usr/bin/env bash
# T-3781: provision (and renew) a non-root user's HUB LINK on .107, so the user's own sessions and
# dispatched workers register on the root hub over TCP (operator ruling A on T-3759, 2026-11-13).
#
# The link is two files in the user's home, both owner the user, mode 600:
#   ~/.termlink/hub-register.token  a registration-claim token (T-3760): observe scope plus a signed
#                                   claim {principal: USER, prefix: "USER-"}. It may register only
#                                   names under that prefix and renew or remove only its own records.
#   ~/.termlink/hub-register.addr   the hub address (127.0.0.1:9100 on .107)
# Expiry: 7 days, renewed weekly (operator ruling A on T-3781, 2026-11-13). The daily cron re-mints
# when under 3 days are left; the first install is runme's (attended), never the cron's.
#
# Steps: mint on .107 (root reads hub.secret; the user never does) -> install AS THE USER through
# runuser with the token on stdin (never in any argv) -> verify AS THE USER: `termlink register`
# with no hub flags must come up linked, owner = USER (the T-3781 code path) -> record sha + expiry.
# Any miss is FAILED (exit 1). 0 = OK/skip, 1 = FAILED, 2 = tooling.
#
# Usage: hub-link-provision.sh --user USER [--dry-run] [--force] [--cron]
# Seams (fixtures only, PL-213): HUBLINK_HOME (the user's home; default from getent),
# HUBLINK_RUN (command run as `$HUBLINK_RUN USER HOME` with a script on stdin; default runuser),
# HUBLINK_MINT, HUBLINK_RUNTIME_DIR, HUBLINK_STATE_DIR, HUBLINK_ADDR, HUBLINK_TL (user-side termlink).
set -uo pipefail

USER_NAME=""; DRY=0; FORCE=0; CRON=0
while [ $# -gt 0 ]; do
    case "$1" in
        --user) shift; [ $# -ge 1 ] || { echo "hub-link-provision: --user needs a user" >&2; exit 2; }; USER_NAME="$1" ;;
        --dry-run) DRY=1 ;;
        --force) FORCE=1 ;;
        --cron) CRON=1 ;;
        -h|--help) sed -n '2,23p' "$0"; exit 0 ;;
        *) echo "hub-link-provision: unknown argument $1" >&2; exit 2 ;;
    esac
    shift
done
[ -n "$USER_NAME" ] || { echo "hub-link-provision: --user is required" >&2; exit 2; }

MINT="${HUBLINK_MINT:-termlink hub token}"
RUNTIME_DIR="${HUBLINK_RUNTIME_DIR:-/var/lib/termlink}"
STATE_DIR="${HUBLINK_STATE_DIR:-/var/lib/termlink/hub-links}"
ADDR="${HUBLINK_ADDR:-127.0.0.1:9100}"
U_TL="${HUBLINK_TL:-/usr/local/bin/termlink}"
TTL=604800            # 7 days (ruling A)
RENEW_BEFORE=259200   # re-mint under 3 days left
PREFIX="$USER_NAME-"
NOW=$(date +%s)
L="hub-link:$USER_NAME"

say() { printf '  %-7s %s\n' "$1" "$2"; }

run_as_user() {  # stdin = script; prints its output
    if [ -n "${HUBLINK_RUN:-}" ]; then $HUBLINK_RUN "$USER_NAME" "$UHOME"; return; fi
    runuser -u "$USER_NAME" -- env -i HOME="$UHOME" PATH=/usr/local/bin:/usr/bin:/bin bash -s
}
field() { printf '%s\n' "$1" | sed -n "s/^$2=//p" | tail -1; }
has() { printf '%s\n' "$1" | grep -qx "$2"; }

UHOME="${HUBLINK_HOME:-}"
if [ -z "$UHOME" ]; then
    UHOME="$(getent passwd "$USER_NAME" | cut -d: -f6)"
    [ -n "$UHOME" ] || { say FAILED "$L: no such user on this host"; exit 1; }
fi
STATE="$STATE_DIR/$USER_NAME.state"
state_get() { [ -r "$STATE" ] && sed -n "s/^$1=//p" "$STATE" | tail -1; }

# The user-side probe: what is installed now (sha, mode:owner of the token, address file).
PROBE='T="$HOME/.termlink/hub-register.token"; A="$HOME/.termlink/hub-register.addr"
echo PROBE-BEGIN
[ -f "$T" ] && echo "SHA=$(sha256sum < "$T" | cut -d" " -f1)" && echo "MODE=$(stat -c %a:%U "$T")"
[ -f "$A" ] && echo "ADDR=$(tr -d "[:space:]" < "$A")"
'
probe="$(printf '%s' "$PROBE" | run_as_user 2>/dev/null)"
if ! has "$probe" PROBE-BEGIN; then
    say FAILED "$L: no answer over runuser as $USER_NAME; nothing minted or written"; exit 1
fi
sha="$(field "$probe" SHA)"; mode="$(field "$probe" MODE)"; addr="$(field "$probe" ADDR)"
exp_at="$(state_get expires_at)"; rec_sha="$(state_get token_sha256)"
remain=$(( ${exp_at:-0} - NOW ))

if [ "$CRON" = 1 ] && [ -z "$exp_at" ]; then
    say skip "$L: no link on record; the first install is runme.sh action 22e"; exit 0
fi
due=""
[ -n "$exp_at" ] || due="no verified link on record"
[ -z "$due" ] && [ "$remain" -lt "$RENEW_BEFORE" ] && due="token expires in $((remain / 3600))h"
[ -z "$due" ] && [ "$sha" != "$rec_sha" ] && due="installed token differs from the one on record"
[ -z "$due" ] && [ "$mode" != "600:$USER_NAME" ] && due="token file mode/owner is '${mode:-missing}'"
[ -z "$due" ] && [ "$addr" != "$ADDR" ] && due="address file holds '${addr:-nothing}', want $ADDR"
[ -z "$due" ] && [ "$FORCE" = 1 ] && due="--force"
if [ -z "$due" ]; then
    say skip "$L: verified link on record, installed unchanged, expires in $((remain / 3600))h (re-mint under $((RENEW_BEFORE / 3600))h)"
    exit 0
fi
if [ "$DRY" = 1 ]; then
    say "[DRY]" "$L: would mint a registration claim (principal $USER_NAME, prefix $PREFIX, 7 days) on .107 ($due), install ~$USER_NAME/.termlink/hub-register.{token,addr} and prove it with register"
    exit 0
fi

# 1. mint (root side)
SCRATCH="$(mktemp -d /var/tmp/hub-link.XXXXXX)" || { say FAILED "$L: no scratch dir"; exit 2; }
trap 'rm -rf "$SCRATCH"' EXIT
tok="$SCRATCH/t"
if ! mj="$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" $MINT --out "$tok" --register-principal "$USER_NAME" --register-prefix "$PREFIX" --ttl "$TTL" --force --json 2>/dev/null)"; then
    say FAILED "$L: minting on .107 failed (TERMLINK_RUNTIME_DIR=$RUNTIME_DIR: is the hub secret readable? is termlink new enough for --register-principal, T-3760?); nothing written"; exit 1
fi
exp_new="$(printf '%s' "$mj" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("expires_at",""))' 2>/dev/null)"
if [ ! -s "$tok" ] || [ -z "$exp_new" ]; then
    say FAILED "$L: the mint produced no token or no expiry; nothing written"; exit 1
fi
want_sha="$(sha256sum < "$tok" | cut -d' ' -f1)"
b64="$(base64 -w0 < "$tok")"; rm -f "$tok"

# 2. install + 3. verify, as the user, in one call. The script never echoes the token.
out="$(printf 'TOK_B64=%s\nADDR=%s\nTL=%s\nWANT_OWNER=%s\n%s' "$b64" "$ADDR" "$U_TL" "$USER_NAME" '
echo INSTALL-BEGIN
D="$HOME/.termlink"; T="$D/hub-register.token"; A="$D/hub-register.addr"
mkdir -p "$D" && chmod 700 "$D" || echo ERR=mkdir
( umask 077; printf %s "$TOK_B64" | base64 -d > "$T.new" ) && chmod 600 "$T.new" && mv -f "$T.new" "$T" || echo ERR=write-token
( umask 077; printf "%s\n" "$ADDR" > "$A.new" ) && chmod 600 "$A.new" && mv -f "$A.new" "$A" || echo ERR=write-addr
echo "SHA=$(sha256sum < "$T" | cut -d" " -f1)"; echo "MODE=$(stat -c %a:%U "$T")"; echo "ADDR=$(tr -d "[:space:]" < "$A")"
# Prove it the way the user uses it: a plain register with no hub flags must come up linked.
f=$(mktemp); "$TL" register --name hub-link-probe --json > "$f" 2>&1 & p=$!
for _ in $(seq 60); do grep -q "\"hub_link\"" "$f" && break; kill -0 $p 2>/dev/null || break; sleep 0.5; done
kill -TERM $p 2>/dev/null; wait $p 2>/dev/null
if python3 -c "import json,sys
for l in open(sys.argv[1]):
    l=l.strip()
    if l.startswith(\"{\"):
        v=json.loads(l); h=v.get(\"hub_link\") or {}
        sys.exit(0 if v.get(\"ok\") and h.get(\"owner\")==sys.argv[2] else 1)
sys.exit(1)" "$f" "$WANT_OWNER"; then echo VERIFY-OK; else echo "VERIFY-OUT=$(tr "\n" " " < "$f" | cut -c1-300)"; fi
rm -f "$f"' | run_as_user 2>&1)"

bad=""
has "$out" INSTALL-BEGIN || bad="no answer to the install command"
[ -z "$bad" ] && [ -n "$(field "$out" ERR)" ] && bad="write step '$(field "$out" ERR)' failed"
[ -z "$bad" ] && [ "$(field "$out" SHA)" != "$want_sha" ] && bad="installed token sha256 differs from the minted one"
[ -z "$bad" ] && [ "$(field "$out" MODE)" != "600:$USER_NAME" ] && bad="token file is '$(field "$out" MODE)', want 600:$USER_NAME"
[ -z "$bad" ] && [ "$(field "$out" ADDR)" != "$ADDR" ] && bad="address file holds '$(field "$out" ADDR)', want $ADDR"
[ -z "$bad" ] && ! has "$out" VERIFY-OK && bad="a plain 'termlink register' as $USER_NAME did not come up linked with owner $USER_NAME ($U_TL older than T-3781? output: $(field "$out" VERIFY-OUT))"
if [ -n "$bad" ]; then
    say FAILED "$L: $bad; the state on record is unchanged"; exit 1
fi
mkdir -p "$STATE_DIR" && chmod 700 "$STATE_DIR" \
    && printf 'expires_at=%s\ntoken_sha256=%s\nrecorded_at=%s\n' "$exp_new" "$want_sha" "$NOW" > "$STATE.new" && mv -f "$STATE.new" "$STATE" \
    || { say FAILED "$L: link installed and proven, but the state could not be written to $STATE_DIR"; exit 1; }
say OK "$L: registration claim (prefix $PREFIX) installed 600:$USER_NAME with address $ADDR and proven (a plain register came up on the hub, owner $USER_NAME), expires $(date -u -d "@$exp_new" +%Y-%m-%dT%H:%MZ 2>/dev/null || echo "$exp_new")"
exit 0
