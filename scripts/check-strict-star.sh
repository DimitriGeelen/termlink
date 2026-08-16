#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# check-strict-star.sh (T-2703, closing T-2702 finding F1)
#
# Guards the architecture doc's decisive invariant:
#
#     "Strict star; spokes never connect to one another."
#     (docs/architecture/parallel-execution-substrate.md § 10)
#
# The charter DELEGATES AUTHORITY to that document — it calls it "the
# authoritative statement of the substrate design and its invariants" — and
# § 3 spends forty lines rejecting a spoke-to-spoke mesh, calling the
# fragility argument decisive. Until this check, that invariant was guarded
# by nothing.
#
# WHERE THIS SITS AMONG THE THREE GUARDS OF THIS INVARIANT
# ---------------------------------------------------------
# This is the THIRD guard, and the smallest. Read the other two first:
#
# (1) crates/termlink-session/tests/no_spoke_mesh_tripwire.rs (T-2703) is the
#     PRIMARY guard. It pins the per-module connect counts in termlink-session
#     (client.rs 5, transport.rs 3, tofu.rs 1, ws_consumer.rs 1) and fires on
#     any socket opened outside those four modules. Stronger than this script:
#     it catches a count change within a known module, which a per-site
#     ledger does not. termlink-session is therefore NOT scanned here.
#
# (2) T-2571 proved the invariant holds BY CONSTRUCTION on the peer-contact
#     path — `agent.rs::resolve_home_hub` deliberately excludes a peer's
#     `metadata.observed_addr` (host + ephemeral process port, T-2297) from
#     routing, pinned by `resolve_home_hub_precedence`. Real and load-bearing,
#     but a BEHAVIOURAL test of ONE function: it cannot fail when somebody
#     adds a NEW dial site somewhere else.
#
# (3) THIS check covers the crates neither of those reaches — termlink-cli and
#     termlink-mcp. T-2702 F1 named them: a direct spoke-to-spoke channel
#     "could be introduced in termlink-session OR termlink-cli and no test
#     would fail". (1) closed the first half; this closes the second.
#
# Not to be confused with T-2569's no_federation_tripwire.rs, which scans only
# crates/termlink-hub/src and forbids the HUB from building a hub-speaking
# client — hub-to-hub FEDERATION, charter non-goal #1. A different edge.
#
# WHAT THIS CHECK ACTUALLY DOES  (scope disclosure, T-2680)
# ---------------------------------------------------------
# It does NOT prove any target is a hub. A shell script cannot resolve, at
# rest, what a runtime address refers to — claiming otherwise would be the
# over-reported-scope defect T-2680 fixed in the charter-drift canary.
#
# What it does is convert UNEXAMINED into ACKNOWLEDGED, the T-2747 ratchet:
# every raw dial site in the spoke crates must be either
#
#   (a) test-context (inside the file's `#[cfg(test)]` region — a test
#       connecting to a socket it just bound itself is not a mesh), or
#   (b) listed in the allowlist WITH a cited reason naming what the target
#       is and why it is not a peer spoke.
#
# Today's surface is frozen and visible; a NEW dial site is in neither set
# and fires on the next run. That is the property T-2571's test structurally
# cannot have.
#
# The allowlist rule is the stricter, T-2693-style one: the reason must name
# THE TARGET (a hub address / a local session socket / the process's own
# listener). "Safe" is not a reason — the acknowledgement IS the topology
# documentation, kept next to the signature so it stays in sync.
#
# WHY THE MESH IS FORBIDDEN  (§ 3's decisive argument, kept here on purpose)
# --------------------------------------------------------------------------
# A future reader who wants to "just add a direct channel between two agents"
# should be able to see the cost without hunting for the doc:
#
#   - N² fragility. A star has N links and one place to reason about. A mesh
#     has up to N(N-1)/2, and every one of them is a new partition mode, a new
#     auth pairing, and a new thing to observe. The failure surface grows
#     quadratically while the operator's attention does not.
#   - No central durable replay. The hub is the append log: a spoke that was
#     down replays from an offset and catches up. A direct spoke-to-spoke
#     message exists only in the sender's and receiver's memory — if the
#     receiver was down, it is simply gone, and nothing can reconstruct it.
#   - Silent partial-partition divergence. In a star, a partition is visible:
#     the spoke cannot reach the hub and says so. In a mesh, A-B can be up
#     while A-C is down, so two agents hold different views of the same
#     conversation and NEITHER can detect the disagreement locally. That is a
#     Directive #2 violation (no silent failures) by topology, not by bug.
#
# RESIDUAL — what this check would still MISS
# -------------------------------------------
# Stated because a guard that does not name its blind spots invites its green
# to be over-read (T-2680):
#
#   1. A dial through an INDIRECTION this anchor cannot see: a helper in
#      another crate, a `dyn` transport injected at runtime, a std-library or
#      third-party client (reqwest, tungstenite's own connector) that opens
#      its own socket. The anchor is the raw stream connect; a spoke-to-spoke
#      channel built on a higher-level library that never names TcpStream
#      would not appear.
#   2. A dial whose TARGET changes meaning without the site changing: an
#      already-acknowledged site whose caller starts passing a peer address
#      instead of a hub address. The ledger pins the SITE, not the value that
#      flows into it — this is exactly the half T-2571's behavioural test
#      covers, which is why both guards are needed and neither is redundant.
#   3. Accepting an INBOUND connection from another spoke. This check only
#      looks at the dialling end.
#   4. A dial added inside a `#[cfg(test)]` region and then promoted to
#      production by deleting the attribute — the site would be re-classified
#      on the next run, but nothing here notices the attribute's removal
#      specifically.
#
# Exit codes: 0 = every dial site acknowledged, 1 = an unacknowledged dial
# site, 2 = tooling error (fail-closed — an empty scan is NEVER clean).
set -uo pipefail

