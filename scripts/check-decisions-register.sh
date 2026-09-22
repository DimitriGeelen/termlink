#!/usr/bin/env bash
# guard-layer: source
# T-2969 — local detector for .context/project/decisions.yaml corruption.
#
# WHY THIS EXISTS, AND WHY THE RECURRENCE COUNT IS THE ARGUMENT
# -------------------------------------------------------------
# The completion-time decision auto-capture has corrupted this register FOUR
# times: T-2892, T-2850, T-2968, and again on 2026-09-22. This task was filed
# after the third and sat in `captured` while the fourth happened.
#
# Every repair so far was REACTIVE, triggered by the pre-push gate
# (T-1599/T-1610) refusing a push. That gate is good and it is the only thing
# that has ever caught this — but it fires at the worst possible moment, when
# someone is trying to ship, and it catches only one of the two failure axes.
# This check runs in CI on every push and PR via the T-2686 guard-layer job, so
# the fifth occurrence surfaces on the commit that causes it.
#
# TWO AXES, AND THE SECOND ONE IS INVISIBLE TODAY
# -----------------------------------------------
#   (a) PARSE FAILURE. What the pre-push gate catches. The 2026-09-22 instance:
#       the generator appended entries indented one level too deep
#           - id: PD-165        <- existing, column 1
#             ...
#             - id: PD-001      <- appended, column 3
#       which breaks the block sequence.
#
#   (b) DUPLICATE IDS. The SAME 2026-09-22 instance also restarted the counter
#       at PD-001 while PD-001 and PD-002 already existed. That axis PARSES
#       PERFECTLY. Had the indentation been right, the register would have
#       silently carried two different decisions under one id and nothing in
#       this repo would have noticed — no gate, no canary, no audit section.
#       A duplicate id is worse than a parse error precisely because it is
#       quiet: the register keeps working and starts lying.
#
# THE GENERATOR IS NOT PATCHED HERE. It is vendored (G-062) and already on the
# upstream record, so a local fix would be erased by the next `fw upgrade` —
# roughly one every two months in this lineage. This is the local DETECTION
# that survives a re-vendor.
#
# SCOPE — read a green narrowly (T-2680). It answers two questions: does the
# register parse, and are its ids unique. It does NOT judge whether a decision
# is correct, well-attributed, or belongs to the task it names.
#
# Exit: 0 clean · 1 corrupt (parse failure or duplicate ids) · 2 tooling.
# FAIL-CLOSED: a missing register, absent python3, or missing PyYAML exit 2,
# never a clean 0 — a checker reporting green because it could not look is the
# disease it exists to cure.
set -u

REGISTER="${DECISIONS_REGISTER:-.context/project/decisions.yaml}"
JSON=0
QUIET=0

usage() {
    cat <<'USAGE'
Usage: check-decisions-register.sh [options]

Detects corruption in .context/project/decisions.yaml on two axes: parse
failure, and duplicate decision ids (which parse cleanly and are therefore
invisible to the pre-push gate).

Options:
  --register FILE   register path (default .context/project/decisions.yaml)
  --json            machine-readable envelope
  --quiet           print only when something fires
  --help            this text

Exit: 0 clean · 1 corrupt · 2 tooling (fail-closed)
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --register) REGISTER="${2:-}"; shift 2 ;;
        --json)     JSON=1; shift ;;
        --quiet)    QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        --help|-h)  usage; exit 0 ;;
        *) echo "check-decisions-register: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null 2>&1 || {
    echo "check-decisions-register: python3 not found" >&2; exit 2; }
[ -f "$REGISTER" ] || {
    echo "check-decisions-register: register not found: $REGISTER" >&2; exit 2; }

REGISTER="$REGISTER" JSON="$JSON" QUIET="$QUIET" python3 - <<'PY'
import os, sys, json, collections

try:
    import yaml
except ImportError:
    print("check-decisions-register: PyYAML not installed", file=sys.stderr)
    sys.exit(2)

path  = os.environ["REGISTER"]
as_json = os.environ["JSON"] == "1"
quiet   = os.environ["QUIET"] == "1"

# --- axis (a): does it parse at all? ---------------------------------------
try:
    data = yaml.safe_load(open(path, encoding="utf-8"))
except yaml.YAMLError as exc:
    mark = getattr(exc, "problem_mark", None)
    where = f" at line {mark.line + 1}, column {mark.column + 1}" if mark else ""
    msg = f"PARSE FAILURE{where}: {getattr(exc, 'problem', exc)}"
    if as_json:
        print(json.dumps({"ok": False, "axis": "parse", "detail": msg, "register": path}))
    else:
        print(f"  PARSE   CORRUPT  {msg}")
        print()
        print("Remediation: repair the malformed entry IN PLACE — never a wholesale")
        print("rewrite, and never `git push --no-verify`. The pre-push gate is the")
        print("only detector that has ever caught this class; bypassing it removes it.")
    sys.exit(1)
except OSError as exc:
    print(f"check-decisions-register: cannot read register: {exc}", file=sys.stderr)
    sys.exit(2)

# Tolerate both shapes: a bare list, or a mapping with a `decisions:` key.
if isinstance(data, dict):
    entries = data.get("decisions")
elif isinstance(data, list):
    entries = data
else:
    entries = None

if entries is None:
    # A register that parses to something with no decisions at all is a tooling
    # error, never a clean bill — a reader that silently stopped matching would
    # otherwise report green forever (the T-2747 zero-items lesson).
    print("check-decisions-register: no decisions found in register (unexpected shape)",
          file=sys.stderr)
    sys.exit(2)

# --- axis (b): duplicate ids — parses cleanly, invisible to the gate --------
ids = [str(e.get("id")) for e in entries if isinstance(e, dict) and e.get("id")]
if not ids:
    print("check-decisions-register: zero decision ids parsed", file=sys.stderr)
    sys.exit(2)

dupes = sorted(i for i, n in collections.Counter(ids).items() if n > 1)

if as_json:
    print(json.dumps({
        "ok": not dupes, "axis": "duplicate-id" if dupes else None,
        "entries": len(entries), "ids": len(ids),
        "duplicates": dupes, "register": path,
        "scope": "checks that the register parses and that ids are unique; does NOT judge whether a decision is correct or well-attributed",
    }))
elif dupes:
    print(f"  IDS     CORRUPT  {len(dupes)} duplicate id(s): {', '.join(dupes)}")
    print()
    print("These PARSE CLEANLY, so the pre-push gate cannot see them. The register")
    print("keeps working and starts lying: two different decisions under one id.")
    print("Remediation: renumber the later entries above the current maximum.")
elif not quiet:
    print(f"  decisions register clean — {len(entries)} entries, {len(ids)} unique ids")
    print("  SCOPE: checks that it parses and that ids are unique. It does NOT judge")
    print("         whether a decision is correct, or belongs to the task it names.")

sys.exit(1 if dupes else 0)
PY
