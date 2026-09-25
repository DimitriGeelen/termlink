#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# T-3142 — an instruction names a verb that does not exist.
#
# WHY THIS EXISTS
# ---------------
# Two surfaces, verified from the main checkout on 2026-09-25, instruct an agent to
# run a verb this build does not have:
#   * `checkpoint.sh budget` — prescribed by the /resume skill AS THE SAFE READ
#     (G-087). Usage at checkpoint.sh:458 is {post-tool|reset|status}.
#   * `fw sidecar send`      — prescribed by a peer project's live e2e protocol.
#     Not in the verb table; returns "Unknown command: sidecar".
# In both, the DOCUMENTED path is absent while an UNDOCUMENTED one works. So the
# failure is invisible to anyone following the documentation — and in the first the
# absent path is the SAFE one, which makes following the docs strictly worse than
# ignoring them.
#
# THE DESIGN CONSTRAINT, AND IT COMES FROM THIS CHECK'S OWN PREHISTORY
# --------------------------------------------------------------------
# A third verb, `fw integrate run`, was reported upstream as never built — and
# RETRACTED (framework:pickup offset 160). It exists; bin/fw references it 28 times
# and all three subcommands respond. Every measurement behind the false claim had
# been taken from inside a git WORKTREE, whose vendored .agentic-framework was not
# the same tree as main's. The readings were true of where the agent stood and false
# of the framework — the T-2817 dangling-reference class biting the REPORT rather
# than the code.
#
# Therefore: THIS CHECK DECLARES THE ROOT IT RESOLVED AGAINST ON EVERY OUTPUT PATH.
# A verb-resolution check that does not say where it looked is one worktree away
# from confidently reporting a checkout defect as a framework defect, which is the
# exact mistake it exists to prevent.
#
# SURFACE, and why it is narrow. It scans .claude/commands — the SKILL files, which
# are literally what an agent is told to do. Shell-script comments were scanned in the
# first three drafts and are DOCUMENTATION, not instruction: every finding they
# produced was prose ("bin/fw missing", "vendored bin/fw and"). Three successive
# anchor tightenings did not fix that, because the problem was never the anchor — it
# was scanning a surface where `fw` appears as a discussed PATH rather than a cited
# COMMAND. Widening back to scripts/ needs a way to tell those apart, which is a
# different task.
#
# SCOPE — read a green narrowly (T-2680). It resolves `fw <verb>` references found in
# instructional surfaces against the live verb table. It does NOT check subcommands
# (`fw integrate run` resolves if `integrate` does), flags, whether the verb does what
# the instruction claims, or non-fw tools. A green means "every fw verb named here is
# a real verb" — nothing more.
#
# ############################################################################
# STATUS: PARKED, NOT IN SERVICE. DO NOT TRUST A GREEN FROM THIS SCRIPT.
# The guard-layer marker is deliberately NOT set (see below) so the runner does
# not adopt it. Two defects, found by its own load-bearing test (T-3142):
#
#   1. VACUOUS PASS. Given a file naming `fw sidecar` — a verb verified absent —
#      it reported "clean, 0 references" and exited 0. There is a floor on the
#      VERB TABLE size but none on the REFERENCE count, so "found nothing"
#      and "everything resolves" share an exit code. That is the exact T-2831
#      shape this repo has a dozen checks against, reproduced in a check whose
#      whole subject is things that look right and are not.
#   2. The VERB_CHECK_DIRS override does not take effect; the fixture directory
#      was not scanned at all, which is what produced (1).
#
# What DOES work and is worth keeping when this resumes: the ROOT line. Every
# output path names the fw binary and cwd it resolved against, because the
# upstream retraction at framework:pickup offset 160 was caused by measurements
# taken inside a worktree being reported as framework facts.
# ############################################################################
#
# Exit: 0 clean · 1 an instructed verb does not resolve · 2 tooling (FAIL-CLOSED).
set -u

