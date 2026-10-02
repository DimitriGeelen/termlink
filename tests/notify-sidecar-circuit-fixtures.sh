#!/usr/bin/env bash
# guard-layer: source
#
# T-3325 — fixtures for five-level circuit addressing in the notify sidecar
# (operator ruling Q1 = C, 2026-10-03).
#
# All co-resident agents on a host can sign as one TermLink identity, so the
# sidecar on that identity used to wake its agent on mail meant for another
# (framework:pickup 257 item 4). The sidecar now reads metadata.to_circuit
# (path form or AEF V9 form, or a bare to_project) and wakes only for mail that
# is ours, unaddressed, or falls back to us. Auto-confirm never acks past mail
# addressed elsewhere, because the ack watermark is shared.
#
# Part 1 drives scripts/lib/circuit.py directly. Part 2 runs the REAL sidecar
# with a stub `termlink` on the TERMLINK_BIN seam. Part 3 mutates the shipped
# code and requires a fixture to go red for each guard.
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/notify-sidecar.sh"
CPY="$PROJECT_ROOT/scripts/lib/circuit.py"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
eq()  { if [ "$2" = "$3" ]; then ok "$1"; else bad "$1 (want [$3] got [$2])"; fi; }

for f in "$SCRIPT" "$CPY"; do [ -r "$f" ] || { echo "TOOLING: cannot read $f" >&2; exit 2; }; done
command -v jq >/dev/null || { echo "TOOLING: jq missing" >&2; exit 2; }
command -v python3 >/dev/null || { echo "TOOLING: python3 missing" >&2; exit 2; }

TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

# ---------------------------------------------------------------------------
echo "part 1: circuit.py parse + decide"
# ---------------------------------------------------------------------------
P() { python3 "$CPY" parse "$1"; }
eq "path form with host"   "$(P '//dimitrimintdev/cacc73ea32b121dd/055-cockpit/@cockpit-agent')" \
   '{"host": "dimitrimintdev", "hub": "cacc73ea32b121dd", "project": "055-cockpit", "agent": "cockpit-agent"}'
eq "path form without host (starts at hub)" "$(P 'cacc73ea32b121dd/832-Workflow-designer')" \
   '{"hub": "cacc73ea32b121dd", "project": "832-Workflow-designer"}'
eq "path form with session and agent" "$(P '//h/x/p/s1/@a')" \
   '{"host": "h", "hub": "x", "project": "p", "session": "s1", "agent": "a"}'
v9="$(P 'aef::host=dimitrimintdev::hub=cacc73ea32b121dd::project=055-cockpit::@cockpit-agent::')"
eq "V9 form parses to the same circuit as the path form" "$v9" \
   "$(P '//dimitrimintdev/cacc73ea32b121dd/055-cockpit/@cockpit-agent')"
eq "empty address is unaddressed" "$(P '')" "null"
eq "garbage V9 piece is unparseable" "$(P 'aef::nonsense::')" "null"
eq "nothing may follow the agent" "$(P '//h/x/p/@a/extra')" "null"

C() { # $1 = to_circuit metadata (or "" for none), rest = extra classify args
    local tc="$1"; shift
    local meta='{}'; [ -n "$tc" ] && meta="$(jq -cn --arg c "$tc" '{to_circuit:$c}')"
    printf '{"offset":10,"msg_type":"post","metadata":%s}\n' "$meta" \
      | python3 "$CPY" classify --first-unread 0 --self-host dimitrimintdev.local \
          --self-hub cacc73ea32b121dd --self-project 010-termlink --self-agent claude-termlink "$@" \
      | jq -r 'if .mine==1 then "mine" elif .unaddressed==1 then "unaddressed"
               elif .fallback==1 then "fallback" elif .foreign==1 then "foreign" else "none" end'
}
eq "no to_circuit -> unaddressed (old senders still wake)" "$(C '')" "unaddressed"
eq "addressed to us (path)"  "$(C '//dimitrimintdev/cacc73ea32b121dd/010-termlink/@claude-termlink')" "mine"
eq "addressed to us (V9)"    "$(C 'aef::hub=cacc73ea::project=010-termlink::@claude-termlink::')" "mine"
eq "short host matches FQDN; hub prefix matches" "$(C '//dimitrimintdev/cacc73ea/010-termlink')" "mine"
eq "other project on our hub -> foreign" "$(C '//dimitrimintdev/cacc73ea32b121dd/055-cockpit/@cockpit-agent')" "foreign"
eq "other hub -> foreign"  "$(C '1389a831016c4bf1/010-termlink')" "foreign"
eq "other host -> foreign" "$(C '//dashboard-agent.ring20/cacc73ea32b121dd/010-termlink')" "foreign"
eq "co-resident agent of our project, LIVE -> foreign" \
   "$(C '//dimitrimintdev/cacc73ea32b121dd/010-termlink/@claude-termlink-alt' --live claude-termlink-alt)" "foreign"
