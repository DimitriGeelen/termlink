#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# check-absence-assertion.sh — a `## Verification` leg that asserts something is ABSENT
# must first establish that the search could have succeeded (T-3148, from the T-3144 census).
#
#   bash scripts/check-absence-assertion.sh [--json] [--quiet]
#                                           [--tasks-dir DIR] [--allowlist PATH]
#                                           [--no-heartbeat]
#
# THE DEFECT
#
# `! grep -q "PATTERN" file` exits 0 when PATTERN is absent. It ALSO exits 0 when the file
# was renamed, deleted, or emptied. The leg cannot distinguish "the bad thing is not there"
# from "I could not look", so P-011 reports green over a check that never ran. The same
# defect wears a second hat over a COMMAND's output:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]
#
# If cargo is absent, or dies before emitting diagnostics, there are no `^error` lines, the
# count is 0, and the leg passes — a build gate that goes green precisely when the build
# could not run. Found in this corpus, not invented.
#
# SCOPE — `.tasks/active/` ONLY, and this is a design decision, not an oversight.
#
# The T-3144 census measured 30 firing legs across the whole corpus. 28 of them sit in
# `.tasks/completed/`, whose verification blocks will never execute again: un-fixable by
# construction, because "fixing" them means editing closed history for no behavioural gain.
#
# The obvious move is to scan everything and allowlist those 28. That is worse. A 28-entry
# silence list is its own slow failure — entries get added, never removed, and the ledger
# stops being read, which is the same alarm-fatigue disease one level up (T-2818, T-2833).
# Putting finished work out of SCOPE says the true thing instead: history is not this
# check's business. The result is a check that reports 2 findings today, both real and both
# fixable, and still catches every new leg at the moment it is written — which is the point.
#
# An allowlist exists anyway, for a future finding that is genuinely real and unfixable. It
# ships EMPTY and empty is the healthy state.
#
# WHAT A GREEN DOES NOT MEAN (T-2680)
#
# It means: no absence assertion in an ACTIVE task lacks a companion. It does NOT mean the
# verification is adequate, that the commands test the ACs, or that a task has any
# verification at all — a task with an empty `## Verification` passes this and gates on
# nothing (that is T-2831's and P-011's territory). Both output paths say so.
#
# Exit: 0 clean · 1 firing · 2 tooling. FAIL-CLOSED — a missing tasks dir, absent python3,
# an unreadable allowlist, or a corpus of zero task files exits 2, never a vacuous clean.
# A checker that reports clean because it could not look is the defect it detects.

set -uo pipefail

TASKS_DIR=".tasks/active"
JSON=0
QUIET=0
HEARTBEAT=1
ALLOWLIST=""
ALLOWLIST_DEFAULT=".context/checks/absence-assertion-allowlist"

usage() {
    sed -n '2,/^set -uo pipefail$/p' "$0" | sed 's/^# \{0,1\}//' | head -n -2
    exit 0
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1 ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --tasks-dir)
            shift
            [ $# -ge 1 ] || { echo "check-absence-assertion: --tasks-dir requires a value" >&2; exit 2; }
            TASKS_DIR="$1"
            ;;
        --allowlist)
            shift
            [ $# -ge 1 ] || { echo "check-absence-assertion: --allowlist requires a value" >&2; exit 2; }
            ALLOWLIST="$1"
            ;;
        -h|--help) usage ;;
        *) echo "check-absence-assertion: unknown flag: $1" >&2; exit 2 ;;
    esac
    shift
done

[ -n "$ALLOWLIST" ] || ALLOWLIST="${ABSENCE_ASSERTION_ALLOWLIST:-$ALLOWLIST_DEFAULT}"

command -v python3 >/dev/null 2>&1 || {
    echo "check-absence-assertion: python3 not available — cannot scan (fail-closed)" >&2
    exit 2
}
[ -d "$TASKS_DIR" ] || {
    echo "check-absence-assertion: tasks dir not found: $TASKS_DIR (fail-closed)" >&2
    exit 2
}