FW_BIN="${VERB_CHECK_FW:-.agentic-framework/bin/fw}"
SCAN_DIRS="${VERB_CHECK_DIRS:-.claude/commands}"
# Was the scan surface overridden? A fixture tree legitimately holds one reference; the
# DEFAULT surface does not, so the reference floor below applies only to the real one.
DIRS_OVERRIDDEN=0; [ -n "${VERB_CHECK_DIRS:-}" ] && DIRS_OVERRIDDEN=1
MIN_REFS="${VERB_CHECK_MIN_REFS:-}"
ALLOWLIST="${VERB_CHECK_ALLOWLIST:-.context/checks/instructed-verb-allowlist}"
JSON=0; QUIET=0

usage() {
    cat <<'USAGE'
Usage: check-instructed-verb-resolves.sh [options]

Fires when an instructional surface tells an agent to run `fw <verb>` and that verb
is not in the shipped verb table.

Options:
  --fw PATH         the fw binary whose verb table is authoritative
  --dirs "A B"      space-separated dirs to scan
  --allowlist PATH  acknowledged references
  --json / --quiet / --no-heartbeat / --help

Exit: 0 clean · 1 unresolvable verb · 2 tooling (fail-closed)
USAGE
}
while [ $# -gt 0 ]; do
    case "$1" in
        --fw) FW_BIN="${2:-}"; shift 2 ;;
        --dirs) SCAN_DIRS="${2:-}"; DIRS_OVERRIDDEN=1; shift 2 ;;
        --min-refs) MIN_REFS="${2:-}"; shift 2 ;;
        --allowlist) ALLOWLIST="${2:-}"; shift 2 ;;
        --json) JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "check-instructed-verb-resolves: unknown argument: $1" >&2; exit 2 ;;
    esac
done

[ -x "$FW_BIN" ] || { echo "check-instructed-verb-resolves: fw not executable at $FW_BIN (fail-closed)" >&2; exit 2; }

# THE ROOT. Resolved and reported, never assumed.
FW_ABS="$(cd "$(dirname "$FW_BIN")" 2>/dev/null && pwd)/$(basename "$FW_BIN")" || {
    echo "check-instructed-verb-resolves: cannot resolve $FW_BIN (fail-closed)" >&2; exit 2; }

VERBS="$("$FW_BIN" help 2>&1 | sed 's/\x1b\[[0-9;]*m//g' | grep -oE "^  [a-z][a-z0-9_-]+" | tr -d ' ' | sort -u)"
VERB_COUNT="$(printf '%s\n' "$VERBS" | grep -c . || true)"
# An empty table would clear every reference vacuously (T-2831).
[ "${VERB_COUNT:-0}" -ge 10 ] || {
    echo "check-instructed-verb-resolves: verb table came back with ${VERB_COUNT:-0} entries — refusing to clear references against an empty table (fail-closed)" >&2
    exit 2
}

allowed=""
[ -f "$ALLOWLIST" ] && allowed="$(grep -vE '^\s*(#|$)' "$ALLOWLIST" 2>/dev/null | awk '{print $1}' || true)"

firing=""; checked=0
for d in $SCAN_DIRS; do
    [ -d "$d" ] || continue
    while IFS= read -r f; do
        [ -f "$f" ] || continue
        # ANCHOR, deliberately narrow (precision over recall, the sibling-check rule).
        # Only COMMAND forms count: backticked `fw x`, bin/fw x, line-leading fw x,
        # or a $-prompted fw x. A line-leading form was tried too and still caught
        # wrapped COMMENT text in shell files, so it was dropped. A bare bin/fw
        # form was dropped too: it matched the PATH being discussed in prose
        # ("bin/fw missing", "vendored bin/fw and"). A backtick is now required,
        # which is what actually distinguishes a cited COMMAND from a mentioned
        # FILE. A loose \bfw <word> anchor was tried first and
        # produced 13 findings of which every one was PROSE — "fw binary", "fw not",
        # "fw verb". A check that permanently false-positives is a check nobody reads.
        while IFS= read -r v; do
            [ -n "$v" ] || continue
            checked=$((checked+1))
            printf '%s\n' "$VERBS" | grep -qx "$v" && continue
            printf '%s\n' "$allowed" | grep -qx "$v" && continue
            case "$firing" in *"|$v:$f|"*) continue ;; esac
            firing="$firing|$v:$f|"
        done <<EOF