ALLOWLIST_DEFAULT=".context/checks/strict-star-allowlist"
ALLOWLIST="${STRICT_STAR_ALLOWLIST:-}"
JSON=0
QUIET=0
ROOTS=()

usage() {
    cat <<'EOF'
Usage: check-strict-star.sh [--json] [--quiet] [--root DIR]... [--allowlist FILE]

Guards "spokes never connect to one another" by requiring every raw dial site
in the spoke crates to be test-context or acknowledged with a cited reason.

  --json         machine-readable envelope
  --quiet        print only findings
  --root DIR     scan DIR instead of the default spoke crates (repeatable)
  --allowlist F  use F as the acknowledgement ledger
  --no-heartbeat accepted for guard-layer parity (this is not a cron canary)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1 ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-strict-star: --root needs a value" >&2; exit 2; }; ROOTS+=("$1") ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-strict-star: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1" ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-strict-star: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
    shift
done

if [ "${#ROOTS[@]}" -eq 0 ]; then
    # The spoke crates this check owns. Two crates are deliberately ABSENT:
    #
    #   termlink-hub     — the star's centre. Its outbound-dial edge is
    #                      hub-to-hub federation, guarded by T-2569's
    #                      no_federation_tripwire.rs. A different edge.
    #   termlink-session — guarded MORE STRONGLY than this script could, by
    #                      crates/termlink-session/tests/no_spoke_mesh_tripwire.rs
    #                      (T-2703), which pins the per-module connect counts
    #                      (client.rs 5, transport.rs 3, tofu.rs 1,
    #                      ws_consumer.rs 1) AND fires on any socket opened
    #                      outside those four modules. Scanning it here too
    #                      would mean two ledgers for one crate — the
    #                      divergence risk this repo keeps finding in
    #                      duplicated sources of truth.
    #
    # What is left is the genuine gap that tripwire does not reach: the CLI
    # and MCP surfaces, which T-2702 F1 named explicitly ("could be introduced
    # in termlink-session OR termlink-cli").
    ROOTS=(crates/termlink-cli/src crates/termlink-mcp/src)
fi

# Allowlist resolution, tracked-first (T-2681): an explicit flag/env always
# wins; otherwise the git-tracked ledger. A guard whose reported health
# depends on unversioned local state is a guard whose green is not evidence.
if [ -z "$ALLOWLIST" ]; then
    ALLOWLIST="$ALLOWLIST_DEFAULT"
fi

present_roots=()
for r in "${ROOTS[@]}"; do
    [ -d "$r" ] && present_roots+=("$r")
done
if [ "${#present_roots[@]}" -eq 0 ]; then
    echo "check-strict-star: none of the scan roots exist: ${ROOTS[*]}" >&2
    echo "  Refusing to report a clean scan over nothing (fail-closed)." >&2
    exit 2
fi

# --- acknowledgement ledger --------------------------------------------------
# Format: "<signature>  # <reason naming the target>"
declare -A ACK_REASON=()
ack_count=0
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in ''|\#*) continue ;; esac
        sig="${line%%#*}"
        reason="${line#*#}"
        # trim
        sig="$(printf '%s' "$sig" | sed 's/[[:space:]]*$//; s/^[[:space:]]*//')"
        reason="$(printf '%s' "$reason" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
        [ -n "$sig" ] || continue
        ACK_REASON["$sig"]="$reason"
        ack_count=$((ack_count+1))
    done < "$ALLOWLIST"
fi

# --- scan --------------------------------------------------------------------
# A "dial site" is a raw stream connect. These are the primitives a
# spoke-to-spoke channel would have to be built on; higher-level helpers all
# bottom out here.
DIAL_RE='(TcpStream::connect|UnixStream::connect|connect_timeout)'

checked=0
firing_lines=()
unacked=0
testctx=0

for root in "${present_roots[@]}"; do
    while IFS= read -r file; do
        [ -n "$file" ] || continue

        # First `#[cfg(test)]` in the file marks the start of test context.
        # Rust convention puts the test module last; anything at or after it
        # is a test connecting to a socket the test itself bound.
        cfgtest_line=$(grep -nE '^[[:space:]]*#\[cfg\(test\)\]' "$file" 2>/dev/null | head -1 | cut -d: -f1)
        [ -n "$cfgtest_line" ] || cfgtest_line=0

        while IFS= read -r hit; do
            [ -n "$hit" ] || continue
            lineno="${hit%%:*}"
            checked=$((checked+1))

            if [ "$cfgtest_line" -gt 0 ] && [ "$lineno" -ge "$cfgtest_line" ]; then
                testctx=$((testctx+1))
                continue
            fi

            # Drift-stable signature: <relpath>::<enclosing-fn>. Line numbers
            # move on every edit; a fn RENAME re-fires, which is the intended
            # re-review on meaningful change (same trade-off as the sibling
            # checks T-2527 / T-2531 / T-2666 / T-2672).
            fn=$(awk -v n="$lineno" '
                /^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]+[a-zA-Z0-9_]+/ {
                    if (NR <= n) {
                        line=$0
                        sub(/^[[:space:]]*/, "", line)
                        sub(/^pub[[:space:]]+/, "", line)
                        sub(/^\(crate\)[[:space:]]*/, "", line)
                        sub(/^async[[:space:]]+/, "", line)
                        sub(/^fn[[:space:]]+/, "", line)
                        sub(/[^a-zA-Z0-9_].*$/, "", line)
                        last=line
                    }
                }
                END { print last }
            ' "$file")
            [ -n "$fn" ] || fn="<toplevel>"

            sig="${file}::${fn}"
            if [ -n "${ACK_REASON[$sig]+set}" ]; then
                continue
            fi
            unacked=$((unacked+1))
            firing_lines+=("${file}|${lineno}|${fn}")
        done < <(grep -nE "$DIAL_RE" "$file" 2>/dev/null | grep -vE '^[0-9]+:[[:space:]]*//' | grep -vE '^[0-9]+:[[:space:]]*///')
    done < <(find "$root" -name '*.rs' -type f 2>/dev/null | sort)
done

if [ "$checked" -eq 0 ]; then
    # Zero dial sites across the whole spoke surface means the anchor stopped
    # matching, not that the code went quiet. Never report that as clean
    # (the T-2747 zero-tools lesson).
    echo "check-strict-star: scanned ${#present_roots[@]} root(s) and found ZERO dial sites." >&2
    echo "  The connect anchor has probably stopped matching. Refusing to report clean." >&2
    exit 2
fi

SCOPE="detects raw dial SITES and whether each is acknowledged; does not resolve what any runtime address points at"

if [ "$JSON" = "1" ]; then
    printf '{"ok":%s,"checked":%d,"test_context":%d,"acknowledged":%d,"unacknowledged":%d,"scope":"%s","firing":[' \
        "$([ "$unacked" -eq 0 ] && echo true || echo false)" \
        "$checked" "$testctx" "$ack_count" "$unacked" "$SCOPE"
    first=1
    for f in "${firing_lines[@]:-}"; do
        [ -n "$f" ] || continue
        IFS='|' read -r ff fl fn <<< "$f"
        [ "$first" = "1" ] || printf ','
        first=0
        printf '{"file":"%s","line":%s,"fn":"%s"}' "$ff" "$fl" "$fn"
    done
    printf ']}\n'
    [ "$unacked" -eq 0 ] && exit 0 || exit 1
fi

if [ "$unacked" -eq 0 ]; then
    if [ "$QUIET" != "1" ]; then
        echo "strict-star: clean — $checked dial site(s): $testctx test-context, $((checked-testctx)) acknowledged."
        echo "  invariant: spokes never connect to one another (substrate doc § 10)"
        echo "  scope: $SCOPE"
    fi
    exit 0
fi

echo "strict-star: $unacked unacknowledged dial site(s) of $checked scanned."
echo "  invariant: spokes never connect to one another (substrate doc § 10)"
echo "  scope: $SCOPE"
echo
for f in "${firing_lines[@]}"; do
    IFS='|' read -r ff fl fn <<< "$f"
    echo "  $ff:$fl  ($fn)"
done
echo
echo "Each dial site must be one of:"
echo "  - a connection to a HUB address (the star's centre) — the allowed edge;"
echo "  - a connection to a LOCAL session socket on this host (an operator tool"
echo "    reaching a local session is not the agent-to-agent mesh, T-2702);"
echo "  - test-context (inside #[cfg(test)])."
echo
echo "If it is one of those, acknowledge it in $ALLOWLIST as:"
echo "  <relpath>::<fn>  # <what the target is, and why it is not a peer spoke>"
echo "If it dials another SPOKE, it violates the invariant — remove it."
exit 1
