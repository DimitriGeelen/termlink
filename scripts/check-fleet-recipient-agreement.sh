#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-fleet-recipient-agreement.sh — do our two fleet-wide broadcast paths
# agree on who "the fleet" is?
#
# WHY THIS EXISTS. chat-arc-broadcast.sh and chat-arc-multicast.sh are both
# documented as posting to every hub in the fleet. They resolve recipients by
# different mechanisms:
#
#   broadcast  ~/.termlink/hubs.toml -> dedup by hub FINGERPRINT -> probe
#   multicast  `fleet status`        -> UP only, minus local-test
#
# Until both grew --dry-run (f0259e2b0, 8f8d36ea7) the only way to ask either
# one who it would reach was to send a real message to everyone, so "do these
# agree?" was not a question anyone could cheaply answer. That is the argument
# for a preview being a DIFFERENTIAL INSTRUMENT and not merely a safety feature.
#
# THE NORMALISATION IS THE WHOLE POINT. The two previews are not directly
# comparable — broadcast names ADDRESSES, multicast names PROFILES. Diffing the
# raw lists says "4 vs 3" and implies far more divergence than exists.
# `fleet status` carries the profile<->address mapping, so both sides are
# normalised to profile names before comparison. Measured 2026-08-26, after
# normalising, the two agree on every reachable hub; the only true difference
# was that broadcast included laptop-141 while it was DOWN.
#
# WHAT IT DOES NOT DO. It does not decide which resolution is correct. Whether
# "the fleet" means every configured hub or every currently-UP one, and whether
# a hub reachable at two addresses is one recipient or two, is a topology
# decision for an operator. This reports; it does not reconcile.
#
# Exit: 0 sets agree after normalisation
#       1 they disagree (details printed)
#       2 could not resolve one of the sets — NOT the same as agreement
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

TL="${TERMLINK_BIN:-./target/release/termlink}"
command -v "$TL" >/dev/null 2>&1 || TL=termlink

# profile<->address map, ANSI stripped. "profile<TAB>addr<TAB>state"
fleet_map() {
  "$TL" fleet status 2>&1 \
    | sed -E 's/\x1B\[[0-9;]*[a-zA-Z]//g' \
    | awk '/^[[:space:]]*(UP|DOWN|AUTH-FAIL)[[:space:]]/ {print $2 "\t" $3 "\t" $1}'
}

addr_to_profile() {  # stdin: addresses -> stdout: profile names
  local map="$1"
  while IFS= read -r a; do
    [ -n "$a" ] || continue
    local p
    p=$(printf '%s\n' "$map" | awk -F'\t' -v A="$a" '$2==A {print $1; exit}')
    printf '%s\n' "${p:-UNMAPPED:$a}"
  done
}

preview_hubs() {  # $1 = script, rest = args. Extracts the indented hub block.
  local s="$1"; shift
  bash "$s" "$@" 2>/dev/null \
    | sed -n '/hubs that would receive/,/payload/p' \
    | grep -E '^[[:space:]]{4}[^[:space:]]' \
    | sed 's/^[[:space:]]*//'
}

# Alias map, taken from broadcast's OWN dedup decisions rather than guessed.
# It prints, on stderr, lines of the form:
#   skipping duplicate <addr> (same hub as <addr>, fingerprint=<hex>)
# Two addresses naming one physical hub are ONE recipient. v1 of this script
# lacked this and reported local-test vs workstation-107-public as a
# disagreement — they are 127.0.0.1:9100 and 192.168.10.107:9100, one hub.
#
# `hub fingerprint` has no --hub flag, so per-hub fingerprints cannot be read
# directly; this is the only place the identity is already computed.
#
# Fails toward NOISE: if the line ever stops being printed, the map empties,
# aliases stop folding, and the check reports more divergence — never a false
# AGREE.
alias_pairs() {
  bash scripts/chat-arc-broadcast.sh --payload probe --from agreement-check --dry-run 2>&1 \
    | grep -oE 'skipping duplicate [^ ]+ \(same hub as [^,]+' \
    | sed -E 's/skipping duplicate ([^ ]+) \(same hub as (.*)/\1\t\2/'
}

# Rewrite any address that is an alias to its canonical partner, so both sides
# resolve to the same profile.
canonicalise() {  # stdin addresses; $1 = alias pairs "dupe<TAB>canonical"
  local pairs="$1"
  while IFS= read -r a; do
    [ -n "$a" ] || continue
    local c
    c=$(printf '%s\n' "$pairs" | awk -F'\t' -v A="$a" '$1==A {print $2; exit}')
    printf '%s\n' "${c:-$a}"
  done
}

