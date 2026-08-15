#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-release-artifact-drift.sh (T-2751, G-019 prevention for the install-path drift class)
#
# WHY: the list of release artifact names exists as THREE hand-maintained copies, and
# nothing verifies they agree.
#
#   1. install.sh                        case arms choosing which artifact to download
#   2. .github/workflows/release.yml     the assets actually published
#   3. homebrew/Formula/termlink.rb      download URLs (a deliberate SUBSET — see below)
#
# This is the shape T-2484 exists for — the charter sentence living as three copies with
# no transclusion — applied to release artifacts instead of prose. Rename a target in
# release.yml and install.sh keeps confidently reaching for a name that is no longer
# published.
#
# WHAT MAKES IT WORSE THAN ORDINARY DRIFT: install.sh is the FIRST option in README Quick
# Start (README.md:91-92), advertised as "no toolchain required" — the path a new user
# takes. It has zero CI coverage. `.github/workflows/install-check.yml` guards the THIRD
# option (`cargo install --git`), and its own header still calls that "the documented
# installation path", which is now stale. So the guarded path is the least-used one, while
# the unguarded one is piped into `sh` on a stranger's machine, where a rename surfaces as
# `die "failed to download"` on THEIR host rather than as a red build on ours.
#
# Same class as T-2683 (static checks that nothing ran) and T-2686 (`parity_topics`
# failing undetected since 2026-08-12): the artifact exists, is believed to work, and
# nothing executes or verifies it.
#
# WHAT: extract the three artifact-name sets and compare them.
#
#   install.sh  <-> release.yml   BIDIRECTIONAL. Both directions are real failures, and
#                                 they are distinct, so they are reported separately:
#       * offered-but-unpublished -- install.sh selects a name release.yml does not ship.
#         The user gets a 404 and a `die` on their own machine. Loudest for the user,
#         invisible to us.
#       * published-but-unreachable -- release.yml ships a name install.sh never selects.
#         Nobody sees an error: we pay to build and host a target the primary installer
#         cannot deliver. Silent, which is the Directive #2 shape.
#
#   formula     ->  release.yml   SUBSET ONLY. Every artifact the formula references must
#                                 be published (a formula pointing at a missing asset is a
#                                 broken `brew install`), but a published artifact the
#                                 formula omits does NOT fire: the gnu linux-x86_64 variant
#                                 is deliberately excluded in favour of the musl static one
#                                 (T-1135). Checking that direction as equality would fire
#                                 daily on a decision already made.
#
# SCOPE — this compares NAMES, and nothing else. It does not verify that a published asset
# is downloadable, that checksums.txt contains a line for it, or that the binary runs.
# A release that publishes all five names as empty files passes this check. It closes the
# rename/typo/forgot-to-add-a-target class only. Stated here because a guard reporting a
# bare green over a partially-examined surface is read as a statement about the whole
# surface (T-2680), and both output paths repeat this.
#
# There is deliberately NO allowlist. The sibling checks have one because their firing
# sets contain sites that are genuinely correct-but-unprovable-by-grep. Here, every
# mismatch is a real defect with a real fix (rename one side, or add the target), and the
# firing set today is empty. An allowlist would only ever be used to silence a genuine
# break.
#
# NOT a runtime cron canary — a source-level static check, sibling of
# check-alloc-sink-clamps.sh (T-2527), check-drain-sink-caps.sh (T-2531),
# check-silent-exit.sh (T-2666), check-busy-spin.sh (T-2672), check-platform-lock.sh
# (T-2693), check-error-code-emission.sh (T-2699), check-version-derivation.sh (T-2746)
# and check-mcp-parity-census.sh (T-2747).
#
# EXIT CODES:
#   0  clean    -- the three lists agree (formula as a subset).
#   1  firing   -- >=1 name mismatch.
#   2  tooling  -- a source file is missing/unparseable, or a set came back EMPTY.
#                  An empty set is a tooling error, never a clean census: "0 artifacts
#                  agree with 0 artifacts" is vacuously true and would report green over a
#                  parse that silently stopped working (the T-2747 zero-tools lesson).
#
# USAGE:
#   check-release-artifact-drift.sh [--json] [--quiet] [--no-heartbeat]
#                                   [--install-sh <file>] [--release-yml <file>]
#                                   [--formula <file>]
#     --json           emit {ok, firing:[{name,why}], published[], offered[],
#                      formula[], published_count, scope}
#     --quiet          print only on firing (cron mode); clean prints nothing
#     --no-heartbeat   skip the heartbeat touch (guard-layer runner invokes with this)
#     --install-sh <f> / --release-yml <f> / --formula <f>
#                      override input paths (fixtures point these at scratch files)
#
# Origin: T-2751, from investigating herdr backlog rank 20 — which proposed BUILDING a
# `curl | sh` installer that already existed. Load-bearing proof:
# tests/release-artifact-drift-fixtures.sh, and renaming a target on either side re-fires.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
INSTALL_SH="${RELEASE_DRIFT_INSTALL_SH:-install.sh}"
RELEASE_YML="${RELEASE_DRIFT_RELEASE_YML:-.github/workflows/release.yml}"
FORMULA="${RELEASE_DRIFT_FORMULA:-homebrew/Formula/termlink.rb}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --install-sh) shift; [ $# -gt 0 ] || { echo "check-release-artifact-drift: --install-sh needs a value" >&2; exit 2; }; INSTALL_SH="$1"; shift ;;
        --release-yml) shift; [ $# -gt 0 ] || { echo "check-release-artifact-drift: --release-yml needs a value" >&2; exit 2; }; RELEASE_YML="$1"; shift ;;
        --formula) shift; [ $# -gt 0 ] || { echo "check-release-artifact-drift: --formula needs a value" >&2; exit 2; }; FORMULA="$1"; shift ;;
        -h|--help) sed -n '2,95p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-release-artifact-drift: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-release-artifact-drift: grep not found (required)" >&2; exit 2; }

