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

# Non-empty line count of the committed version vs the working copy.
#
# HOLD/RESTORE is decided by DIRECTION OF SIZE, not by presence of novel lines.
# v1 of this script held any file containing a line absent from HEAD; the live
# test held 10 of 15 files and lost all three canaries. The reason is measured:
# `fw upgrade` replaces our files with the LEANER upstream template
# (scripts/agent-send.sh 717 lines -> 229), and the "new" lines are reformatting
# and rewording of features we already carry — every one of ours was a superset.
# Novelty is therefore a bad signal here; size direction is a good one.
head_lines()     { git show "HEAD:$1" 2>/dev/null | grep -cv '^[[:space:]]*$' || true; }
working_lines()  { grep -cv '^[[:space:]]*$' "$1" 2>/dev/null || true; }

# Does this file carry a canary? Canaries outrank the size heuristic: restoring
# is what brings back a closed task's cited evidence.
has_canary() {
  for c in "${CANARIES[@]}"; do
    [ "${c%%:*}" = "$1" ] && return 0
  done
  return 1
}

# --- 0. flags the handoff will silently discard (T-2016) ---------------------
#
# `fw upgrade`'s bare-from-consumer auto-clone path rebuilds its argv by hand
# (lib/upgrade.sh:1068-1070) and replays only --force and --dedupe-user-hooks.
# Everything else the operator typed is dropped without a word. That path is
# only taken when the upgrade hands off to a cloned upstream, so this warns
# rather than refuses — but it warns BEFORE the run, which is the difference
# between noticing and finding out afterwards.
#
# Measured against the vendored tree, not assumed. --dry-run is deliberately
# NOT listed: it short-circuits before the replay is built and is safe.
# --from-upstream is excluded on purpose and documented in-source.
DROPPED_BY_HANDOFF=(--force-downgrade --strict --no-self-vendor)
_dropped=""
for _a in "$@"; do
  for _d in "${DROPPED_BY_HANDOFF[@]}"; do
    [ "$_a" = "$_d" ] && _dropped="$_dropped $_a"
  done
done
if [ -n "$_dropped" ]; then
  say "fw-upgrade-safe: WARNING — flag(s) dropped if this run hands off to a"
  say "  cloned upstream (T-2016, lib/upgrade.sh:1068):$_dropped"
  say "  The handoff replays only --force and --dedupe-user-hooks. --strict"
  say "  becoming continue-on-error is the one that bites quietly."
  say "  Workaround: invoke upstream bin/fw directly with FRAMEWORK_ROOT and"
  say "  PROJECT_ROOT set, bypassing the bare-from-consumer path."
  hr
fi

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
  h=$(head_lines "$f"); w=$(working_lines "$f")
  if has_canary "$f"; then
    say "  restore $f — carries a closed task's cited evidence ($h -> $w lines)"
    restore="$restore $f"
  elif [ "${w:-0}" -lt "${h:-0}" ]; then
    say "  restore $f — incoming is smaller ($h -> $w lines); pure-loss direction"
    restore="$restore $f"
  else
    say "  HOLD    $f — incoming is NOT smaller ($h -> $w lines); may carry real"
    say "          additions. Not auto-restored. Review: git diff -- $f"
    needs_human=1
    hold="$hold $f"
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