$(sed 's/\x1b\[[0-9;]*m//g' "$f" 2>/dev/null | grep -oE '(`fw |bin/fw |^ *fw |\$ fw )[a-z][a-z0-9_-]+' | awk '{print $NF}' | sort -u)
EOF
    done <<EOF
$(find "$d" -type f \( -name '*.md' -o -name '*.sh' \) 2>/dev/null)
EOF
done

# ############################################################################
# THE REFERENCE FLOOR (T-3142, the defect this check was parked for).
#
# "0 references, all resolve" is not a clean bill — it is the check reporting that it
# could not look, in the same words it uses for success. That is the exact defect this
# script exists to detect, committed inside the detector (T-2831).
#
# It is not hypothetical here. The anchor's backtick branch was written `\\`` and GNU
# ERE reads \` as the START-OF-BUFFER anchor, not a literal backtick — so every
# backticked `fw x` reference in the corpus was invisible. The check reported "clean, 4
# references" for as long as that stood; the true figure is 10. A floor turns that from
# a quiet green into a loud refusal.
#
# The floor applies to the DEFAULT surface only. A fixture tree legitimately holds one
# reference, so an explicit --dirs / VERB_CHECK_DIRS drops it to 0 unless --min-refs
# says otherwise.
# ############################################################################
if [ -z "$MIN_REFS" ]; then
    if [ "$DIRS_OVERRIDDEN" = "1" ]; then MIN_REFS=0; else MIN_REFS=5; fi
fi
if [ "$checked" -lt "$MIN_REFS" ]; then
    echo "check-instructed-verb-resolves: only $checked reference(s) found in [$SCAN_DIRS], floor is $MIN_REFS — refusing to report clean (fail-closed)" >&2
    echo "  The anchor has most likely stopped matching. 'No references' and 'no broken references'" >&2
    echo "  are the same output otherwise, which is the defect this check detects." >&2
    echo "  ROOT: resolved against: $FW_ABS ($VERB_COUNT verbs) — cwd $(pwd)" >&2
    exit 2
fi

ROOTLINE="resolved against: $FW_ABS ($VERB_COUNT verbs) — cwd $(pwd)"
SCOPE="resolves 'fw <verb>' references in instructional surfaces against the SHIPPED verb table. It does NOT check subcommands, flags, whether the verb does what the instruction claims, or non-fw tools."

n=0; [ -n "$firing" ] && n="$(printf '%s' "$firing" | tr '|' '\n' | grep -c ':' || true)"

if [ "$JSON" = "1" ]; then
    printf '{"ok":%s,"firing_count":%d,"checked":%d,"verb_table_size":%d,"root":"%s","scope":"%s"}\n' \
        "$([ "$n" -eq 0 ] && echo true || echo false)" "$n" "$checked" "$VERB_COUNT" "$FW_ABS" "$SCOPE"
elif [ "$n" -gt 0 ]; then
    echo "check-instructed-verb-resolves: $n instructed verb(s) do not resolve"
    printf '%s' "$firing" | tr '|' '\n' | grep ':' | sed 's/^/  fw /' | sed 's/:/  <- named in /'
    echo ""
    echo "  ROOT: $ROOTLINE"
    echo "  SCOPE: $SCOPE"
    echo ""
    echo "Remediation: either the verb ships, or the instruction stops naming it. Note the"
    echo "ROOT line above before reporting this upstream — a reading taken inside a git"
    echo "worktree is true of that checkout and NOT of the framework (offset-160 retraction)."
elif [ "$QUIET" != "1" ]; then
    echo "check-instructed-verb-resolves: clean — $checked reference(s) all resolve"
    echo "  ROOT: $ROOTLINE"
    echo "  SCOPE: $SCOPE"
fi
exit $([ "$n" -eq 0 ] && echo 0 || echo 1)
