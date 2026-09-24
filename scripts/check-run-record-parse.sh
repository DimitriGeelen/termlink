#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# T-3092 — a run record can be left UNPARSEABLE and nothing detects it.
#
# WHY THIS EXISTS
# ---------------
# `.context/runs/<task>-<slug>.yaml` is the load-bearing state carrier for a
# multi-round orchestrated sequence. It is what lets round 4 know what round 1
# decided, and what lets a session resume the sequence after a context reset —
# the T-3044 precedent proved that property only by re-reading the record intact
# after a compaction.
#
# On 2026-09-24 round 3 of T-3089 left that file unparseable: an unquoted list
# scalar wrapped onto a continuation line containing ": " —
#     - T-2978 (...) - deferred, and now additionally confirmed out of THIS
#       project's scope: penelope is /opt/050-email-archive's ...
# which YAML reads as a mapping key (`yaml.scanner.ScannerError`). NOTHING
# detected it. The orchestrator found it only because its own update happened to
# fail closed ON THE READ rather than writing over the damage. That is luck in
# the code, not a guard.
#
# It is the THIRD occurrence of this class here. `arc-011.yaml` was corrupted the
# same way while recording operator decisions (T-3084), which is why
# parse-before-write became a convention — a convention is discipline, and
# discipline is what this layer exists to replace.
#
# WHAT IT ASSERTS — the REAL consumer's property, not a weaker one that is easier
# to pass (the T-2805 rule). Every consumer of a run record does
# `yaml.safe_load(...)` and then subscripts it (`d["steps"]`). So the assertion is
# "loads AND is a mapping", not "is syntactically valid YAML": a file that parses
# to a string or a list is exactly as unusable to the reader as one that raises.
#
# THERE IS DELIBERATELY NO ALLOWLIST. Unlike the alloc-sink / drain-sink / busy-spin
# siblings, every finding here is a real defect with a real fix — an unreadable
# state carrier is never the intended state. An allowlist could only ever silence
# a genuine break (the `check-release-artifact-drift.sh` reasoning).
#
# SCOPE — read a green narrowly (T-2680). This answers ONE question: does every
# run record load as a mapping? It does NOT check that the record's CONTENT is
# true — that its step states match what the workers actually did, that `feeds`
# ordering is coherent, or that a step marked `complete` produced its handback.
# A run record that parses cleanly and lies about every step passes this check.
#
# Exit: 0 all readable · 1 one or more are not · 2 tooling (FAIL-CLOSED).
set -u

RUNS_DIR="${RUN_RECORD_DIR:-.context/runs}"
JSON=0; QUIET=0

usage() {
    cat <<'USAGE'
Usage: check-run-record-parse.sh [options]

Fires when any .context/runs/*.yaml does not load as a YAML mapping — the
property every consumer of a run record actually requires.

Options:
  --dir DIR         run-record directory (default .context/runs)
  --json            machine-readable
  --quiet           print only when something fires
  --no-heartbeat    accepted and ignored (guard-layer runner compatibility)
  --help            this text

Exit: 0 clean · 1 an unreadable record · 2 tooling (fail-closed)
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --dir)          RUNS_DIR="${2:-}"; shift 2 ;;
        --json)         JSON=1; shift ;;
        --quiet)        QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        --help|-h)      usage; exit 0 ;;
        *) echo "check-run-record-parse: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null 2>&1 || {
    echo "check-run-record-parse: python3 not found — cannot verify (fail-closed)" >&2
    exit 2
}

RUN_RECORD_DIR="$RUNS_DIR" RR_JSON="$JSON" RR_QUIET="$QUIET" python3 - <<'PY'
import glob, json, os, sys

runs_dir = os.environ["RUN_RECORD_DIR"]
as_json  = os.environ["RR_JSON"] == "1"
quiet    = os.environ["RR_QUIET"] == "1"

SCOPE = ("asserts every .context/runs/*.yaml loads as a YAML MAPPING — the property its "
         "readers actually require. It does NOT check that the record's content is true: "
         "a record that parses cleanly and misstates every step's outcome passes.")

try:
    import yaml
except Exception as e:                                    # fail-closed
    sys.stderr.write("check-run-record-parse: PyYAML not installed — cannot verify "
                     "(fail-closed): %s\n" % e)
    sys.exit(2)

# An absent directory is healthy: nothing was ever recorded is not a fault.
if not os.path.isdir(runs_dir):
    if as_json:
        print(json.dumps({"ok": True, "firing": [], "firing_count": 0, "checked": 0,
                          "note": "no run-record directory — nothing was ever recorded",
                          "scope": SCOPE}, indent=2))
    elif not quiet:
        print("check-run-record-parse: no run-record directory at %s — nothing recorded" % runs_dir)
    sys.exit(0)

files = sorted(glob.glob(os.path.join(runs_dir, "*.yaml")))
if not files:
    # A directory that EXISTS but holds no records is the vacuous-pass shape
    # (T-2831): "nothing to check" and "everything checks out" must not share
    # an exit code.
    sys.stderr.write("check-run-record-parse: %s exists but contains no *.yaml — "
                     "refusing to report clean on an empty corpus\n" % runs_dir)
    sys.exit(2)

firing = []
for f in files:
    rel = os.path.relpath(f)
    try:
        raw = open(f, encoding="utf-8").read()
    except Exception as e:
        sys.stderr.write("check-run-record-parse: cannot read %s (%s)\n" % (rel, e))
        sys.exit(2)                                       # fail-closed
    try:
        doc = yaml.safe_load(raw)
    except Exception as e:
        detail = str(e).replace("\n", " ")[:300]
        firing.append({"file": rel, "class": "UNPARSEABLE", "detail": detail})
        continue
    if not isinstance(doc, dict):
        firing.append({"file": rel, "class": "NOT-A-MAPPING",
                       "detail": "loads as %s; readers subscript it as a mapping"
                                 % type(doc).__name__})

ok = not firing
if as_json:
    print(json.dumps({"ok": ok, "firing": firing, "firing_count": len(firing),
                      "checked": len(files), "scope": SCOPE}, indent=2))
elif firing:
    print("check-run-record-parse: %d of %d run record(s) unreadable"
          % (len(firing), len(files)))
    for e in firing:
        print("  %-13s %s" % (e["class"], e["file"]))
        print("      %s" % e["detail"])
    print("")
    print("  SCOPE: %s" % SCOPE)
    print("")
    print("Remediation: a run record is the state carrier that lets a sequence resume")
    print("after a context reset — an unreadable one cannot be resumed and cannot be")
    print("told apart from a missing one. The recurring cause here is an unquoted")
    print("scalar wrapping onto a continuation line containing ': ' (T-3084 class).")
    print("Quote the scalar, then RE-PARSE before trusting the write.")
elif not quiet:
    print("check-run-record-parse: clean — %d run record(s) load as mappings" % len(files))
    print("  SCOPE: %s" % SCOPE)

sys.exit(1 if firing else 0)
PY
