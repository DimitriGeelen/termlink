#!/usr/bin/env bash
# guard-layer: source
# fabric-workflow-link.sh — validate workflow steps registered as fabric components.
#
# REWRITTEN 2026-08-27. The first version read a sidecar (.fabric/workflow-links.yaml)
# that related two ID spaces. After the operator's correction — "a process step and a
# code snippet IS a component ID, in the component fabric" — the steps became first-class
# cards and the sidecar became a second copy of the same relation.
#
# PL-362: a deduplication that adds the shared definition without DELETING the copies
# leaves the codebase strictly worse than before. So the sidecar's 5 remaining links were
# promoted into cards and the sidecar is gone. This reads cards only. There is now ONE
# representation of the relation.
#
# WHAT IT CHECKS, and the third one is the one that silently breaks:
#
#   1. aef_uid resolves      the card's uid must exist in the .bpmn at its location.
#                            A renamed node leaves a card pointing at nothing, and the
#                            join reads exactly like "no link declared".
#   2. targets have cards    a depends_on target with no card is a claim about a
#                            component the fabric does not know.
#   3. RECIPROCAL EDGE       `fw fabric deps` reads the TARGET's depended_by list — it
#                            does NOT scan the corpus for inbound depends_on. A step
#                            declaring `depends_on: <code>` is INVISIBLE from the code
#                            side until the code card declares the inverse. Forward-only
#                            edges look complete in the file and answer nothing at the
#                            terminal. Measured: that is exactly what the first draft
#                            shipped with for ten minutes.
#
# ADOPTION IS REPORTED, NOT ASSUMED. A workflow is ADOPTED when every node carries a card
# (aef-dispatch-loop, 6/6 — including four with no implementing component, so its gaps are
# visible). Others are PARTIAL: only steps with a real relation are registered. Printing
# which is which keeps the rule visible instead of implicit, and stops a partial workflow
# reading as a complete one.
#
# Exit: 0 every registered step resolves · 1 something is broken · 2 cannot run
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

CARDS="${FABRIC_COMPONENTS_DIR:-.fabric/components}"
PROJECTS="${DESIGNER_PROJECTS_DIR:-.agentic-framework/.context/designer/projects}"

if [ "${1:-}" = "--self-test" ]; then
  t=$(mktemp -d) || exit 2
  trap 'rm -rf "$t"' EXIT INT TERM HUP
  mkdir -p "$t/cards" "$t/proj/demo"
  cat > "$t/proj/demo/v1.bpmn" <<'BPMN'
<bpmn:definitions xmlns:bpmn="b" xmlns:aef="a">
  <bpmn:serviceTask id="n1" name="present"><bpmn:extensionElements><aef:uid value="present"/></bpmn:extensionElements></bpmn:serviceTask>
  <bpmn:serviceTask id="n2" name="bare"><bpmn:extensionElements><aef:uid value="bare"/></bpmn:extensionElements></bpmn:serviceTask>
</bpmn:definitions>
BPMN
  mkcode() { printf 'id: src/real.rs\nname: real\nlocation: src/real.rs\npurpose: p\n%s' "$1" > "$t/cards/real.yaml"; }
  mkstep() { printf 'id: workflow/demo/%s\nname: %s\ntype: workflow-step\nlocation: demo/v1.bpmn\nworkflow: demo\naef_uid: %s\ndepends_on:\n- target: src/real.rs\n  type: implements\n' "$1" "$1" "$1" > "$t/cards/step.yaml"; }
  run() { FABRIC_COMPONENTS_DIR="$t/cards" DESIGNER_PROJECTS_DIR="$t/proj" bash "$0" >/dev/null 2>&1; echo $?; }

  fail=0
  # healthy: uid present, target carded, reciprocal edge present
  mkcode 'depended_by:
- target: workflow/demo/present
  type: implemented_by
'; mkstep present
  [ "$(run)" = "0" ] || { echo "self-test: FAIL healthy step did not pass"; fail=1; }

  # THE DISCRIMINATING CASE: everything valid EXCEPT the reciprocal edge. Both a
  # correct and a forward-only join look identical from the step card alone — only
  # the target's depended_by separates them, which is precisely why this leg exists.
  mkcode ''; mkstep present
  [ "$(run)" = "1" ] || { echo "self-test: FAIL missing reciprocal edge not caught"; fail=1; }

  # uid renamed out of the workflow
  mkcode 'depended_by:
- target: workflow/demo/GONE
  type: implemented_by
