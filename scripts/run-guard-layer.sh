#!/usr/bin/env bash
# T-2684 (T-2683 F1/G1) — single entry point for the source-level guard layer.
#
# THE PROBLEM THIS CLOSES
#
# TermLink's guard layer splits cleanly in two, and only one half was automated:
#
#   runtime guards  — 17 cron canaries watching hubs, wakers, queues, claims.
#                     Wired to cron, they heartbeat, a meta-canary watches them.
#                     These genuinely run.
#   source guards   — the static checks that guard the CODE, plus the fixture
#                     suites that prove those checks can still fire.
#                     Executed by NOTHING.
#
# T-2683 established the second half with a tree-wide grep: no CI job, no cron
# entry, no git hook, no `fw doctor` step, and no aggregate runner invokes any of
# `check-alloc-sink-clamps` / `check-drain-sink-caps` / `check-silent-exit` /
# `check-busy-spin`. Their only references outside their own implementation are
# their own fixture suites and `.context/episodic/*` — the records of the tasks
# that created them. `release.yml` runs `cargo build --release` and never
# `cargo test`; `doc-lint.yml` runs 2 of 28 check scripts; the pre-push audit runs
# the `structure` section only.
#
# That is the exact disease the static checks were built to cure, one level up.
# From check-alloc-sink-clamps.sh's own header: "the convention is by discipline,
# not enforced — two instances in one window means the mechanism recurs." The
# check converts a convention into structure; but invoking the check was itself
# left to discipline, and nothing was watching that.
#
# This script is deliberately the SMALLEST thing that fixes it: one command that
# runs the layer. It does not schedule itself, does not install anything, and
# takes no position on where it should be wired in — CI, a git hook, a cron entry,
# or an agent typing it are all now possible, and none of them were before.
#
# MEMBERSHIP IS DECLARED, NOT GUESSED
#
# A static check joins the layer by carrying a marker in its own header:
#
#     # guard-layer: source [extra args...]
#
# `source` means "safe to run anywhere: no live hub, no network, no host state".
# Anything else (a runtime canary, a deploy-time host check) is not a member.
# The optional trailing args are how the check wants to be invoked here — chiefly
# `--no-heartbeat`, because a check that writes its cron heartbeat when run from
# this runner would mask a dead cron from the T-1723 meta-canary. Declaring the
# invocation next to the membership keeps those two facts from drifting apart.
#
# A `scripts/check-*.sh` with NO marker is reported as SKIP(unclassified) rather
# than silently ignored — a forgotten marker is exactly the shipped-but-dark
# condition this script exists to surface, so it stays visible. It does not fire:
# most unmarked checks are legitimately runtime canaries.
#
# Fixture suites (`tests/*fixtures*.sh`) are members by naming convention. They are
# hermetic by construction — each builds a scratch dir and drives its check through
# documented test seams (PL-213) — so there is nothing to declare.
#
# VERDICTS mirror the contract the layer already uses everywhere else:
#
#   PASS   rc 0   the guard ran and found nothing
#   FAIL   rc 1   the guard FIRED — a real finding, act on it
#   ERROR  rc 2   the guard could NOT RUN — tooling/environment, you know nothing
#   SKIP          not a member, or unclassified
#
# Keeping ERROR distinct from PASS is the whole point: a check that could not run
# must never read as a clean bill. (This is the same split T-2683 found the canary
# CRONTABS destroying with `2>&1` — see T-2685. Here it is preserved by construction.)
#
# ROLL-UP: any FAIL → exit 1. Else any ERROR → exit 2. Else 0. Findings dominate
# tooling errors, mirroring `fleet verify`'s "drift dominates".
#
# Exit codes: 0 all members passed · 1 a member fired · 2 a member errored, or the
#             runner could not enumerate members (fail-closed)
set -uo pipefail

