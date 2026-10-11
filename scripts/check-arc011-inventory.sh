#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-arc011-inventory.sh (T-3397, arc-011 PH-0 6.2.1.a (5), R-70)
#
# WHY: R-70 requires that the message-state and marker vocabulary is counted, not
# remembered. OP-7 ruled a tripwire of at most 14 state values and 8 markers. The
# numbers live in docs/design/arc-011-inventory.yaml; this check counts them again
# where they are DEFINED (section 7.3.a/b/c of the step-3 document, and any code
# root given with --code-root that carries `// arc011-state: NAME` /
# `// arc011-marker: NAME` annotations) and fires when the counts disagree with the
# file, when a name differs, or when a count exceeds the cap.
#
# Also pins the GP-12 numbers (7.2.a-c bounds, five real failures) to the inventory.
#
# SCOPE: it compares the vocabulary in the doc and in annotated code against the
# inventory. It does not prove a state is reachable. Today no code defines states
# (PH-1), so the code side is empty and is reported as 0, never as a pass of code.
#
# Exit 0 clean / 1 disagreement or cap exceeded / 2 tooling (fail-closed: a missing
# or unparseable file, or an anchor that extracts nothing, is never a clean result).
# Usage: check-arc011-inventory.sh [--json] [--quiet] [--no-heartbeat]
#          [--inventory P] [--doc P] [--code-root DIR]
set -uo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INV="$REPO_ROOT/docs/design/arc-011-inventory.yaml"
DOC=""; CODE_ROOT=""; WANT_JSON=0; QUIET=0
while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1 ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) ;;
        --inventory) INV="$2"; shift ;;
        --doc) DOC="$2"; shift ;;
        --code-root) CODE_ROOT="$2"; shift ;;
        -h|--help) sed -n 2,22p "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done
command -v python3 >/dev/null 2>&1 || { echo "check-arc011-inventory: python3 missing" >&2; exit 2; }
INV="$INV" DOC="$DOC" CODE_ROOT="$CODE_ROOT" WANT_JSON="$WANT_JSON" QUIET="$QUIET" REPO_ROOT="$REPO_ROOT" \
python3 -I - <<'PY'
import json, os, re, sys
try:
    import yaml
except ImportError:
    print("check-arc011-inventory: PyYAML missing", file=sys.stderr); sys.exit(2)

def tool(msg):
    print("check-arc011-inventory: " + msg, file=sys.stderr); sys.exit(2)

root = os.environ["REPO_ROOT"]
try:
    inv = yaml.safe_load(open(os.environ["INV"]))
    assert isinstance(inv, dict)
    inv_chain = list(inv["states"]["chain"]); inv_lab = list(inv["states"]["sender_labels"])
    inv_states = inv_chain + inv_lab
    inv_markers = list(inv["markers"])
    cap_s, cap_m = int(inv["cap"]["states"]), int(inv["cap"]["markers"])
    cnt_s, cnt_m = int(inv["counts"]["states"]), int(inv["counts"]["markers"])
    gp = inv["gp12"]
except Exception as e:
    tool("cannot read inventory: %s" % e)
doc = os.environ["DOC"] or os.path.join(root, inv.get("source_doc", ""))
try:
    lines = open(doc).read().splitlines()
except Exception as e:
    tool("cannot read doc: %s" % e)

def anchor(prefix):
    for l in lines:
        if l.startswith(prefix + " "):
            return l
    tool("anchor %s not found in %s" % (prefix, doc))

def seg(line, start, end):
    i = line.find(start)
    if i < 0: tool("marker %r missing" % start)
    i += len(start); j = line.find(end, i)
    if j < 0: tool("marker %r missing" % end)
    return [x.strip().strip("`*").strip() for x in line[i:j].split(",") if x.strip()]

def num(pattern, line):
    m = re.search(pattern, line)
    if not m: tool("pattern %r not found" % pattern)
    return int(m.group(1))

doc_chain = seg(anchor("7.3.a"), "derived):", "(REQ")
doc_lab = seg(anchor("7.3.b"), "shared):", "(R-44)")
l3c = anchor("7.3.c")
doc_markers = re.findall(r'"([^"]+)"', l3c.split(":**", 1)[1] if ":**" in l3c else l3c)
m = re.search(r"at most \*\*(\d+) state values\*\* and \*\*(\d+) markers\*\*", anchor("7.3.d"))
if not m: tool("cap sentence of 7.3.d not found")
doc_cap = (int(m.group(1)), int(m.group(2)))
if not doc_chain or not doc_lab or not doc_markers: tool("an anchor extracted nothing")

