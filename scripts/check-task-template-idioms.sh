#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# check-task-template-idioms.sh (T-2777)
#
# Guards the TASK TEMPLATES against prescribing a verification idiom whose exit
# status is decided by a pipeline — the shape T-2775 measured as returning 141
# when the check succeeds.
#
# WHY THIS IS A SEPARATE GUARD FROM check-verification-pipefail.sh.
# That check scans `## Verification` blocks in real tasks and — correctly —
# skips `#` comment lines, because a comment is not an executable verification
# command. The templates carry their guidance entirely in comments and in the
# `<!-- -->` AC-authoring block. Pointing the existing check at
# `.tasks/templates/` reports `tasks_with_verification: 0`: a guard that can
# never fire. So the template needed a control that reads PROSE, not commands.
#
# The gap this closes is the T-2680 shape one layer up. The tree scanned clean
# while `.tasks/templates/default.md` prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# in the L-387 hint, and a bare `... | grep -q "Overall:.*PASS"` in the
# [REVIEWER] conversion example, as the forms to ADD to a Verification block.
# Every task created from the template inherited that, which is the most
# plausible mechanism behind the 158 lines T-2775 had to ledger. The guard built
# to catch the idiom was structurally unable to see the file teaching it.
#
# PRESCRIPTION VS CITATION. A template SHOULD be able to show the unsafe form in
# order to warn against it — this one does. The two are separated by an explicit
# marker: a line demonstrating a bad idiom must carry one of UNSAFE / DO NOT /
# WRONG / NEVER (case-insensitive) on that same line. Unmarked, it reads as a
# recommendation and fires. That keeps the convention self-documenting: if you
# add a counter-example, you label it.
#
# SCOPE. Detects pipeline-decided command SHAPES in template guidance text. It
# does not execute anything, and it does not verify that the idioms a template
# recommends are otherwise correct.
#
# EXIT: 0 = no template prescribes a risky shape, 1 = one does, 2 = tooling error.
# FLAGS: --json  --quiet  --templates-dir <p> (repeatable)  --no-heartbeat
# TEST SEAM: --templates-dir points at a fixture dir (PL-213).
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JSON=0; QUIET=0; NO_HEARTBEAT=0
TEMPLATE_DIRS=()

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1 ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) NO_HEARTBEAT=1 ;;
        --templates-dir)
            shift
            [ $# -gt 0 ] || { echo "check-task-template-idioms: --templates-dir needs a value" >&2; exit 2; }
            TEMPLATE_DIRS+=("$1") ;;
        -h|--help) sed -n '3,40p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) echo "check-task-template-idioms: unknown flag '$1'" >&2; exit 2 ;;
    esac
    shift
done

if [ "${#TEMPLATE_DIRS[@]}" -eq 0 ]; then
    TEMPLATE_DIRS=("$REPO_ROOT/.tasks/templates")
fi

for d in "${TEMPLATE_DIRS[@]}"; do
    [ -d "$d" ] || { echo "check-task-template-idioms: not a directory: $d" >&2; exit 2; }
done

OUT="$(python3 - "$JSON" "${TEMPLATE_DIRS[@]}" <<'PY'
import json, re, sys
from pathlib import Path

json_mode = sys.argv[1] == "1"
dirs = [Path(p) for p in sys.argv[2:]]

# A pipe into a consumer that exits early (or parses and can fail independently).
PIPE_TO_CONSUMER = re.compile(
    r"\|\s*(?:sudo\s+|command\s+)?(?:grep\s+(?:-\w*\s+)*-\w*q|head|python3?|jq)\b"
)
# An explicit label marking the line as a counter-example rather than advice.
NEGATIVE_MARKER = re.compile(r"\b(UNSAFE|DO NOT|DON'T|WRONG|NEVER|BAD)\b", re.IGNORECASE)


def strip_substitutions(s: str) -> str:
    """Remove `$( )` spans: a pipeline inside a command substitution has its
    status discarded, so the OUTER command decides and the shape is safe.
    Mirrors the same helper in check-verification-pipefail.sh.

    Backticks are deliberately NOT treated as substitutions here, which is where
    this diverges from its sibling. In a shell command a backtick span IS a
    substitution; in a task template it is markdown code formatting, and the
    prescription lives inside it. The real pre-T-2777 line was

        `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.

    — an unsafe pipeline the reader is told to copy into their Verification
    block, wrapped in markdown backticks. Treating those as status-discarding
    made the check silently clear the exact defect it exists to catch, so we
    unwrap the delimiters and read the content. Prescribing a real backtick
    substitution instead of `$( )` would false-fire; that trade is accepted,
    since the idiom this template teaches is `$( )` and backticks are deprecated
    shell style anyway."""
    s = s.replace("`", " ")
    out, i, n = [], 0, len(s)
    while i < n:
        if s[i] == "$" and i + 1 < n and s[i + 1] == "(":
            depth, i = 1, i + 2
            while i < n and depth:
                if s[i] == "(":
                    depth += 1
                elif s[i] == ")":
                    depth -= 1
                i += 1
            out.append(" SUBST ")
            continue
        out.append(s[i])
        i += 1
    return "".join(out)


firing, checked = [], 0
for d in sorted(dirs):
    for path in sorted(d.glob("*.md")):
        checked += 1
        for lineno, raw in enumerate(path.read_text(encoding="utf-8",
                                                    errors="replace").splitlines(), 1):
            line = raw.strip()
            if not line:
                continue
            if NEGATIVE_MARKER.search(line):
                continue  # labelled counter-example, not a recommendation
            if not PIPE_TO_CONSUMER.search(strip_substitutions(line)):
                continue
            firing.append({
                "file": str(path),
                "line": lineno,
                "text": line[:160],
            })

scope = ("detects pipeline-decided command shapes PRESCRIBED in task-template "
         "guidance; a counter-example labelled UNSAFE/DO NOT/WRONG/NEVER is "
         "treated as a warning, not advice")

if json_mode:
    print(json.dumps({
        "ok": not firing,
        "firing": firing,
        "checked": checked,
        "scope": scope,
    }, sort_keys=True))
else:
    if firing:
        print(f"check-task-template-idioms: {len(firing)} prescribed risky shape(s)")
        for f in firing:
            print(f"  {f['file']}:{f['line']}: {f['text']}")
        print()
        print("Each line above reads as a RECOMMENDED verification idiom, but its exit")
        print("status is decided by the pipeline: the consumer exits on match, the")
        print("producer takes SIGPIPE, and pipefail propagates 141 — the gate fails")
        print("because the check succeeded.")
        print()
        print("Fix: prescribe a herestring instead —")
        print('  out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"')
        print("Or, if the line is deliberately showing a BAD example, label it on the")
        print("same line with UNSAFE / DO NOT / WRONG / NEVER.")
    else:
        print(f"check-task-template-idioms: clean ({checked} template(s) scanned)")
    print(f"Scope: {scope}.")

sys.exit(1 if firing else 0)
PY
)"
RC=$?

if [ "$RC" -eq 2 ]; then
    echo "check-task-template-idioms: internal error" >&2
    exit 2
fi

if [ "$QUIET" -eq 1 ] && [ "$RC" -eq 0 ]; then
    :
else
    printf '%s\n' "$OUT"
fi

if [ "$NO_HEARTBEAT" -eq 0 ]; then
    HB="$REPO_ROOT/.context/working/.task-template-idioms-check.heartbeat"
    mkdir -p "$(dirname "$HB")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HB" 2>/dev/null || true
fi

exit "$RC"
