#!/usr/bin/env bash
# guard-layer: source
# (no --no-heartbeat: these checks have no heartbeat companion to skip. The
#  marker IS the invocation the runner uses, so declaring a flag we do not
#  accept made check-receiver-ack-lag ERROR under the layer — a contract
#  asserting more than the mechanism, in the machine-readable direction.)
# check-hubs-parse-agreement.sh — do our two hubs.toml parsers agree?
#
# WHY THIS EXISTS. hubs.toml is parsed twice in this repo by independent means:
#
#   scripts/lib/hubs-toml-walk.sh :: hub_addrs_from_toml   awk
#   scripts/agent-chat-arc-recent.sh                       bash regex loop
#
# The first defines "the fleet" for BOTH broadcast paths (operator ruling
# 2026-08-26: every configured hub). A hub it fails to parse is a hub no
# broadcast reaches, silently.
#
# FOUND BY THIS COMPARISON, 2026-08-26: the awk version anchored /^address/ and
# therefore SKIPPED TAB-INDENTED ADDRESS LINES, which are valid TOML. The bash
# parser trims leading whitespace first and accepted them. The shared definition
# — the one implementing the ruling — was the weaker of the two, and the
# tolerant implementation was the one about to be replaced by it.
#
# That is the argument for this file: two implementations of one contract are a
# liability until something compares them, and the comparison is cheap. Merging
# them would remove the redundancy that made the defect visible, so they stay
# separate and this check keeps them honest.
#
# Exit: 0 both parsers agree on every fixture
#       1 they disagree (the differing addresses are printed)
#       2 could not run — NOT the same as agreement
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

LIB=scripts/lib/hubs-toml-walk.sh
[ -f "$LIB" ] || { echo "check-hubs-parse-agreement: $LIB not found — no verdict"; exit 2; }
# shellcheck source=/dev/null
. "$LIB"

T="$(mktemp -d)" || exit 2
trap 'rm -rf "$T"' EXIT INT TERM HUP

# Fixtures are the shapes real TOML permits and a hand-written parser forgets.
# Each is a separate hub so a dropped one is visible by address.
cat > "$T/hubs.toml" <<'TOML'
[hubs.plain]
address = "10.0.0.1:9100"
[hubs.trailing_comment]
address = "10.0.0.2:9100"  # primary link
[hubs.trailing_space]
address = "10.0.0.3:9100"
[hubs.tab_indented]
	address	=	"10.0.0.4:9100"
[hubs.space_indented]
  address = "10.0.0.5:9100"
[hubs.crlf]
address = "10.0.0.6:9100"
TOML
# deliberately no trailing newline on the last entry
printf '[hubs.no_final_newline]\naddress = "10.0.0.7:9100"' >> "$T/hubs.toml"

EXPECTED=7

awk_side="$(hub_addrs_from_toml "$T/hubs.toml")"

# Faithful copy of agent-chat-arc-recent.sh's loop INCLUDING its preprocessing.
# An earlier version of this comparison copied only the regex and omitted the
# comment-strip, which manufactured a divergence that did not exist. A partial
# reproduction is not a control.
bash_side="$(
  current_name=""
  while IFS= read -r raw_line || [ -n "$raw_line" ]; do
    line="${raw_line%$'\r'}"
    line="${line%%#*}"
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    [ -z "$line" ] && continue
    if [[ "$line" =~ ^\[hubs\.([A-Za-z0-9_.-]+)\][[:space:]]*$ ]]; then
      current_name="${BASH_REMATCH[1]}"
    elif [ -n "$current_name" ] && [[ "$line" =~ ^address[[:space:]]*=[[:space:]]*\"([^\"]+)\"[[:space:]]*$ ]]; then
      printf '%s\n' "${BASH_REMATCH[1]}"; current_name=""
    fi
  done < "$T/hubs.toml" | sort -u
)"

n_awk=$(printf '%s\n' "$awk_side" | grep -c . || true)
n_bash=$(printf '%s\n' "$bash_side" | grep -c . || true)

echo "check-hubs-parse-agreement: two independent hubs.toml parsers, $EXPECTED fixtures"
echo "  PREDICATE: fixtures cover plain, trailing comment, trailing space, tab"
echo "             indent, space indent, CRLF and no-final-newline. Both parsers"
echo "             run on the SAME file; the bash side is reproduced with its"
echo "             preprocessing, not just its regex."
echo ""
echo "  hub_addrs_from_toml (awk)      : $n_awk/$EXPECTED"
echo "  agent-chat-arc-recent (regex)  : $n_bash/$EXPECTED"
echo ""

d="$(diff <(printf '%s\n' "$awk_side") <(printf '%s\n' "$bash_side") || true)"
if [ -n "$d" ]; then
  echo "  DISAGREE:"
  printf '%s\n' "$d" | sed 's/^/    /'
  echo ""
  echo "  A hub the SHARED parser drops is unreachable by every broadcast path,"
  echo "  silently — which the 'every configured hub' ruling forbids."
  exit 1
fi

if [ "$n_awk" != "$EXPECTED" ]; then
  echo "  AGREE, but both parsed $n_awk of $EXPECTED fixtures — they may share a"
  echo "  blind spot. Agreement between two implementations is not correctness."
  exit 1
fi

echo "  AGREE — both parsers return all $EXPECTED fixture addresses."
exit 0