code_s, code_m = [], []
cr = os.environ["CODE_ROOT"]
if cr:
    if not os.path.isdir(cr): tool("code root not a directory: " + cr)
    for d, _, fs in os.walk(cr):
        for f in fs:
            try: t = open(os.path.join(d, f), errors="replace").read()
            except OSError: continue
            code_s += re.findall(r"//\s*arc011-state:\s*(\S.*?)\s*$", t, re.M)
            code_m += re.findall(r"//\s*arc011-marker:\s*(\S.*?)\s*$", t, re.M)

a7a, a7b, a7c = anchor("7.2.a"), anchor("7.2.b"), anchor("7.2.c")
gp_doc = {
    "detection_to_diagnosis_seconds": num(r"Detection to diagnosis: (\d+) seconds", a7a),
    "start_or_tick_check_seconds": num(r"every tick the bound is (\d+) seconds", a7a),
    "query_seconds": num(r"Query time: (\d+) seconds", a7b),
    "drilled_human_minutes": num(r"Drilled human time: (\d+) minutes", a7c),
    "real_failures": len(re.findall(r"^\| [1-9]\d* \| ", "\n".join(lines[lines.index(anchor("7.2.d")):lines.index(anchor("7.2.e"))]), re.M)),
}

findings = []
def need(cond, msg):
    if not cond: findings.append(msg)
all_doc = doc_chain + doc_lab
need(doc_chain == inv_chain, "chain states differ: doc=%s inventory=%s" % (doc_chain, inv_chain))
need(doc_lab == inv_lab, "sender labels differ: doc=%s inventory=%s" % (doc_lab, inv_lab))
need(doc_markers == inv_markers, "markers differ: doc=%s inventory=%s" % (doc_markers, inv_markers))
need(len(set(all_doc)) == len(all_doc), "duplicate state names in doc")
need(len(inv_states) == cnt_s, "inventory declares %d states but lists %d" % (cnt_s, len(inv_states)))
need(len(inv_markers) == cnt_m, "inventory declares %d markers but lists %d" % (cnt_m, len(inv_markers)))
need(len(all_doc) == cnt_s, "doc defines %d states, inventory counts %d" % (len(all_doc), cnt_s))
need(len(doc_markers) == cnt_m, "doc defines %d markers, inventory counts %d" % (len(doc_markers), cnt_m))
need((cap_s, cap_m) == doc_cap, "cap differs: inventory=%s doc=%s" % ((cap_s, cap_m), doc_cap))
need(len(inv_states) <= cap_s, "CAP EXCEEDED: %d states > cap %d (7.3.e review required)" % (len(inv_states), cap_s))
need(len(inv_markers) <= cap_m, "CAP EXCEEDED: %d markers > cap %d (7.3.e review required)" % (len(inv_markers), cap_m))
need(len(all_doc) <= cap_s, "CAP EXCEEDED in doc: %d states > cap %d" % (len(all_doc), cap_s))
need(len(doc_markers) <= cap_m, "CAP EXCEEDED in doc: %d markers > cap %d" % (len(doc_markers), cap_m))
if cr:
    need(sorted(set(code_s)) == sorted(inv_states), "code states differ from inventory: %s" % sorted(set(code_s) ^ set(inv_states)))
    need(sorted(set(code_m)) == sorted(inv_markers), "code markers differ from inventory: %s" % sorted(set(code_m) ^ set(inv_markers)))
    need(len(set(code_s)) <= cap_s and len(set(code_m)) <= cap_m, "CAP EXCEEDED in code")
for k, v in gp_doc.items():
    need(gp.get(k) == v, "gp12.%s: inventory=%s doc=%s" % (k, gp.get(k), v))

ok = not findings
res = {"ok": ok, "states": {"doc": len(all_doc), "inventory": len(inv_states), "code": len(set(code_s)), "cap": cap_s},
       "markers": {"doc": len(doc_markers), "inventory": len(inv_markers), "code": len(set(code_m)), "cap": cap_m},
       "gp12": gp_doc, "findings": findings,
       "scope": "compares doc/annotated-code vocabulary with the inventory; does not prove reachability"}
if os.environ["WANT_JSON"] == "1":
    print(json.dumps(res))
elif not ok or os.environ["QUIET"] != "1":
    print("arc-011 inventory: states doc %d / inventory %d (cap %d), markers doc %d / inventory %d (cap %d), code-annotated %d/%d"
          % (len(all_doc), len(inv_states), cap_s, len(doc_markers), len(inv_markers), cap_m, len(set(code_s)), len(set(code_m))))
    for f in findings: print("  FIRING: " + f)
    if ok: print("  clean (scope: " + res["scope"] + ")")
sys.exit(0 if ok else 1)
PY
