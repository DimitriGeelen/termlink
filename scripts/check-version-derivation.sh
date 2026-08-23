#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-version-derivation.sh (T-2746, G-019 prevention for the T-1458 / T-2744 class)
#
# WHY: a crate that reads `env!("CARGO_PKG_VERSION")` gets its own Cargo.toml version
# unless a build.rs overrides it with the git-derived one. When the override is missing
# the crate does not fail — it reports a plausible WRONG version, forever, which is the
# Directive #2 shape (a wrong answer, not an error).
#
# This has now happened twice, three months apart, in two different crates:
#
#   T-1458 (2026-05-03)  termlink-hub    hub.version RPC returned "0.9.0" for every
#                                        fleet hub regardless of binary freshness.
#   T-2744 (2026-08-15)  termlink-session every session ever registered recorded
#                                        "0.9.0" in its metadata while the binary
#                                        reporting it was at 0.11.720.
#
# The first instance produced PL-148, which is accurate, specific, and even names the
# detection lever ("if a fleet-wide version histogram shows uniform '0.9.0', or matches
# the workspace Cargo.toml literal exactly, suspect a missing build.rs"). It did not
# prevent the second instance. A learning that precise failing to prevent a recurrence
# is the argument for a structural check rather than more documentation.
#
# PL-148 also names why the obvious test does NOT catch this:
#
#     assert_eq!(reported_version, env!("CARGO_PKG_VERSION"));
#
# passes whether env! resolves to "0.9.0" or "0.11.1359", because both sides are the
# same compile-time constant. The bug is invisible from inside the crate that has it.
# That is what makes it a job for a check that reads the build configuration from the
# outside, not for a unit test.
#
# WHAT: for each crate under the scan root, decide two things independently —
#   (1) does the crate READ the version?  any non-comment `env!("CARGO_PKG_VERSION")`
#       or `option_env!("CARGO_PKG_VERSION")` in its sources (build.rs excluded — a
#       build script reading its own Cargo.toml value is the mechanism, not a surface)
#   (2) does the crate DERIVE the version?  a build.rs containing a non-comment
#       `cargo:rustc-env=CARGO_PKG_VERSION`
# A crate that reads but does not derive is firing. A crate that does neither is not
# examined at all — this check never says "every crate must have a build.rs".
#
# SCOPE — this detects a MISSING derivation, not a WRONG one. It reads whether the
# emit line is present; it does not evaluate what the build script computes. A build.rs
# that emits a bogus version passes. The regression test added by T-2744
# (`recorded_version_is_the_build_version_not_the_cargo_toml_constant`) covers that
# other half from inside the crate, by comparing against a separately-exported copy of
# the Cargo.toml value rather than a literal — which is how it escapes the PL-148
# tautology above. The two are complements; neither alone is sufficient.
#
# A test-only read (`assert_eq!(x, env!("CARGO_PKG_VERSION"))` inside `#[cfg(test)]`)
# is treated as a read. That is deliberate rather than an accepted false positive: a
# test-only read is exactly the tautological assertion PL-148 warns about, so a crate
# whose only use is that one is a crate whose version cannot be verified from inside.
# If a crate genuinely wants the Cargo.toml value, allowlist it and say why.
#
# ALLOWLIST: `.context/checks/version-derivation-allowlist` (git-tracked per T-2681 —
# a guard whose reported health depends on unversioned local state is a guard whose
# green is not evidence). One crate name per line, `<crate>  # <reason>`. Entries are
# counted and REPORTED but do not fire; removing a line re-fires that crate. The reason
# must say why the Cargo.toml version is the right answer for that crate.
#
# NOT a runtime cron canary — a source-level static check, sibling of
# check-alloc-sink-clamps.sh (T-2527), check-drain-sink-caps.sh (T-2531),
# check-silent-exit.sh (T-2666), check-busy-spin.sh (T-2672), check-platform-lock.sh
# (T-2693) and check-error-code-emission.sh (T-2699).
#
# EXIT CODES:
#   0  clean    -- every version-reading crate derives its version (or is allowlisted).
#   1  firing   -- >=1 crate reads the version without deriving it.
#   2  tooling  -- missing dep / bad scan root.
#
# USAGE:
#   check-version-derivation.sh [--json] [--quiet] [--no-heartbeat]
#                               [--root <dir>]... [--allowlist <file>]
#     --json          emit {ok, firing:[{crate,why}], checked, candidates,
#                     acknowledged_count, acknowledged[]}
#     --quiet         print only on firing (cron mode); clean prints nothing
#     --no-heartbeat  skip the heartbeat touch (guard-layer runner invokes with this)
#     --root <dir>    override scan root (repeatable; default = crates/)
#     --allowlist <f> override allowlist path (fixtures point this at a scratch file)
#
# Origin: T-2746. Load-bearing proof: tests/version-derivation-check-fixtures.sh, and
# neutering the emit line in crates/termlink-session/build.rs re-fires the check.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
ROOTS=()
# T-2681 — tracked-first allowlist resolution; see the header of
# .context/checks/alloc-sink-allowlist for why. Explicit VERSION_DERIVATION_ALLOWLIST
# or --allowlist always wins.
ALLOWLIST="${VERSION_DERIVATION_ALLOWLIST:-.context/checks/version-derivation-allowlist}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-version-derivation: --root needs a value" >&2; exit 2; }; ROOTS+=("$1"); shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-version-derivation: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,86p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-version-derivation: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-version-derivation: grep not found (required)" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(crates)
fi
for r in "${ROOTS[@]}"; do
    [ -d "$r" ] || { echo "check-version-derivation: scan root not found: $r" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.version-derivation-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# A read of the compiled-in version. build.rs is excluded by path below: a build script
# reading CARGO_PKG_VERSION from its own environment is how the derivation works.
READ_RE='(env!|option_env!)\("CARGO_PKG_VERSION"\)'
# The emit that overrides it for the crate being built.
EMIT_RE='cargo:rustc-env=CARGO_PKG_VERSION'

declare -A ALLOW=()
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        line="${line%%#*}"
        line="$(printf '%s' "$line" | tr -d '[:space:]')"
        [ -n "$line" ] || continue
        ALLOW["$line"]=1
    done < "$ALLOWLIST"
fi

checked=0
firing_lines=""
ack_lines=""

for root in "${ROOTS[@]}"; do
    for crate_dir in "$root"/*/; do
        [ -d "$crate_dir" ] || continue
        crate="$(basename "$crate_dir")"

        # (1) does it READ the version? Comment-only lines are stripped: prose *about*
        # env!("CARGO_PKG_VERSION") — of which this repo has plenty — is not a read of it.
        reads="$(grep -rnE "$READ_RE" --include='*.rs' "$crate_dir" 2>/dev/null \
                 | grep -vE '/build\.rs:' \
                 | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)"
        [ -n "$reads" ] || continue

        checked=$((checked + 1))

        # (2) does it DERIVE the version?
        bs="${crate_dir}build.rs"
        why=""
        if [ ! -f "$bs" ]; then
            why="reads the version but has no build.rs — reports its Cargo.toml constant"
        else
            bs_body="$(grep -vE '^[[:space:]]*//' "$bs" 2>/dev/null || true)"
            if ! printf '%s' "$bs_body" | grep -q "$EMIT_RE"; then
                why="has a build.rs but it never emits $EMIT_RE"
            fi
        fi
        [ -n "$why" ] || continue

        if [ -n "${ALLOW[$crate]:-}" ]; then
            ack_lines="${ack_lines}${crate}|${why}"$'\n'
        else
            firing_lines="${firing_lines}${crate}|${why}"$'\n'
        fi
    done
done

fire_count="$(printf '%s' "$firing_lines" | grep -c . || true)"
ack_count="$(printf '%s' "$ack_lines" | grep -c . || true)"

to_json_array() {
    if command -v jq >/dev/null 2>&1; then
        printf '%s' "$1" | grep . | \
            sed -E 's/^([^|]*)\|(.*)$/{"crate":"\1","why":"\2"}/' | \
            jq -sc '.' 2>/dev/null || echo '[]'
    else
        echo '[]'
    fi
}

if [ "${fire_count:-0}" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        printf '{"ok":true,"firing":[],"checked":%d,"candidates":0,"acknowledged_count":%d,"acknowledged":%s}\n' \
            "$checked" "$ack_count" "$(to_json_array "$ack_lines")"
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-version-derivation: clean — every version-reading crate derives its version ($checked scanned, $ack_count acknowledged)."
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    printf '{"ok":false,"firing":%s,"checked":%d,"candidates":%d,"acknowledged_count":%d,"acknowledged":%s}\n' \
        "$(to_json_array "$firing_lines")" "$checked" "$fire_count" "$ack_count" "$(to_json_array "$ack_lines")"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-version-derivation: FIRING — $fire_count crate(s) read the version without deriving it:"
    printf '%s' "$firing_lines" | grep . | sed -E 's/^([^|]*)\|(.*)$/  ↳ \1: \2/'
    echo "  Such a crate does not fail — it reports a plausible wrong version indefinitely"
    echo "  (T-1458 hub.version, T-2744 session metadata; both read 0.9.0 for months)."
    echo "  Fix: add a build.rs emitting the git-derived version — copy one from a sibling"
    echo "  that has it; crates/termlink-session/build.rs documents the re-run triggers (T-1057)."
    echo "  OR — if the Cargo.toml version genuinely IS the right answer for that crate —"
    echo "  add it to $ALLOWLIST with a reason saying why."
    echo "---"
fi
exit 1
