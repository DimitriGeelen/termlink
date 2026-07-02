#!/usr/bin/env bash
# T-2302 (arc-003 reliable-comms, V6 slice S5) — tests for scripts/journal-reaper.sh
# and the recent-dm.sh journal-merge read path.
#
# Hub-independent on loopback: everything posts to the live local hub at
# 127.0.0.1:9100 (a real hub, no mock). A fresh $$/timestamp run-tag gives unique
# synthetic dm: topics per run — no cross-run bleed, and the reaper is always scoped
# with --topic so it never touches any OTHER dm: topic on the live hub.
#
# An isolated $TERMLINK_JOURNAL_PATH under a tmp dir keeps the journal out of the real
# ~/.termlink/journals/journal.sqlite.
#
# Covers the S5 ACs:
#   R1 — full reap: N dm turns trimmed off the firehose but ALL present in the journal;
#        newest WINDOW kept; a non-dm presence topic is untouched (AC1/AC4).
#   R2 — SAFETY GUARD: an un-journaled offset in the prune range makes the reaper
#        REFUSE to trim the whole topic (AC2) — trim-ahead-of-journal is impossible.
#   R3 — SCOPE: reaper refuses a non-dm --topic (AC4).
#   R4 — READ PATH: /recent-dm output is unchanged after reaping — trimmed turns still
#        appear because recent-dm merges the journal (AC3).
#   R5 — IDEMPOTENT: re-running the reaper on an already-trimmed topic is a no-op (AC1).
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REAPER="${REAPER:-$HERE/journal-reaper.sh}"
MIRROR="${MIRROR:-$HERE/journal-mirror.sh}"
JOURNALQ="${JOURNALQ:-$HERE/agent-journal.sh}"
RECENT_DM="${RECENT_DM:-$HERE/recent-dm.sh}"
TERMLINK="${TERMLINK_BIN:-termlink}"
H=127.0.0.1:9100

command -v "$TERMLINK" >/dev/null 2>&1 || { echo "SKIP: termlink not on PATH"; exit 0; }
"$TERMLINK" hub status >/dev/null 2>&1  || { echo "SKIP: no local hub running"; exit 0; }
command -v jq >/dev/null 2>&1           || { echo "SKIP: jq not available"; exit 0; }
command -v sqlite3 >/dev/null 2>&1      || { echo "SKIP: sqlite3 not available"; exit 0; }

PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
export TERMLINK_JOURNAL_PATH="$tmp/journal.sqlite"

run_tag="t2302-$$-$(date +%s)"

# Post $2 turns to dm topic $1, payloads "$3-1".."$3-N". Distinct payloads → distinct
# previews so the recent-dm dedup keeps them all as separate rows.
post_turns() {
    local topic="$1" n="$2" prefix="$3" i
    for ((i = 1; i <= n; i++)); do
        "$TERMLINK" channel post "$topic" --hub "$H" --msg-type turn \
            --payload "$prefix-$i" --metadata conversation_id="cid-$run_tag" \
            --ensure-topic --json >/dev/null 2>&1
    done
}

# Sorted offsets currently on the firehose for a topic.
fh_offsets() {
    "$TERMLINK" channel subscribe "$1" --hub "$H" --cursor 0 --limit 100000 --json 2>/dev/null \
        | jq -r '.offset' 2>/dev/null | sort -n | tr '\n' ' '
}
fh_count() { "$TERMLINK" channel subscribe "$1" --hub "$H" --cursor 0 --limit 100000 --json 2>/dev/null | jq -r '.offset' 2>/dev/null | grep -c '^[0-9]'; }
# Count rows for a topic in the isolated journal.
journal_count() { bash "$JOURNALQ" "$1" --json 2>/dev/null | jq -r '.count // 0' 2>/dev/null; }

# -------- R1: full reap — trim off firehose, keep newest WINDOW, all in journal --------
echo "R1: reap 5 dm turns (window=2) → oldest 3 pruned from firehose, all 5 in journal; presence topic untouched"
dm1="dm:${run_tag}:r1peer"
pres="agent-presence-${run_tag}"     # a NON-dm topic — must never be touched
post_turns "$dm1" 5 "r1turn"
# a few "heartbeats" on the presence topic
for i in 1 2 3; do "$TERMLINK" channel post "$pres" --hub "$H" --msg-type note --payload "hb-$i" --ensure-topic --json >/dev/null 2>&1; done
pres_before="$(fh_count "$pres")"
bash "$MIRROR" --hub "$H" --topic "$dm1" >/dev/null 2>&1
"$REAPER" --hub "$H" --topic "$dm1" --window 2 >/dev/null 2>&1
after_off="$(fh_offsets "$dm1")"
jrows="$(journal_count "$dm1")"
pres_after="$(fh_count "$pres")"
# Expect firehose = the newest 2 offsets (3 4); journal has all 5; presence unchanged.
if [ "$after_off" = "3 4 " ] && [ "$jrows" = "5" ] && [ "$pres_before" = "$pres_after" ]; then
    pass "R1: firehose trimmed to newest 2 (offsets: $after_off), journal has all 5, presence untouched ($pres_after)"