OUT=$(TASKS_DIR="$TASKS_DIR" ALLOWLIST="$ALLOWLIST" JSON="$JSON" python3 - <<'PY'
import os, sys, re, json

tasks_dir = os.environ["TASKS_DIR"]
allow_path = os.environ["ALLOWLIST"]
want_json = os.environ["JSON"] == "1"

# --- allowlist: "<relpath>::<leg-substring>  # reason" ------------------------
acks = []
if os.path.isfile(allow_path):
    try:
        with open(allow_path, encoding="utf-8") as fh:
            for line in fh:
                entry = line.split("#", 1)[0].strip()
                if entry:
                    acks.append(entry)
    except OSError as e:
        print(json.dumps({"_error": f"allowlist unreadable: {e}"}))
        sys.exit(0)

# A negated search, or a count-equals-zero assertion.
NEG = re.compile(r'(^|[;&|]\s*|\bif\s+|\bthen\s+)!\s*(grep|rg|egrep|fgrep)\b')
CNT = re.compile(r'\b(grep\s+-c|wc\s+-l)\b')
ZERO = re.compile(r'-eq\s+0\b|-qx\s+0\b|==\s*"?0"?|=\s*"?0"?\s*\]')
PATH_RE = re.compile(r'[\w./-]+\.(?:sh|py|md|yaml|yml|json|rs|toml|txt|out|log)\b')

files = 0
candidates = 0
cleared = 0
firing = []
acked = []

for root, _dirs, names in os.walk(tasks_dir):
    for name in sorted(names):
        if not name.endswith(".md"):
            continue
        path = os.path.join(root, name)
        files += 1
        try:
            text = open(path, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        if "## Verification" not in text:
            continue
        block = text.split("## Verification", 1)[1]
        nxt = re.search(r"\n## ", block)
        if nxt:
            block = block[: nxt.start()]
        body = [l for l in block.split("\n") if l.strip() and not l.lstrip().startswith("#")]
        blocktext = "\n".join(body)

        for leg in body:
            is_neg = bool(NEG.search(leg))
            is_cnt = bool(CNT.search(leg) and ZERO.search(leg))
            if not (is_neg or is_cnt):
                continue
            candidates += 1

            paths = [p for p in PATH_RE.findall(leg) if "/" in p or p.startswith(".")]
            guarded = False

            # (a) an existence test on the same line
            if re.search(r'\btest\s+-[fsedr]\b|\[\s+-[fsedr]\s', leg):
                guarded = True
            # (b) an &&-joined producer feeding the file this leg reads
            if re.search(r'>\s*\S+.*&&', leg) or re.search(r'&&\s*!\s*(grep|rg)', leg):
                guarded = True
            for p in paths:
                esc = re.escape(p)
                # (c) an existence test anywhere in the same block
                if re.search(r'(test\s+-[fsedr]|\[\s+-[fsedr])\s+"?' + esc, blocktext):
                    guarded = True
                # (d) a POSITIVE grep on the same file in the same block. The negative
                #     lookbehind keeps this leg from clearing itself.
                if re.search(r'(?<![!]\s)\bgrep\s+-[a-zA-Z]*q[a-zA-Z]*\s+[^|\n]*' + esc, blocktext):
                    guarded = True

            if guarded:
                cleared += 1
                continue

            rel = os.path.relpath(path)
            if rel.startswith(".."):
                rel = path
            stem = os.path.splitext(os.path.basename(path))[0]
            sig = f"{stem}::{leg.strip()[:60]}"
            if any(a in sig or sig.startswith(a) for a in acks):
                acked.append({"file": rel, "leg": leg.strip()[:200]})
            else:
                firing.append({"file": rel, "leg": leg.strip()[:200]})

print(json.dumps({
    "files": files,
    "candidates": candidates,
    "cleared": cleared,
    "firing": firing,
    "acknowledged": acked,
}))
PY
) || {
    echo "check-absence-assertion: scan failed (fail-closed)" >&2
    exit 2
}

case "$OUT" in
    *'"_error"'*)
        echo "check-absence-assertion: $OUT (fail-closed)" >&2
        exit 2
        ;;