for f in "$INSTALL_SH" "$RELEASE_YML"; do
    [ -f "$f" ] || { echo "check-release-artifact-drift: input not found: $f" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.release-artifact-drift-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# ---- extraction -------------------------------------------------------------------
#
# Each extractor strips `#`-comment lines first. A commented-out target is not a target,
# and this file's own header names artifacts in prose — parsing those would inflate every
# set identically and mask a real mismatch by making both sides wrong the same way.

# install.sh: the case arms that pick a download.  artifact="termlink-<triple>"
offered="$(grep -vE '^[[:space:]]*#' "$INSTALL_SH" 2>/dev/null \
           | grep -oE 'artifact="termlink-[A-Za-z0-9_.-]+"' \
           | sed -E 's/^artifact="//; s/"$//' \
           | sort -u || true)"

# release.yml: the publish block.  dist/termlink-<triple>
# checksums.txt is published too but is not a binary artifact — install.sh fetches it
# unconditionally by a separate path, so it is not part of the per-target name set.
published="$(grep -vE '^[[:space:]]*#' "$RELEASE_YML" 2>/dev/null \
             | grep -oE 'dist/termlink-[A-Za-z0-9_.-]+' \
             | sed -E 's#^dist/##' \
             | sort -u || true)"

# formula: only `url "...releases/download/.../termlink-<triple>"` lines.
# Deliberately NOT any `termlink-*` token: the formula also contains
# `Dir["termlink-*"].first`, a runtime glob that is not an artifact name.
formula=""
formula_present=0
if [ -f "$FORMULA" ]; then
    formula_present=1
    formula="$(grep -vE '^[[:space:]]*#' "$FORMULA" 2>/dev/null \
               | grep -E 'url[[:space:]]+"' \
               | grep -oE 'releases/download/[^"]*/termlink-[A-Za-z0-9_.-]+' \
               | sed -E 's#.*/##' \
               | sort -u || true)"
fi

# Fail closed on an empty set. "0 agrees with 0" is vacuously true and would report a
# clean census over a parse that silently stopped matching (T-2747 zero-tools lesson).
if [ -z "$offered" ]; then
    echo "check-release-artifact-drift: extracted ZERO artifact names from $INSTALL_SH — refusing to report clean (parse likely broken)" >&2
    exit 2
fi
if [ -z "$published" ]; then
    echo "check-release-artifact-drift: extracted ZERO published assets from $RELEASE_YML — refusing to report clean (parse likely broken)" >&2
    exit 2
fi

# ---- comparison -------------------------------------------------------------------

firing_lines=""
fire() { firing_lines+="$1|$2"$'\n'; }

in_set() { printf '%s\n' "$2" | grep -qxF "$1"; }

offered_count=0
while IFS= read -r a; do
    [ -n "$a" ] || continue
    offered_count=$((offered_count + 1))
    in_set "$a" "$published" || \
        fire "$a" "install.sh offers it but release.yml does not publish it — users get a 404 and a die on their own machine"
done <<< "$offered"

published_count=0
while IFS= read -r a; do
    [ -n "$a" ] || continue
    published_count=$((published_count + 1))
    in_set "$a" "$offered" || \
        fire "$a" "release.yml publishes it but install.sh never selects it — a target built and hosted that the primary installer cannot deliver"
done <<< "$published"

formula_count=0
if [ "$formula_present" -eq 1 ] && [ -n "$formula" ]; then
    while IFS= read -r a; do
        [ -n "$a" ] || continue
        formula_count=$((formula_count + 1))
        in_set "$a" "$published" || \
            fire "$a" "homebrew formula points at it but release.yml does not publish it — brew install fetches a missing asset"
    done <<< "$formula"
fi

firing_count=0
[ -n "$firing_lines" ] && firing_count="$(printf '%s' "$firing_lines" | grep -c . || true)"

SCOPE="compares artifact NAMES across install.sh / release.yml / formula; does NOT verify an asset is downloadable or that the binary runs"

# ---- output -----------------------------------------------------------------------

json_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

json_array() {
    local first=1 out="["
    while IFS= read -r a; do
        [ -n "$a" ] || continue
        [ "$first" -eq 1 ] || out+=","
        first=0
        out+="\"$(json_escape "$a")\""
    done <<< "$1"
    printf '%s]' "$out"
}

if [ "$WANT_JSON" -eq 1 ]; then
    ok=true; [ "$firing_count" -gt 0 ] && ok=false
    printf '{"ok":%s,"firing":[' "$ok"
    first=1
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        name="${line%%|*}"; why="${line#*|}"
        [ "$first" -eq 1 ] || printf ','
        first=0
        printf '{"name":"%s","why":"%s"}' "$(json_escape "$name")" "$(json_escape "$why")"
    done <<< "$firing_lines"
    printf '],"published":%s' "$(json_array "$published")"
    printf ',"offered":%s' "$(json_array "$offered")"
    printf ',"formula":%s' "$(json_array "$formula")"
    printf ',"published_count":%d,"offered_count":%d,"formula_count":%d' \
        "$published_count" "$offered_count" "$formula_count"
    printf ',"scope":"%s"}\n' "$(json_escape "$SCOPE")"
    [ "$firing_count" -gt 0 ] && exit 1
    exit 0
fi

if [ "$firing_count" -gt 0 ]; then
    echo "check-release-artifact-drift: FIRING — $firing_count artifact-name mismatch(es)"
    echo
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        name="${line%%|*}"; why="${line#*|}"
        printf '  %s\n      %s\n' "$name" "$why"
    done <<< "$firing_lines"
    echo
    echo "  census: $published_count published, $offered_count offered by install.sh, $formula_count in formula"
    echo "  scope: $SCOPE"
    echo
    echo "  Fix: make the names agree. install.sh:70-81 (case arms),"
    echo "       .github/workflows/release.yml (publish block), homebrew/Formula/termlink.rb (url lines)."
    echo "       The formula is a deliberate SUBSET (gnu variant excluded, T-1135) — that direction never fires."
    exit 1
fi

if [ "$QUIET" -eq 0 ]; then
    echo "check-release-artifact-drift: clean — $published_count published asset(s), all offered by install.sh and vice versa; $formula_count formula reference(s) all published"
    echo "  scope: $SCOPE"
fi
exit 0
