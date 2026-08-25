#!/usr/bin/env bash
# fw-upgrade-safe.sh — run `fw upgrade` without silently losing consumer work.
#
# WHY (T-2015, measured on four consecutive clean-tree runs of this repo)
# ----------------------------------------------------------------------
# `fw upgrade` rewrites consumer-owned files from vendored templates. For
# CLAUDE.md it diffs and warns ("N line(s) ... are absent"). For
# .claude/commands/ and scripts/ it prints only:
#
#     UPDATED  scripts/agent-send.sh (backup: .bak)
#
# ...while removing hundreds of lines. Measured here: 15 files, 2058 deletions,
# IDENTICAL on every run. Among the losses are deliverables of tasks still
# marked work-completed with their ACs ticked — T-2402's Stage 6 wake protocol,
# T-2295's delivery-confirming path, T-2091's capability flags. The recovery
# window is one upgrade wide: `.bak` is the only witness for uncommitted work,
# and the next upgrade overwrites `.bak`.
#
# WHAT THIS DOES
#   1. Refuses to run if the consumer surfaces are dirty. Restore-from-HEAD is
#      only safe when HEAD is what you want back; uncommitted work would be
#      destroyed by the step meant to protect it.
#   2. Snapshots canary strings — grep-able evidence cited by closed tasks.
#   3. Runs `fw upgrade`, passing arguments through.
#   4. Reports what was destroyed, per file.
#   5. Reports any INCOMING line absent from HEAD. A file carrying genuinely new
#      upstream content is HELD, not auto-restored — that decision stays human.
#   6. Restores the pure-loss files and verifies the canaries came back.
#
# Deliberately does NOT touch .agentic-framework/ or CLAUDE.md. The vendored
# tree may legitimately be replaced by a newer one (decide by CONTENT — VERSION
# is a resetting counter and cannot order two trees), and CLAUDE.md already has
# a safe zone above `## Core Principle`.
#
# Exit: 0 clean | 1 refused (dirty) | 2 new upstream content needs a decision
#       | 3 a canary did not come back
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

SURFACES=(scripts .claude/commands)

# "<file>:<needle>:<expected-min-count>"
CANARIES=(
  ".claude/commands/check-arc.md:Drain ALL unread:1"
  ".claude/commands/peers.md:filter-capability:5"
  ".claude/commands/agent-handoff.md:T-2295:1"
)

say() { printf '%s\n' "$*"; }
hr() { printf -- '---------------------------------------------------------------\n'; }

canary_count() { grep -c -- "$2" "$1" 2>/dev/null || true; }

# Lines present in the working copy but nowhere in the committed version.
# Whitespace-normalised and sorted so pure reflowing is not reported as new.
new_lines() {
  comm -13 \
    <(git show "HEAD:$1" 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' | sort -u) \
    <(sed 's/^[[:space:]]*//;s/[[:space:]]*$//' "$1" | grep -v '^$' | sort -u) \
    | wc -l | tr -d ' '
}

# --- 1. refuse on a dirty tree ------------------------------------------------
dirty=$(git status --porcelain -- "${SURFACES[@]}" | grep -v '\.bak$' || true)
if [ -n "$dirty" ]; then
  say "fw-upgrade-safe: REFUSING — consumer surfaces have uncommitted changes."
  say "  A restore-from-HEAD would destroy them. Commit or stash first:"
  printf '%s\n' "$dirty" | sed 's/^/    /'
  exit 1
fi

# --- 2. snapshot --------------------------------------------------------------
say "fw-upgrade-safe: pre-flight"
for c in "${CANARIES[@]}"; do
  f=${c%%:*}; rest=${c#*:}; needle=${rest%:*}; want=${rest##*:}
  say "  canary $f '$needle' = $(canary_count "$f" "$needle") (expect >= $want)"
done
hr

# --- 3. upgrade ---------------------------------------------------------------
say "fw-upgrade-safe: running fw upgrade $*"
.agentic-framework/bin/fw upgrade "$@"
hr

# --- 4/5. what was destroyed; is anything genuinely new? ----------------------
changed=$(git diff --name-only -- "${SURFACES[@]}" || true)
if [ -z "$changed" ]; then
  say "fw-upgrade-safe: consumer surfaces untouched. Nothing to restore."
  exit 0
fi

say "fw-upgrade-safe: consumer files rewritten by the upgrade:"
git diff --stat -- "${SURFACES[@]}" | sed 's/^/  /'
hr

needs_human=0
hold=""
restore=""
for f in $changed; do
  n=$(new_lines "$f")
  if [ "${n:-0}" -gt 0 ]; then
    say "  HOLD  $f — carries $n line(s) absent from HEAD; NOT auto-restored."
    say "        review with: git diff -- $f"
    needs_human=1
    hold="$hold $f"
  else
    restore="$restore $f"
  fi
done

# --- 6. restore pure-loss files, verify canaries ------------------------------
if [ -n "$restore" ]; then
  # shellcheck disable=SC2086
  say "fw-upgrade-safe: restoring $(echo $restore | wc -w) pure-loss file(s) from HEAD"
  # shellcheck disable=SC2086
  git checkout -- $restore
fi

hr
rc=0
for c in "${CANARIES[@]}"; do
  f=${c%%:*}; rest=${c#*:}; needle=${rest%:*}; want=${rest##*:}
  have=$(canary_count "$f" "$needle")
  if [ "${have:-0}" -lt "$want" ]; then
    say "  CANARY LOST  $f '$needle' = $have (expected >= $want)"
    rc=3
  else
    say "  canary ok    $f '$needle' = $have"
  fi
done

if [ "$rc" -ne 0 ]; then exit "$rc"; fi
if [ "$needs_human" -ne 0 ]; then
  say "fw-upgrade-safe: HELD file(s) need a decision:$hold"
  exit 2
fi
say "fw-upgrade-safe: consumer work intact."
exit 0