eq "co-resident agent of our project, NOT live -> fallback (wakes)" \
   "$(C '//dimitrimintdev/cacc73ea32b121dd/010-termlink/@claude-termlink-alt')" "fallback"
eq "unparseable to_circuit -> unaddressed (wake, never drop)" "$(C 'aef::nonsense::')" "unaddressed"
eq "named session never makes it foreign" "$(C '//dimitrimintdev/cacc73ea32b121dd/010-termlink/some-session')" "mine"
tp="$(printf '{"offset":10,"msg_type":"post","metadata":{"to_project":"055-cockpit"}}\n' \
   | python3 "$CPY" classify --first-unread 0 --self-project 010-termlink --self-agent claude-termlink | jq -r .foreign)"
eq "bare to_project (agent contact name:project) is read as a project address" "$tp" "1"
mt="$(printf '%s\n' '{"offset":10,"msg_type":"receipt","metadata":{}}' '{"offset":3,"msg_type":"post","metadata":{}}' \
   | python3 "$CPY" classify --first-unread 5 --self-project 010-termlink | jq -r '.mine+.unaddressed+.fallback+.foreign')"
eq "receipts and pre-first_unread offsets are not counted" "$mt" "0"

# ---------------------------------------------------------------------------
echo "part 2: the real sidecar against a stub termlink"
# ---------------------------------------------------------------------------
SELF_FP="aaaabbbbccccdddd"
TOPIC="inbox:cacc73ea32b121dd/010-termlink"
cat > "$TMP/termlink" <<STUB
#!/usr/bin/env bash
ENVS="$TMP/envs.ndjson"; POSTED="$TMP/posted.txt"; SELF_FP="$SELF_FP"
STUB
cat >> "$TMP/termlink" <<'STUB'
verb="${1:-}"; sub="${2:-}"
case "$verb $sub" in
  "agent identity") echo "{\"fingerprint\":\"$SELF_FP\"}"; exit 0 ;;
  "hub fingerprint") echo "sha256:cacc73ea32b121dd0000"; exit 0 ;;
  "channel unread")
      jq -s '[.[] | select(.msg_type!="receipt")] as $c
             | {unread_count: ($c|length), first_unread: ($c|map(.offset)|min), last_offset: ($c|map(.offset)|max)}' "$ENVS"
      exit 0 ;;
  "channel subscribe")
      cur=0; while [ $# -gt 0 ]; do case "$1" in --cursor) cur="$2"; shift 2 ;; *) shift ;; esac; done
      jq -c --argjson c "$cur" 'select(.offset >= $c)' "$ENVS"; exit 0 ;;
  "channel post")
      up=""; for a in "$@"; do case "$a" in up_to=*) up="${a#up_to=}" ;; esac; done
      echo "up_to=$up" >> "$POSTED"; echo '{"offset":999}'; exit 0 ;;
esac
exit 0
STUB
chmod +x "$TMP/termlink"

env_line() { # offset, to_circuit-or-empty
    if [ -n "$2" ]; then jq -cn --argjson o "$1" --arg c "$2" '{offset:$o,msg_type:"post",metadata:{to_circuit:$c}}'
    else jq -cn --argjson o "$1" '{offset:$o,msg_type:"post",metadata:{}}'; fi
}
ME='//dimitrimintdev/cacc73ea32b121dd/010-termlink/@fixture-agent'
OTHER='//dimitrimintdev/cacc73ea32b121dd/055-cockpit/@cockpit-agent'

run_cycle() { # writes flag; echoes pending
    TERMLINK_BIN="$TMP/termlink" TERMLINK_NOTIFY_TEST_TOPICS="$TOPIC" \
    TERMLINK_JOURNAL_PATH="$TMP/journal.sqlite" TERMLINK_IDENTITY_DIR="$TMP/none" \
    FW_SIDECAR_SELF_PROJECT=010-termlink FW_SIDECAR_SELF_HOST=dimitrimintdev \
    FW_SIDECAR_SELF_HUB=cacc73ea32b121dd FW_SIDECAR_PRESENCE_AGENTS="${PRESENCE:-}" \
        bash "${SIDECAR:-$SCRIPT}" --agent-id fixture-agent --self-fp "$SELF_FP" \
            --notify-dir "$TMP/notify" --once "$@" > "$TMP/out.txt" 2>&1
    grep -E '^pending=' "$TMP/notify/fixture-agent.flag" 2>/dev/null | cut -d= -f2
}
flag() { grep -E "^$1=" "$TMP/notify/fixture-agent.flag" 2>/dev/null | cut -d= -f2-; }
reset() { rm -rf "$TMP/notify" "$TMP/posted.txt"; : > "$TMP/posted.txt"; : > "$TMP/envs.ndjson"; }