if [ "${1:-}" = "--self-test" ]; then
  # Prove the normaliser can see a difference AND that it does not invent one.
  map=$'a\t1.1.1.1:9100\tUP\nb\t2.2.2.2:9100\tUP'
  same=$(printf '1.1.1.1:9100\n' | addr_to_profile "$map")
  [ "$same" = "a" ] || { echo "self-test: FAIL — address did not map to its profile (got '$same')"; exit 2; }
  unk=$(printf '9.9.9.9:9100\n' | addr_to_profile "$map")
  case "$unk" in
    UNMAPPED:*) ;;
    *) echo "self-test: FAIL — an unmapped address must be reported, not silently dropped (got '$unk')"; exit 2 ;;
  esac
  echo "self-test: PASS — maps a known address, and reports an unknown one instead of dropping it"
  exit 0
fi

MAP="$(fleet_map)"
[ -n "$MAP" ] || { echo "check-fleet-recipient-agreement: could not read fleet status — no verdict"; exit 2; }

B_RAW="$(preview_hubs scripts/chat-arc-broadcast.sh --payload probe --from agreement-check --dry-run)"
M_RAW="$(preview_hubs scripts/chat-arc-multicast.sh --dry-run probe)"

[ -n "$B_RAW" ] || { echo "check-fleet-recipient-agreement: broadcast preview produced no hubs — no verdict"; exit 2; }
[ -n "$M_RAW" ] || { echo "check-fleet-recipient-agreement: multicast preview produced no hubs — no verdict"; exit 2; }

ALIASES="$(alias_pairs)"

# Both sides -> addresses -> canonical address -> profile. Doing it in address
# space is what lets the alias map apply; profile names cannot express "these
# two are one hub".
profile_to_addr() {
  local map="$1"
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    printf '%s\n' "$(printf '%s\n' "$map" | awk -F'\t' -v P="$p" '$1==P {print $2; exit}')"
  done
}

B="$(printf '%s\n' "$B_RAW" | canonicalise "$ALIASES" | addr_to_profile "$MAP" | sort -u)"
M="$(printf '%s\n' "$M_RAW" | profile_to_addr "$MAP" | canonicalise "$ALIASES" | addr_to_profile "$MAP" | sort -u)"

echo "check-fleet-recipient-agreement: comparing resolved recipients"
echo "  PREDICATE: both --dry-run previews, normalised to PROFILE names via"
echo "             \`fleet status\`. Raw lists are not comparable (one names"
echo "             addresses, one names profiles); diffing them raw overstates"
echo "             divergence. Addresses that broadcast itself reports as the"
echo "             SAME hub (fingerprint dedup) are folded to one recipient."
echo "             Run --self-test to confirm the normaliser works."
echo ""
echo "  broadcast (hubs.toml -> fingerprint-dedup -> probe):"
printf '%s\n' "$B" | sed 's/^/    /'
echo "  multicast (fleet status, UP only, minus local-test):"
printf '%s\n' "$M" | sed 's/^/    /'
echo ""

only_b="$(comm -23 <(printf '%s\n' "$B") <(printf '%s\n' "$M"))"
only_m="$(comm -13 <(printf '%s\n' "$B") <(printf '%s\n' "$M"))"

if [ -z "$only_b" ] && [ -z "$only_m" ]; then
  echo "  AGREE — both resolve the same set of hubs."
  exit 0
fi

echo "  DISAGREE:"
[ -n "$only_b" ] && printf '%s\n' "$only_b" | while IFS= read -r p; do
  st=$(printf '%s\n' "$MAP" | awk -F'\t' -v P="$p" '$1==P {print $3; exit}')
  printf '    broadcast only : %-24s %s\n' "$p" "${st:+(fleet status: $st)}"
done
[ -n "$only_m" ] && printf '%s\n' "$only_m" | while IFS= read -r p; do
  st=$(printf '%s\n' "$MAP" | awk -F'\t' -v P="$p" '$1==P {print $3; exit}')
  printf '    multicast only : %-24s %s\n' "$p" "${st:+(fleet status: $st)}"
done
echo ""
echo "  Before treating any of the above as a bug: broadcast KEEPS an address it"
echo "  cannot probe (hubs-toml-walk.sh fails open by design, so an unreachable"
echo "  hub is reported by the per-hub loop rather than silently omitted). A hub"
echo "  listed here as broadcast-only with fleet status DOWN is that behaviour,"
echo "  not a defect."
echo ""
echo "  Not reconciled here: whether the fleet means every CONFIGURED hub or"
echo "  every currently-UP one is a topology decision, not a scripting one."
exit 1