SCRIPTS_DIR="${GUARD_LAYER_SCRIPTS_DIR:-scripts}"
TESTS_DIR="${GUARD_LAYER_TESTS_DIR:-tests}"
MEMBER_TIMEOUT="${GUARD_LAYER_TIMEOUT:-300}"
# Lines of a firing member's own output to echo inline. Exceeding it is reported
# explicitly (never a silent cap) along with the command that shows the rest.
OUTPUT_LINES="${GUARD_LAYER_OUTPUT_LINES:-12}"
FORMAT=human
QUIET=0
LIST_ONLY=0
WITH_TESTS=0

usage() {
    cat <<'EOF'
run-guard-layer.sh — run TermLink's source-level guard layer in one command.

Members:
  * every scripts/check-*.sh carrying a `# guard-layer: source` header marker
  * every tests/*fixtures*.sh (hermetic by convention)
  * every scripts/test-*.sh and non-*fixtures* tests/*.sh carrying the SAME marker
    (T-2779 — opt-in, because many of these need a live hub and would make a
     run-anywhere layer flaky; un-marked ones are listed as unclassified, not hidden)
  * optionally `cargo test --workspace` (--tests)

Usage: run-guard-layer.sh [OPTIONS]
  --tests        Also run `cargo test --workspace` (default OFF — the fast path
                 is seconds; the suite is minutes)
  --list         Print the discovered members and exit without running them
  --json         Emit {ok, members[], summary} instead of human output
  --quiet        Print only non-PASS members and the footer
  -h, --help     This help

Verdicts: PASS (rc 0) · FAIL (rc 1, guard fired) · ERROR (rc 2, guard could not
run) · SKIP (not a member / unclassified).

Exit: 0 all passed · 1 a member fired · 2 a member errored or enumeration failed.
Findings dominate tooling errors.

Test seams: GUARD_LAYER_SCRIPTS_DIR, GUARD_LAYER_TESTS_DIR (point at fixture
dirs), GUARD_LAYER_TIMEOUT (per-member seconds, default 300).
Fixtures: bash tests/guard-layer-runner-fixtures.sh
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --tests) WITH_TESTS=1; shift ;;
        --list)  LIST_ONLY=1; shift ;;
        --json)  FORMAT=json; shift ;;
        --quiet) QUIET=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "run-guard-layer: unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [ ! -d "$SCRIPTS_DIR" ]; then
    echo "run-guard-layer: scripts dir not found: $SCRIPTS_DIR" >&2
    exit 2
fi

# ---------------------------------------------------------------- discovery ---
# Parallel arrays; bash 3.2-compatible (no associative arrays — macOS ships 3.2).
m_name=(); m_kind=(); m_cmd=()
unclassified=()

# Static checks: opt in via the header marker, which also carries the invocation.
for f in "$SCRIPTS_DIR"/check-*.sh; do
    [ -e "$f" ] || continue
    marker="$(grep -m1 -E '^#[[:space:]]*guard-layer:[[:space:]]*source' "$f" 2>/dev/null || true)"
    if [ -z "$marker" ]; then
        unclassified+=("$(basename "$f")")
        continue
    fi
    # Everything after `source` is the declared invocation (usually --no-heartbeat).
    extra="$(printf '%s' "$marker" | sed -E 's/^#[[:space:]]*guard-layer:[[:space:]]*source[[:space:]]*//')"
    m_name+=("$(basename "$f")")
    m_kind+=("static-check")
    m_cmd+=("bash $f $extra")
done

# Fixture suites: members by naming convention, hermetic by construction.
if [ -d "$TESTS_DIR" ]; then
    for f in "$TESTS_DIR"/*fixtures*.sh; do
        [ -e "$f" ] || continue
        m_name+=("$(basename "$f")")
        m_kind+=("fixture-suite")
        m_cmd+=("bash $f")
    done
fi

# Suite-style tests that sit outside the fixture naming convention opt in exactly the
# way a static check does — by marker. Before T-2779 these were not merely un-run, they
# were INVISIBLE: the unclassified note counted only check-*.sh, so 38 suites under
# scripts/test-*.sh (plus the non-*fixtures* files in tests/) sat outside the accounting
# entirely, and "45/45 members clean" read as a statement about the whole guard layer.
# Marker-gated on purpose: many of these need a live hub / network and would make the
# layer flaky, violating its "safe to run anywhere" contract. Un-marked ones are now
# reported as unclassified — visible and countable, which is the actual fix.
for f in "$SCRIPTS_DIR"/test-*.sh "$TESTS_DIR"/*.sh; do
    [ -e "$f" ] || continue
    # *fixtures*.sh under TESTS_DIR already joined above by convention — never double-add.
    case "$(basename "$f")" in *fixtures*) continue ;; esac
    marker="$(grep -m1 -E '^#[[:space:]]*guard-layer:[[:space:]]*source' "$f" 2>/dev/null || true)"
    if [ -z "$marker" ]; then
        unclassified+=("$(basename "$f")")
        continue
    fi
    extra="$(printf '%s' "$marker" | sed -E 's/^#[[:space:]]*guard-layer:[[:space:]]*source[[:space:]]*//')"
    m_name+=("$(basename "$f")")
    m_kind+=("suite")
    m_cmd+=("bash $f $extra")
done

if [ "$WITH_TESTS" -eq 1 ]; then
    m_name+=("cargo test --workspace")
    m_kind+=("unit-tests")
    m_cmd+=("cargo test --workspace")
fi

total=${#m_name[@]}

if [ "$total" -eq 0 ]; then
    # Fail closed. An empty layer is never a clean bill — it means discovery broke
    # or every marker was lost, which is worse than any single guard firing.
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"no guard-layer members discovered","members":[],"summary":{"total":0}}\n'
    else
        echo "run-guard-layer: no members discovered under $SCRIPTS_DIR / $TESTS_DIR — enumeration failed" >&2
        echo "  A source-level check joins the layer with a '# guard-layer: source' header marker." >&2
    fi
    exit 2
fi

# ------------------------------------------------------------------- --list ---
if [ "$LIST_ONLY" -eq 1 ]; then
    if [ "$FORMAT" = json ]; then
        printf '{"ok":true,"members":['
        i=0
        while [ "$i" -lt "$total" ]; do
            [ "$i" -eq 0 ] || printf ','
            printf '{"name":%s,"kind":%s}' \
                "$(printf '%s' "${m_name[$i]}" | jq -R .)" \
                "$(printf '%s' "${m_kind[$i]}" | jq -R .)"
            i=$((i+1))
        done
        printf '],"summary":{"total":%s,"unclassified":%s}}\n' "$total" "${#unclassified[@]}"
    else
        echo "guard-layer members ($total):"
        i=0
        while [ "$i" -lt "$total" ]; do
            printf '  %-14s %s\n' "${m_kind[$i]}" "${m_name[$i]}"
            i=$((i+1))
        done
        if [ "${#unclassified[@]}" -gt 0 ]; then
            echo
            echo "unclassified checks + suites (${#unclassified[@]}) — no '# guard-layer:' marker:"
            for u in "${unclassified[@]}"; do echo "  $u"; done
        fi
    fi
    exit 0
fi

# -------------------------------------------------------------------- run -----
r_verdict=(); r_rc=()
pass_n=0; fail_n=0; err_n=0

i=0
while [ "$i" -lt "$total" ]; do
    out="$(timeout "$MEMBER_TIMEOUT" bash -c "${m_cmd[$i]}" 2>&1)"; rc=$?
    # `timeout` reports 124 on expiry — a member that hangs is a tooling error,
    # never a pass. Any rc outside {0,1} is treated as ERROR for the same reason:
    # an unexpected status means we do not know what the guard found.
    case "$rc" in
        0) verdict=PASS;  pass_n=$((pass_n+1)) ;;
        1) verdict=FAIL;  fail_n=$((fail_n+1)) ;;
        *) verdict=ERROR; err_n=$((err_n+1)) ;;
    esac
    r_verdict+=("$verdict"); r_rc+=("$rc")

    if [ "$FORMAT" = human ]; then
        if [ "$verdict" != PASS ] || [ "$QUIET" -eq 0 ]; then
            printf '  %-5s %-14s %s\n' "$verdict" "${m_kind[$i]}" "${m_name[$i]}"
        fi
        if [ "$verdict" != PASS ]; then
            # Surface the member's own words — the runner never paraphrases a
            # finding, so the operator acts on the guard's message, not ours.
            printf '%s\n' "$out" | sed -n "1,${OUTPUT_LINES}p" | sed 's/^/        │ /'
            # No silent caps: if the member said more than we showed, say so and
            # name the command that prints the rest. A truncated finding list that
            # looks complete is how a real finding gets missed.
            outlines=$(printf '%s\n' "$out" | wc -l)
            if [ "$outlines" -gt "$OUTPUT_LINES" ]; then
                echo "        │ … $((outlines - OUTPUT_LINES)) more line(s) suppressed — run: ${m_cmd[$i]}"
            fi
            [ "$rc" -eq 124 ] && echo "        │ (timed out after ${MEMBER_TIMEOUT}s)"
        fi
    fi
    i=$((i+1))
