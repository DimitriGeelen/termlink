#!/usr/bin/env bash
# guard-layer: source warn  # arc-011's three drivers carry this exact gap until the operator re-attaches their specs (T-3440 5c, T-3472); FAIL tier once they are attached
#
# check-arc-driver-scoring-persisted.sh — T-3472 (G-019 detection for the
# approve-driver persistence bug).
#
# `fw arc approve-driver --scoring-file` used to pass the spec to the static
# reviewer, record "check (a) scorable: inline scoring: validates", and then write
# the scoped_drivers entry WITHOUT the `scoring:` block. The estimator reads only an
# inline block, so the driver scored nothing while its reviewer record said it
# would. arc-011's three drivers (approved 2026-10-06) were left exactly so, and
# nothing flagged it: the audit's unscored-driver WARN fires, but cannot tell "never
# had a spec" from "the spec was approved and then lost".
#
# This check fires on that second shape: a scoped driver whose reviewer record says
# check (a) passed with a scoring spec, but whose entry has no `scoring:` block.
#
# Scope (read a green narrowly): it compares the reviewer record with the stored
# entry. It does not judge whether a spec is good, and a driver approved on the
# human path (no reviewer record) is outside it; the audit's unscored WARN covers
# that.
#
# Exit 0 clean · 1 firing · 2 tooling (missing dir, unparseable arc). Flags:
# --quiet (print only when firing), --arcs-dir DIR (fixtures).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCS="$ROOT/.context/arcs"; QUIET=0
while [ $# -gt 0 ]; do
    case "$1" in
        --quiet) QUIET=1; shift;;
        --arcs-dir) ARCS="$2"; shift 2;;
        -h|--help) sed -n '2,25p' "$0"; exit 0;;
        *) echo "unknown arg: $1" >&2; exit 2;;
    esac
done
[ -d "$ARCS" ] || { echo "check-arc-driver-scoring-persisted: no arcs dir $ARCS" >&2; exit 2; }
QUIET="$QUIET" python3 - "$ARCS" <<'PY'
import glob, os, sys, yaml
arcs = sorted(glob.glob(os.path.join(sys.argv[1], "*.yaml")))
if not arcs:
    print("check-arc-driver-scoring-persisted: no arc files", file=sys.stderr); sys.exit(2)
firing, checked = [], 0
for fn in arcs:
    try:
        d = yaml.safe_load(open(fn)) or {}
    except Exception as e:
        print(f"check-arc-driver-scoring-persisted: {fn}: unparseable ({e})", file=sys.stderr); sys.exit(2)
    for sd in d.get("scoped_drivers") or []:
        if not isinstance(sd, dict):
            continue
        checked += 1
        a = (((sd.get("reviewer") or {}).get("checks") or {}).get("a") or {})
        said_spec = a.get("verdict") == "pass" and "scoring" in str(a.get("reason", ""))
        has_spec = isinstance(sd.get("scoring"), dict) and bool(sd.get("scoring"))
        if said_spec and not has_spec:
            firing.append((os.path.basename(fn), sd.get("name"), a.get("reason")))
quiet = os.environ.get("QUIET") == "1"
if firing:
    print(f"check-arc-driver-scoring-persisted: FIRING — {len(firing)} driver(s) reviewed as scorable but stored without a scoring: block")
    for arc, name, reason in firing:
        print(f"  FIRING  {arc}: {name}  (reviewer said: {reason})")
    print("  operator: re-attach the spec with `fw arc approve-driver <arc> \"<name>\" --scoring-file F --i-am-human` (attaches in place since T-3472)")
    sys.exit(1)
if not quiet:
    print(f"check-arc-driver-scoring-persisted: clean — {checked} scoped driver(s) checked")
    print("  SCOPE: compares each reviewer record with the stored entry; does not judge spec quality or cover human-approved drivers")
sys.exit(0)
PY
