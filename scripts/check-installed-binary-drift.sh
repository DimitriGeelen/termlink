#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-installed-binary-drift.sh — is the fix you landed actually RUNNING?
#
# WHY (measured on dimitrimintdev, 2026-08-26)
# -------------------------------------------
# "Deployed" had no single answer on this host. Three termlink binaries existed
# in three different states, and different consumers used different ones:
#
#   /root/.cargo/bin/termlink   0.11.1196   4 systemd units, and `which termlink`
#   /root/.local/bin/termlink   0.11.693    the running MCP server
#   /usr/local/bin/termlink     0.11.693    the vendored chat-arc heartbeat cron
#   (repo build)                0.11.1600
#
# ~900 versions of drift, nothing reconciling them. Three CLI behaviour fixes had
# been committed, pushed and reported as landed while every consumer on the box
# still ran code that predated all of them. `git log` cannot see this; `--version`
# on ONE path cannot see it either, because it answers for the path you happened
# to pick.
#
# check-release-artifact-drift.sh (T-2751) covers a DIFFERENT thing — that the
# release artifact NAMES agree across install.sh, the workflow and homebrew. It
# says nothing about which binary a given host actually executes.
#
# WHAT THIS ASSERTS, AND WHAT IT DOES NOT
# ---------------------------------------
# It states its predicate. It reports exactly which paths it probed, so "no drift"
# can never mean "I looked in one place". A short list because something was
# skipped looks identical to a short list because nothing was wrong — that
# distinction is the whole point of this script and of G-019.
#
# It does NOT assert that any particular version is correct, only that the
# installed set disagrees (or does not) with itself and with the repo build.
#
# Exit: 0 all probed paths agree with each other and with --expect (if given)
#       1 drift between installed paths
#       2 installed paths agree but differ from --expect
#       3 no termlink binary found on any probed path (a check that found nothing
#         to check is NOT a pass)
set -uo pipefail

# Every location a consumer on this host has been observed to execute.
# Adding a path here is the only way this check widens; it never guesses.
PROBE_PATHS=(
  /root/.cargo/bin/termlink
  /root/.local/bin/termlink
  /usr/local/bin/termlink
  /usr/bin/termlink
  /opt/termlink/target/release/termlink
)

EXPECT=""
[ "${1:-}" = "--expect" ] && EXPECT="${2:-}"

say() { printf '%s\n' "$*"; }

say "check-installed-binary-drift: probing ${#PROBE_PATHS[@]} known install path(s)"
say ""
printf '  %-42s %-22s %s\n' "PATH" "VERSION" "MTIME"
printf '  %-42s %-22s %s\n' "----" "-------" "-----"

found=0
versions=()
for p in "${PROBE_PATHS[@]}"; do
  if [ ! -x "$p" ]; then
    printf '  %-42s %-22s %s\n' "$p" "(absent)" "-"
    continue
  fi
  v=$("$p" --version 2>/dev/null | head -1 | awk '{print $NF}')
  [ -n "$v" ] || v="(unreadable)"
  m=$(stat -c %y "$p" 2>/dev/null | cut -d' ' -f1)
  printf '  %-42s %-22s %s\n' "$p" "$v" "$m"
  found=$((found + 1))
  versions+=("$v")
done

say ""
say "  PREDICATE: ${found} of ${#PROBE_PATHS[@]} probed paths carry an executable binary."
say "             Paths not listed above were NOT examined. This check asserts"
say "             nothing about install locations it does not know about."

if [ "$found" -eq 0 ]; then
  say ""
  say "  FAIL(3): no termlink binary on any probed path. This is 'nothing was"
  say "           looked at', not 'nothing is wrong'."
  exit 3
fi

distinct=$(printf '%s\n' "${versions[@]}" | sort -u | tr '\n' ' ' | sed 's/ $//')
n_distinct=$(printf '%s\n' "${versions[@]}" | sort -u | wc -l)

say ""
if [ "$n_distinct" -gt 1 ]; then
  say "  DRIFT(1): ${n_distinct} distinct versions across installed paths: $distinct"
  say "            Different consumers on this host execute different code. A fix"
  say "            landed in git is running for some callers and not others."
  exit 1
fi

say "  installed paths agree: $distinct"

if [ -n "$EXPECT" ]; then
  if [ "$distinct" != "$EXPECT" ]; then
    say ""
    say "  STALE(2): installed=$distinct but expected=$EXPECT"
    say "            The installed set is self-consistent and behind the build."
    exit 2
  fi
  say "  matches --expect: $EXPECT"
fi
exit 0