done

if [ "$fail_n" -gt 0 ]; then exit_rc=1
elif [ "$err_n" -gt 0 ]; then exit_rc=2
else exit_rc=0
fi

# ------------------------------------------------------------------ report ----
if [ "$FORMAT" = json ]; then
    printf '{"ok":%s,"members":[' "$([ "$exit_rc" -eq 0 ] && echo true || echo false)"
    i=0
    while [ "$i" -lt "$total" ]; do
        [ "$i" -eq 0 ] || printf ','
        printf '{"name":%s,"kind":%s,"rc":%s,"verdict":%s}' \
            "$(printf '%s' "${m_name[$i]}" | jq -R .)" \
            "$(printf '%s' "${m_kind[$i]}" | jq -R .)" \
            "${r_rc[$i]}" \
            "$(printf '%s' "${r_verdict[$i]}" | jq -R .)"
        i=$((i+1))
    done
    printf '],"summary":{"total":%s,"passed":%s,"fired":%s,"errored":%s,"unclassified":%s,"with_tests":%s,"exit_code":%s}}\n' \
        "$total" "$pass_n" "$fail_n" "$err_n" "${#unclassified[@]}" \
        "$([ "$WITH_TESTS" -eq 1 ] && echo true || echo false)" "$exit_rc"
    exit "$exit_rc"
fi

echo
if [ "$exit_rc" -eq 0 ]; then
    echo "guard layer: PASS — $pass_n/$total members clean"
elif [ "$exit_rc" -eq 1 ]; then
    echo "guard layer: FIRING — $fail_n guard(s) found something ($pass_n passed, $err_n errored)"
    echo "  A firing guard is a real finding. Act on the member's own message above."
else
    echo "guard layer: TOOLING ERROR — $err_n member(s) could not run ($pass_n passed)"
    echo "  This is NOT a clean bill: those guards found nothing because they never looked."
fi
if [ "${#unclassified[@]}" -gt 0 ] && [ "$QUIET" -eq 0 ]; then
    echo "  note: ${#unclassified[@]} check/suite script(s) carry no '# guard-layer:' marker and were not run"
    echo "        (runtime canaries belong to cron; live-hub suites cannot join a"
    echo "         run-anywhere layer — run --list to see them)"
fi
[ "$WITH_TESTS" -eq 0 ] && [ "$QUIET" -eq 0 ] && \
    echo "  note: cargo test --workspace not run — pass --tests to include it"
exit "$exit_rc"