'; mkstep GONE
  [ "$(run)" = "1" ] || { echo "self-test: FAIL uid absent from workflow not caught"; fail=1; }

  # target has no card
  printf 'id: workflow/demo/present\nname: present\ntype: workflow-step\nlocation: demo/v1.bpmn\nworkflow: demo\naef_uid: present\ndepends_on:\n- target: src/ghost.rs\n  type: implements\n' > "$t/cards/step.yaml"
  rm -f "$t/cards/real.yaml"
  [ "$(run)" = "1" ] || { echo "self-test: FAIL target without a card not caught"; fail=1; }

  FABRIC_COMPONENTS_DIR="$t/nope" DESIGNER_PROJECTS_DIR="$t/proj" bash "$0" >/dev/null 2>&1
  [ $? -eq 2 ] || { echo "self-test: FAIL missing cards dir must be NO VERDICT (2)"; fail=1; }

  [ "$fail" = "0" ] || exit 2
  echo "self-test: PASS — healthy 0, no-reciprocal 1, dead uid 1, uncarded target 1, missing dir 2"
  exit 0
fi

command -v python3 >/dev/null 2>&1 || { echo "fabric-workflow-link: no python3 — no verdict"; exit 2; }
[ -d "$CARDS" ] || { echo "fabric-workflow-link: $CARDS missing — NO VERDICT (not 'clean')"; exit 2; }
[ -d "$PROJECTS" ] || { echo "fabric-workflow-link: $PROJECTS missing — NO VERDICT"; exit 2; }

python3 - "$CARDS" "$PROJECTS" <<'PY'
import glob, os, re, sys, yaml
def _vkey(p):
    """Sort .bpmn by NUMERIC version. Lexically v9 > v12, which silently pins every
    card to a stale workflow: nodes added since are absent, and the stale location
    still exists so drift reports no orphan."""
    m = re.search(r"v(\d+)\.bpmn$", p)
    return (int(m.group(1)) if m else -1, p)


cards_d, proj_d = sys.argv[1], sys.argv[2]

cards, steps = {}, []
for p in glob.glob(os.path.join(cards_d, "*.yaml")):
    try:
        d = yaml.safe_load(open(p, encoding="utf-8"))
    except Exception:
        continue
    if not d or not d.get("id"):
        continue
    cards[d["id"]] = d
    if d.get("type") == "workflow-step":
        steps.append(d)

# uid -> name, per workflow, from the highest version file.
flows = {}
for wf in sorted(os.listdir(proj_d)):
    fs = sorted(glob.glob(os.path.join(proj_d, wf, "*.bpmn")), key=_vkey)
    if not fs:
        continue
    s = open(fs[-1], encoding="utf-8").read()
    u = {}
    for m in re.finditer(
            r'<bpmn:(serviceTask|userTask|scriptTask|task|intermediateCatchEvent)\s+id="([^"]+)"'
            r'\s+name="([^"]*)"(.*?)</bpmn:\1>', s, re.S):
        _k, _n, name, body = m.groups()
        g = re.search(r'aef:uid value="([^"]+)"', body)
        if g:
            u[g.group(1)] = name
    flows[wf] = u


INVERSE = {"implements": "implemented_by", "tested_by": "tests", "probed_by": "probes"}
broken, rc = [], 0

print(f"fabric-workflow-link: {len(steps)} workflow-step card(s) of {len(cards)} components")
print("  A process step IS a component. One ID space — no sidecar.")
print("  Checks uid resolution, target cards, and the RECIPROCAL edge (`fw fabric deps`")
print("  reads the target's depended_by; a forward-only edge answers nothing).")
print()

by_wf = {}
for s in steps:
    by_wf.setdefault(s.get("workflow"), []).append(s)

for wf in sorted(by_wf):
    total = len(flows.get(wf, {}))
    got = len(by_wf[wf])
    tag = "ADOPTED" if total and got >= total else "partial"
    print(f"  {wf}  —  {got}/{total} node(s) registered  [{tag}]")
    for s in sorted(by_wf[wf], key=lambda x: x["id"]):
        uid = s.get("aef_uid")
        if wf not in flows:
            broken.append(f"{s['id']}: workflow '{wf}' not found under {proj_d}"); rc = 1
        elif uid not in flows[wf]:
            broken.append(f"{s['id']}: aef_uid '{uid}' not present in the workflow"); rc = 1
        deps = s.get("depends_on") or []
        if not deps:
            print(f"    {uid:<18} (no implementing component)")
            continue
        print(f"    {uid:<18}")
        for d in deps:
            tgt, typ = d.get("target"), d.get("type")
            t = cards.get(tgt)
            if not t:
                print(f"        {typ}={tgt}   <-- NO FABRIC CARD")
                broken.append(f"{s['id']}: target '{tgt}' has no card"); rc = 1
                continue
            inv = INVERSE.get(typ, typ)
            back = any(e.get("target") == s["id"] for e in (t.get("depended_by") or []))
            mark = "" if back else "   <-- NO RECIPROCAL EDGE (invisible to `fw fabric deps`)"
            print(f"        {typ}={tgt}{mark}")
            if not back:
                broken.append(f"{tgt}: missing depended_by {inv} -> {s['id']}"); rc = 1
    print()

if broken:
    print("  BROKEN:")
    for b in broken:
        print(f"    {b}")
else:
    print("  every registered step resolves, and every edge is reciprocal.")
sys.exit(rc)
PY
