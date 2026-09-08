#!/usr/bin/env bash
# T-2929 (G-019 prevention for the T-2683 / T-2928 class) — guard-runner coverage.
#
# `run-guard-layer.sh` reports N check/suite scripts as "unclassified" because they
# carry no `# guard-layer: source` marker. That word is doing too much work. It says
# only "this script did not declare itself a member of the run-anywhere layer" — it
# says NOTHING about whether anything, anywhere, executes it. Read as "unrun" it
# overstates the problem; read as "fine, cron has it" it understates it. Measured on
# this tree the split was 36 covered / 48 dormant against 85 unclassified, so both
# readings were wrong by a wide margin.
#
# Why it matters, concretely. T-2928: `check-pickup-cron-lock.sh` was correct, exited
# 1 against the live host, and named both unflocked cron lines — for a full day, while
# the defect it guards fired four times and created duplicate task files. No cron, no
# CI job, and no caller ever ran it. The only thing referencing it was its own fixture
# suite, which exercised it against synthetic input and went green. A guard that is
# green against fixtures and never run against the tree it guards is not a guard.
#
# CLASSES (a script gets exactly one):
#   COVERED       named in an INSTALLED /etc/cron.d file, a CI workflow, or a live
#                 caller — actually executed by something
#   SHIPPED-DARK  named in a git-tracked .context/cron/*.crontab but NOT installed.
#                 Reported, deliberately NON-FIRING: that is the T-2561/T-2682
#                 install-drift class, already owned by check-cron-install-drift.sh.
#                 Firing here too would double-own it and misroute the remediation.
#   OPERATOR-INVOKED  backs a slash command under .claude/commands/. Reported,
#                 deliberately NON-FIRING: a human runs it on demand, which is a real
#                 invocation path — it is simply not a scheduled one. Found by testing
#                 this check against the tree: check-outbox.sh backs /check-outbox and
#                 had been read as "nothing runs it", which is false.
#   DORMANT-FIXTURES-ONLY   referenced only from the tests/ tree
#   DORMANT       referenced by nothing
#
# TWO FALSE-POSITIVE GUARDS, both learned from real misreads:
#
#  1. A reference from a FIXTURE SUITE IS NOT COVERAGE. `tests/*fixtures*.sh` are
#     themselves guard-layer members, so it is tempting to call the check they invoke
#     "run on every push". It is not run against the REPOSITORY — it is run against a
#     synthetic fixture tree. That is exactly how check-pickup-cron-lock.sh looked
#     covered while the live defect went unreported. Fixture references are recorded
#     and classified DORMANT-FIXTURES-ONLY, never COVERED.
#
#  2. TASK FILES AND HANDOVERS ARE NEVER SEARCHED. A script name appears in the task
#     that created it, in every handover since, and in episodic summaries. None of
#     those execute anything. Searching them would mark essentially every script
#     covered and produce a permanently-green check — the worst failure mode a guard
#     has, because green is why nobody looks.
#
#  3. A DORMANT CALLER DOES NOT RESCUE ITS CALLEE. If script A invokes script B but
#     nothing invokes A, B is not covered. A caller counts only when it is itself a
#     declared guard-layer member or is itself named by cron/CI. Without this, two
#     dormant scripts calling each other would certify each other as covered.
#
# INVENTORY AGREEMENT (fail-closed). This check derives the unclassified set with its
# own glob, then cross-checks the COUNT against `run-guard-layer.sh --list --json`,
# which is the authority on what "unclassified" means. Disagreement exits 2 rather
# than reporting on a set the layer does not recognise. Two independent definitions
# of the same set is precisely the hazard T-2818 documents — the copy that drifts is
# the one that quietly stops catching things — so they are pinned to each other here
# instead of being allowed to diverge.
#
# THIS CHECK IS DEPLOY-TIME, NOT A GUARD-LAYER MEMBER, AND THAT IS DELIBERATE. It
# reads /etc/cron.d — host state. In CI no termlink crontab is installed, so every
# cron-covered script would read DORMANT and it would fire on every push. Its own
# disposition, by its own taxonomy, is `deploy-time`. It is the sibling of
# check-cron-install-drift.sh, which is unmarked for the identical reason.
#
# SCOPE — read a clean exit narrowly. It answers one question: does every unclassified
# script have something that runs it? It does NOT check that the runner is correctly
# configured, that the script passes, or that a COVERED script's cron line is sane.
#
# Exit codes: 0 every unclassified script has a runner · 1 one or more dormant
#             · 2 tooling error (fail-closed)
set -uo pipefail