esac

FILES=$(printf '%s' "$OUT" | python3 -c 'import sys,json; print(json.load(sys.stdin)["files"])' 2>/dev/null) || {
    echo "check-absence-assertion: unparseable scan result (fail-closed)" >&2; exit 2; }

# A corpus with zero task files is a tooling error, never a clean census (T-2747 lesson:
# "0 agrees with 0" is vacuously true and would report green over a scan that stopped working).
if [ "${FILES:-0}" -eq 0 ]; then
    echo "check-absence-assertion: no task files under $TASKS_DIR — refusing to report clean (fail-closed)" >&2
    exit 2
fi

if [ "$HEARTBEAT" = "1" ] && [ -d ".context/working" ]; then
    date -u +%Y-%m-%dT%H:%M:%SZ > .context/working/.absence-assertion.heartbeat 2>/dev/null || true
fi

SCOPE="detects a '## Verification' leg asserting an ABSENCE with no companion proving the search could succeed, in ACTIVE tasks only (completed tasks are out of scope by construction, not allowlisted); does NOT audit whether a task's verification is adequate or present at all"

printf '%s' "$OUT" > "${TMPDIR:-/tmp}/.absence-assertion.$$.json"
RESULT_JSON="${TMPDIR:-/tmp}/.absence-assertion.$$.json"
trap 'rm -f "$RESULT_JSON"' EXIT

if [ "$JSON" = "1" ]; then
    RESULT_JSON="$RESULT_JSON" SCOPE="$SCOPE" python3 - <<'PY'
import sys, json, os
d = json.load(open(os.environ["RESULT_JSON"]))
d["ok"] = len(d["firing"]) == 0
d["firing_count"] = len(d["firing"])
d["acknowledged_count"] = len(d["acknowledged"])
d["scope"] = os.environ["SCOPE"]
print(json.dumps(d))
sys.exit(1 if d["firing"] else 0)
PY
    rc=$?
    [ "$rc" -le 1 ] || { echo "check-absence-assertion: renderer crashed (fail-closed)" >&2; exit 2; }
    exit "$rc"
fi

RESULT_JSON="$RESULT_JSON" SCOPE="$SCOPE" QUIET="$QUIET" ALLOW="$ALLOWLIST" python3 - <<'PY'
import sys, json, os
d = json.load(open(os.environ["RESULT_JSON"]))
quiet = os.environ["QUIET"] == "1"
fire, ack = d["firing"], d["acknowledged"]
files, cand, clr = d["files"], d["candidates"], d["cleared"]

if not fire:
    if not quiet:
        print("check-absence-assertion: clean — %d active task file(s), %d absence assertion(s), "
              "%d already carry a companion, %d acknowledged" % (files, cand, clr, len(ack)))
        print("  Scope: " + os.environ["SCOPE"])
        for a in ack:
            print("    ACK  %s: %s" % (a["file"], a["leg"]))
    sys.exit(0)

print("check-absence-assertion: %d absence assertion(s) with nothing proving the search could succeed:" % len(fire))
for f in fire:
    print("  VACUOUS-RISK  " + f["file"])
    print("                " + f["leg"])
if not quiet:
    print("")
    print("  Scope: " + os.environ["SCOPE"])
    print("  Scanned %d active task file(s); %d of %d absence assertion(s) already carry a companion."
          % (files, clr, cand))
    print("")
    print("Remediation — pair the assertion with something that fails if the search could not happen:")
    print('    test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first')
    print('    grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion')
    print('    cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer')
    print("")
    print("  If a finding is genuinely real AND unfixable, acknowledge it in " + os.environ["ALLOW"])
    print("  with a cited reason. It ships empty on purpose — anything fixable gets fixed, not listed.")
sys.exit(1)
PY
rc=$?
[ "$rc" -le 1 ] || { echo "check-absence-assertion: renderer crashed (fail-closed)" >&2; exit 2; }
exit "$rc"