reset
{ env_line 10 "$OTHER"; env_line 11 "$OTHER"; } > "$TMP/envs.ndjson"
eq "S1 mail only for another project does not wake (pending=0)" "$(run_cycle)" "0"
eq "S1 no arrival recorded for it" "$(flag last_mail_ts)" ""

reset
{ env_line 10 "$ME"; env_line 11 ""; env_line 12 "$OTHER"; } > "$TMP/envs.ndjson"
eq "S2 mixed: counts ours + unaddressed only (pending=2)" "$(run_cycle)" "2"
ts1="$(flag last_mail_ts)"
[ -n "$ts1" ] && ok "S2 arrival recorded" || bad "S2 arrival recorded"

reset
{ env_line 10 "$ME"; env_line 11 "$OTHER"; env_line 12 "$ME"; } > "$TMP/envs.ndjson"
run_cycle --auto-confirm >/dev/null
eq "S3 auto-confirm acks only up to the message before the foreign one (up_to=10)" \
   "$(grep -c '^up_to=10$' "$TMP/posted.txt")/$(grep -c '^up_to=' "$TMP/posted.txt")" "1/1"

reset
{ env_line 10 "$OTHER"; env_line 11 "$ME"; } > "$TMP/envs.ndjson"
run_cycle --auto-confirm >/dev/null
eq "S4 foreign mail first: nothing is acked at all" "$(grep -c '^up_to=' "$TMP/posted.txt")" "0"

reset
{ env_line 10 "$ME"; env_line 11 "$OTHER"; env_line 12 "$ME"; } > "$TMP/envs.ndjson"
run_cycle >/dev/null; a="$(flag last_mail_ts)"; sleep 1.1
run_cycle >/dev/null; b="$(flag last_mail_ts)"
eq "S5 same unread set next cycle does NOT re-advance the arrival record" "$b" "$a"
env_line 13 "$ME" >> "$TMP/envs.ndjson"; sleep 1.1
run_cycle >/dev/null; c="$(flag last_mail_ts)"
[ -n "$c" ] && [ "$c" != "$b" ] && ok "S5 new mail for us DOES advance it" || bad "S5 new mail for us DOES advance it ($b -> $c)"

reset
mkdir -p "$TMP/notify"
ALT='//dimitrimintdev/cacc73ea32b121dd/010-termlink/@other-agent'
env_line 10 "$ALT" > "$TMP/envs.ndjson"
eq "S6 co-resident agent of our project NOT live -> fallback wakes us" "$(run_cycle)" "1"
reset; mkdir -p "$TMP/notify"
date +%s%3N > "$TMP/notify/other-agent.heartbeat"
env_line 10 "$ALT" > "$TMP/envs.ndjson"
eq "S6 same address with other-agent LIVE (fresh heartbeat) -> does not wake" "$(run_cycle)" "0"

reset
env_line 10 "$ALT" > "$TMP/envs.ndjson"
eq "S8 co-resident agent LIVE on the presence rail (no sidecar here) -> does not wake" \
   "$(PRESENCE=other-agent run_cycle)" "0"

reset
env_line 10 "$OTHER" > "$TMP/envs.ndjson"
mkdir -p "$TMP/nolib" && cp "$SCRIPT" "$TMP/nolib/notify-sidecar.sh"   # no lib/circuit.py beside it
p="$(SIDECAR="$TMP/nolib/notify-sidecar.sh" run_cycle)"
eq "S7 classifier unavailable -> falls back to the raw count (wakes, never drops)" "$p" "1"