SCRIPTS_DIR="${GUARD_COVERAGE_SCRIPTS_DIR:-scripts}"
TESTS_DIR="${GUARD_COVERAGE_TESTS_DIR:-tests}"
CRON_SRC_DIR="${GUARD_COVERAGE_CRON_SRC_DIR:-.context/cron}"
CRON_INSTALLED_DIR="${GUARD_COVERAGE_CRON_INSTALLED_DIR:-/etc/cron.d}"
CI_DIR="${GUARD_COVERAGE_CI_DIR:-.github/workflows}"
COMMANDS_DIR="${GUARD_COVERAGE_COMMANDS_DIR:-.claude/commands}"
CALLER_DIRS="${GUARD_COVERAGE_CALLER_DIRS:-scripts bin}"
RUNNER="${GUARD_COVERAGE_RUNNER:-scripts/run-guard-layer.sh}"
SKIP_AGREEMENT="${GUARD_COVERAGE_SKIP_AGREEMENT:-0}"

QUIET=0
FORMAT=human

usage() {
    cat <<'EOF'
check-guard-runner-coverage.sh — every guard script the layer calls "unclassified"
must still be run by SOMETHING: an installed crontab, a CI workflow, or a live caller.

"unclassified" means "carries no # guard-layer: marker". It does not mean "unrun",
and it does not mean "covered". This check resolves which.

Usage: check-guard-runner-coverage.sh [OPTIONS]
  --json          Emit {ok, firing[], shipped_dark[], operator_invoked[], covered[], summary}
  --quiet         Print only on firing (cron-friendly)
  --no-heartbeat  Accepted for parity; this check writes no heartbeat
  -h, --help      This help

Test seams (fixtures need no host state):
  GUARD_COVERAGE_SCRIPTS_DIR         default scripts
  GUARD_COVERAGE_TESTS_DIR           default tests
  GUARD_COVERAGE_CRON_SRC_DIR        default .context/cron
  GUARD_COVERAGE_CRON_INSTALLED_DIR  default /etc/cron.d
  GUARD_COVERAGE_CI_DIR              default .github/workflows
  GUARD_COVERAGE_COMMANDS_DIR        default .claude/commands
  GUARD_COVERAGE_CALLER_DIRS         default "scripts bin"
  GUARD_COVERAGE_RUNNER              default scripts/run-guard-layer.sh
  GUARD_COVERAGE_SKIP_AGREEMENT=1    skip the inventory cross-check

Fixtures: bash tests/guard-runner-coverage-fixtures.sh

Exit: 0 all covered · 1 dormant script(s) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-guard-runner-coverage: unknown arg: $1" >&2; exit 2 ;;
    esac
done

[ -d "$SCRIPTS_DIR" ] || { echo "check-guard-runner-coverage: scripts dir not found: $SCRIPTS_DIR" >&2; exit 2; }

has_marker() { grep -qm1 -E '^#[[:space:]]*guard-layer:[[:space:]]*source' "$1" 2>/dev/null; }

# ---- inventory: mirrors run-guard-layer.sh's own globs -----------------------
unclassified=(); marked=()
for f in "$SCRIPTS_DIR"/check-*.sh "$SCRIPTS_DIR"/test-*.sh "$TESTS_DIR"/*.sh; do
    [ -e "$f" ] || continue
    b="$(basename "$f")"
    # tests/*fixtures*.sh join the layer by naming convention — never unclassified.
    case "$f" in "$TESTS_DIR"/*) case "$b" in *fixtures*) marked+=("$b"); continue ;; esac ;; esac
    if has_marker "$f"; then marked+=("$b"); else unclassified+=("$b"); fi
done

if [ "${#unclassified[@]}" -eq 0 ]; then
    echo "check-guard-runner-coverage: empty unclassified inventory under $SCRIPTS_DIR / $TESTS_DIR — enumeration failed" >&2
    echo "  An empty inventory is a tooling error, never a vacuous clean (T-2747)." >&2
    exit 2
fi

# ---- inventory agreement with the authority (fail-closed) -------------------
if [ "$SKIP_AGREEMENT" != "1" ]; then
    if [ ! -f "$RUNNER" ]; then
        echo "check-guard-runner-coverage: runner not found: $RUNNER (cannot verify inventory agreement)" >&2
        exit 2
    fi
    rj="$(GUARD_LAYER_SCRIPTS_DIR="$SCRIPTS_DIR" GUARD_LAYER_TESTS_DIR="$TESTS_DIR" \
          bash "$RUNNER" --list --json 2>/dev/null)" || true
    rn="$(printf '%s' "$rj" | sed -n 's/.*"unclassified":\([0-9]*\).*/\1/p' | head -1)"
    if [ -z "$rn" ]; then
        echo "check-guard-runner-coverage: could not read unclassified count from $RUNNER --list --json" >&2
        exit 2
    fi
    if [ "$rn" != "${#unclassified[@]}" ]; then
        echo "check-guard-runner-coverage: INVENTORY DISAGREEMENT — this check sees ${#unclassified[@]} unclassified, $RUNNER reports $rn" >&2
        echo "  The two definitions of 'unclassified' have drifted. Reconcile before trusting either." >&2
        exit 2
    fi
fi

# ---- helpers ---------------------------------------------------------------
# Literal-name search across a set of dirs, excluding the script's own file.
refs_in() { # $1=name  $2..=dirs
    local name="$1"; shift
    local d out=""
    for d in "$@"; do
        [ -d "$d" ] || continue
        out="$out$(grep -rlF -- "$name" "$d" 2>/dev/null | grep -v -- "/$name\$" || true)
"
    done
    printf '%s' "$out" | grep -v '^$' || true
}

# A caller rescues its callee only if the caller is itself run.
caller_is_live() { # $1 = path to caller
    local c="$1" cb; cb="$(basename "$c")"
    has_marker "$c" && return 0
    [ -d "$CRON_INSTALLED_DIR" ] && grep -rqF -- "$cb" "$CRON_INSTALLED_DIR" 2>/dev/null && return 0
    [ -d "$CI_DIR" ] && grep -rqF -- "$cb" "$CI_DIR" 2>/dev/null && return 0
    return 1
}

covered=(); shipped_dark=(); operator=(); dormant=(); dormant_fx=()

for s in "${unclassified[@]}"; do
    why=""
    # (a) installed crontab
    if [ -d "$CRON_INSTALLED_DIR" ] && grep -rqF -- "$s" "$CRON_INSTALLED_DIR" 2>/dev/null; then
        why="installed crontab"
    fi
    # (b) CI workflow
    if [ -z "$why" ] && [ -d "$CI_DIR" ] && grep -rqF -- "$s" "$CI_DIR" 2>/dev/null; then
        why="CI workflow"
    fi
    # (c) a LIVE caller under scripts/ or bin/
    if [ -z "$why" ]; then
        while IFS= read -r c; do
            [ -n "$c" ] || continue
            if caller_is_live "$c"; then why="caller $(basename "$c")"; break; fi
        done <<< "$(refs_in "$s" $CALLER_DIRS)"
    fi
    if [ -n "$why" ]; then covered+=("$s|$why"); continue; fi
    # (d) backs a slash command -> operator-invoked (non-firing)
    if [ -d "$COMMANDS_DIR" ] && grep -rqF -- "$s" "$COMMANDS_DIR" 2>/dev/null; then
        operator+=("$s|backs a slash command under $COMMANDS_DIR — operator-invoked, not scheduled"); continue
    fi
    # (e) git-tracked but uninstalled crontab -> shipped-dark (non-firing)
    if [ -d "$CRON_SRC_DIR" ] && grep -rqF -- "$s" "$CRON_SRC_DIR" 2>/dev/null; then
        shipped_dark+=("$s|crontab committed under $CRON_SRC_DIR but not installed"); continue
    fi
    # (f) fixture-only reference -> still dormant
    if [ -n "$(refs_in "$s" "$TESTS_DIR")" ]; then
        dormant_fx+=("$s|referenced only by its fixture suite — exercised against synthetic input, never against this tree")
        continue
    fi
    dormant+=("$s|no runner anywhere")
done

n_fire=$(( ${#dormant[@]} + ${#dormant_fx[@]} ))
total=${#unclassified[@]}

json_arr() { # entries as name|why
    local first=1 e
    printf '['
    for e in "$@"; do
        [ -n "$e" ] || continue
        [ $first -eq 1 ] || printf ','
        first=0
        printf '{"script":%s,"why":%s}' \
            "$(printf '%s' "${e%%|*}" | jq -R .)" \
            "$(printf '%s' "${e#*|}" | jq -R .)"
    done
    printf ']'
}

if [ "$FORMAT" = json ]; then
    printf '{"ok":%s,"firing":' "$([ "$n_fire" -eq 0 ] && echo true || echo false)"
    json_arr ${dormant[@]+"${dormant[@]}"} ${dormant_fx[@]+"${dormant_fx[@]}"}
    printf ',"shipped_dark":'; json_arr ${shipped_dark[@]+"${shipped_dark[@]}"}
    printf ',"operator_invoked":'; json_arr ${operator[@]+"${operator[@]}"}
    printf ',"covered":'; json_arr ${covered[@]+"${covered[@]}"}
    printf ',"summary":{"unclassified":%s,"covered":%s,"shipped_dark":%s,"operator_invoked":%s,"dormant":%s,"dormant_fixtures_only":%s},' \
        "$total" "${#covered[@]}" "${#shipped_dark[@]}" "${#operator[@]}" "${#dormant[@]}" "${#dormant_fx[@]}"
    printf '"scope":"Detects whether an unclassified guard script has any runner. Does NOT verify the runner is correctly configured, nor that the script passes."}\n'
    [ "$n_fire" -eq 0 ] && exit 0 || exit 1
fi

if [ "$n_fire" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || {
        echo "check-guard-runner-coverage: clean — all $total unclassified script(s) have a runner (${#covered[@]} covered, ${#operator[@]} operator-invoked, ${#shipped_dark[@]} shipped-dark)."
        echo "  Scope: this proves each has a runner. It does not verify the runner is correctly configured, nor that the script passes."
    }
    exit 0
fi

echo "check-guard-runner-coverage: FIRING — $n_fire of $total unclassified script(s) are DORMANT (nothing runs them):"
for e in ${dormant[@]+"${dormant[@]}"};    do echo "  ↳ ${e%%|*}: ${e#*|}"; done
for e in ${dormant_fx[@]+"${dormant_fx[@]}"}; do echo "  ↳ ${e%%|*}: ${e#*|}"; done
if [ "${#operator[@]}" -gt 0 ]; then
    echo "  ${#operator[@]} operator-invoked (reported, NOT firing — a human runs these on demand):"
    for e in ${operator[@]+"${operator[@]}"}; do echo "    · ${e%%|*}"; done
fi
if [ "${#shipped_dark[@]}" -gt 0 ]; then
    echo "  ${#shipped_dark[@]} shipped-dark (reported, NOT firing — owned by check-cron-install-drift.sh):"
    for e in ${shipped_dark[@]+"${shipped_dark[@]}"}; do echo "    · ${e%%|*}"; done
fi
echo "  ${#covered[@]} covered by an installed crontab, CI, or a live caller."
cat <<'EOF'
  A guard nothing executes asserts nothing. Give each dormant script exactly one
  disposition: cron (runtime canary — name the crontab), deploy-time (reads host
  state; needs a preflight-tier runner), guard-layer (hermetic — add the
  '# guard-layer: source' marker), or retired (superseded/dead).
  Do NOT bulk-add the marker: a script that reaches a live hub or hangs would then
  run on every push and PR.
EOF
exit 1