else
    fail "R1: after_off='$after_off' jrows=$jrows pres_before=$pres_before pres_after=$pres_after"
fi

# -------- R2: SAFETY GUARD — un-journaled prune-range offset ⇒ refuse to trim --------
echo "R2: partial journal (offsets 0-2 missing) + --no-mirror → guard SKIP-UNSAFE, nothing pruned"
dm2="dm:${run_tag}:r2peer"
post_turns "$dm2" 6 "r2turn"   # offsets 0..5
# Mirror ONLY offsets >= 3 → journal missing 0,1,2 (which fall in the window=2 prune range).
bash "$MIRROR" --hub "$H" --topic "$dm2" --since-offset 3 >/dev/null 2>&1
out2="$("$REAPER" --hub "$H" --topic "$dm2" --window 2 --no-mirror 2>&1)"
after2="$(fh_offsets "$dm2")"
if printf '%s' "$out2" | grep -q "SKIP-UNSAFE" && [ "$(fh_count "$dm2")" = "6" ] && printf '%s' "$after2" | grep -q '^0 '; then
    pass "R2: guard refused (SKIP-UNSAFE), all 6 offsets still on firehose (offset 0 survived)"
else
    fail "R2: out='$(printf '%s' "$out2" | tr '\n' '|')' after='$after2' count=$(fh_count "$dm2")"
fi

# -------- R3: SCOPE — reaper refuses a non-dm --topic --------
echo "R3: --topic on a non-dm topic → refuse (exit 2), no mutation"
out3="$("$REAPER" --hub "$H" --topic "$pres" --window 1 2>&1)"; rc3=$?
if [ "$rc3" -eq 2 ] && printf '%s' "$out3" | grep -qF "refusing to reap non-dm topic"; then
    pass "R3: non-dm topic refused loud (rc=2)"
else
    fail "R3: rc=$rc3 out='$(printf '%s' "$out3" | tr '\n' '|')'"
fi

# -------- R4: READ PATH — /recent-dm unchanged after reaping (journal merge) --------
echo "R4: recent-dm shows all turns before AND after reaping (journal-backed read survives trim)"
dm4="dm:${run_tag}:r4peer"
post_turns "$dm4" 5 "r4turn"
bash "$MIRROR" --hub "$H" --topic "$dm4" >/dev/null 2>&1
# previews before reap (firehose has all 5)
before_json="$(bash "$RECENT_DM" --topic "$dm4" --hub "$H" --limit 50 --json 2>/dev/null)"
before_previews="$(printf '%s' "$before_json" | jq -r '.posts[].payload_preview' 2>/dev/null | sort | tr '\n' ',')"
"$REAPER" --hub "$H" --topic "$dm4" --window 2 >/dev/null 2>&1
after_json="$(bash "$RECENT_DM" --topic "$dm4" --hub "$H" --limit 50 --json 2>/dev/null)"
after_previews="$(printf '%s' "$after_json" | jq -r '.posts[].payload_preview' 2>/dev/null | sort | tr '\n' ',')"
# Firehose really was trimmed (control): --no-journal now sees fewer than 5.
fhonly="$(bash "$RECENT_DM" --topic "$dm4" --hub "$H" --limit 50 --no-journal --json 2>/dev/null | jq -r '.posts | length' 2>/dev/null)"
if [ -n "$before_previews" ] && [ "$before_previews" = "$after_previews" ] && [ "${fhonly:-0}" -lt 5 ]; then
    pass "R4: recent-dm unchanged after reap (5 turns), firehose-only control sees only $fhonly (trim confirmed)"
else
    fail "R4: before='$before_previews' after='$after_previews' fhonly='$fhonly'"
fi

# -------- R5: IDEMPOTENT — re-reap an already-trimmed topic is a no-op --------
echo "R5: re-run reaper on dm1 (already trimmed to window) → SKIP-WINDOW, 0 pruned"
out5="$("$REAPER" --hub "$H" --topic "$dm1" --window 2 2>&1)"; rc5=$?
if [ "$rc5" -eq 0 ] && printf '%s' "$out5" | grep -q "SKIP-WINDOW" && [ "$(fh_offsets "$dm1")" = "3 4 " ]; then
    pass "R5: idempotent no-op (SKIP-WINDOW), firehose unchanged"
else
    fail "R5: rc=$rc5 out='$(printf '%s' "$out5" | tr '\n' '|')' off='$(fh_offsets "$dm1")'"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