# ---------------------------------------------------------------------------
echo "part 3: mutants (each must turn a fixture red)"
# ---------------------------------------------------------------------------
mutant() { # name, sed expression applied to a copy of the sidecar; expect pending
    local name="$1" expr="$2" want_red="$3"
    sed "$expr" "$SCRIPT" > "$TMP/mut.sh"
    if cmp -s "$TMP/mut.sh" "$SCRIPT"; then bad "mutant $name did not apply"; return; fi
    cp "$PROJECT_ROOT/scripts/lib/circuit.py" "$TMP/circuit.py" 2>/dev/null
    mkdir -p "$TMP/lib" && cp "$CPY" "$TMP/lib/circuit.py"
    cp "$TMP/mut.sh" "$TMP/notify-sidecar.sh"
    reset; { env_line 10 "$ME"; env_line 11 "$OTHER"; env_line 12 "$ME"; } > "$TMP/envs.ndjson"
    local p; p="$(SIDECAR="$TMP/notify-sidecar.sh" run_cycle --auto-confirm)"
    local got="pending=$p acks=$(tr '\n' ',' < "$TMP/posted.txt")"
    if [ "$got" != "$want_red" ]; then ok "mutant $name is caught ($got)"; else bad "mutant $name survived ($got)"; fi
}
GOOD="pending=2 acks=up_to=10,"
# baseline with the unmutated copy, same harness
mkdir -p "$TMP/lib" && cp "$CPY" "$TMP/lib/circuit.py" && cp "$SCRIPT" "$TMP/notify-sidecar.sh"
reset; { env_line 10 "$ME"; env_line 11 "$OTHER"; env_line 12 "$ME"; } > "$TMP/envs.ndjson"
base="pending=$(SIDECAR="$TMP/notify-sidecar.sh" run_cycle --auto-confirm) acks=$(tr '\n' ',' < "$TMP/posted.txt")"
eq "mutant harness baseline is green" "$base" "$GOOD"
mutant "no-filter (wake on raw count)"   's/^                total=\$((total + wake))/                total=$((total + n))/' "$GOOD"
mutant "no-ack-cap"                      's/^        \*) \[ \$((foreign_min - 1)) -lt/        *) false \&\& [ $((foreign_min - 1)) -lt/' "$GOOD"
# first_unread floor: only shows when the FIRST unread message is someone else's.
sed 's/^           \[ "\$latest_off" -ge "\$first_unread" \] || return 0 ;;/           ;;/' "$SCRIPT" > "$TMP/notify-sidecar.sh"
if cmp -s "$TMP/notify-sidecar.sh" "$SCRIPT"; then bad "mutant no-first-unread-floor did not apply"; else
    reset; { env_line 10 "$OTHER"; env_line 11 "$ME"; } > "$TMP/envs.ndjson"
    SIDECAR="$TMP/notify-sidecar.sh" run_cycle --auto-confirm >/dev/null
    n_acks="$(grep -c '^up_to=' "$TMP/posted.txt")"
    [ "$n_acks" != "0" ] && ok "mutant no-first-unread-floor is caught (posts a pointless ack: $(tr '\n' ' ' < "$TMP/posted.txt"))" \
                         || bad "mutant no-first-unread-floor survived"
fi
# The coverage-guard mutant only shows on a short fetch, so it gets its own shape.
sed 's/^    \[ "\$seen" -ge "\$n" \] || return 1/    true/' "$SCRIPT" > "$TMP/notify-sidecar.sh"
reset; env_line 10 "$ME" > "$TMP/envs.ndjson"
cat > "$TMP/termlink.short" <<STUB
#!/usr/bin/env bash
case "\${1:-} \${2:-}" in "channel subscribe") exit 0 ;; esac
exec "$TMP/termlink" "\$@"
STUB
chmod +x "$TMP/termlink.short"
p="$(TERMLINK_BIN="$TMP/termlink.short" TERMLINK_NOTIFY_TEST_TOPICS="$TOPIC" TERMLINK_IDENTITY_DIR="$TMP/none" FW_SIDECAR_PRESENCE_AGENTS= \
     FW_SIDECAR_SELF_PROJECT=010-termlink bash "$TMP/notify-sidecar.sh" --agent-id fixture-agent --self-fp "$SELF_FP" \
     --notify-dir "$TMP/notify" --once >/dev/null 2>&1; flag pending)"
[ "$p" = "0" ] && ok "mutant no-coverage-guard is caught on a short fetch (pending=0 = silent miss)" \
               || bad "mutant no-coverage-guard survived on a short fetch (pending=$p)"
reset; env_line 10 "$ME" > "$TMP/envs.ndjson"
p="$(TERMLINK_BIN="$TMP/termlink.short" TERMLINK_NOTIFY_TEST_TOPICS="$TOPIC" TERMLINK_IDENTITY_DIR="$TMP/none" FW_SIDECAR_PRESENCE_AGENTS= \
     FW_SIDECAR_SELF_PROJECT=010-termlink bash "$SCRIPT" --agent-id fixture-agent --self-fp "$SELF_FP" \
     --notify-dir "$TMP/notify" --once >/dev/null 2>&1; flag pending)"
eq "shipped code on the same short fetch falls back and wakes (pending=1)" "$p" "1"

echo
echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
