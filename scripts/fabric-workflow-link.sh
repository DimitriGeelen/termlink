#!/usr/bin/env bash
# guard-layer: source
# fabric-workflow-link.sh — join workflow-designer nodes to component-fabric cards.
#
# Two ID spaces describing one system, with no reference between them (measured
# 2026-08-26: 393 cards, 8 workflows, zero links either way). This resolves the join
# declared in .fabric/workflow-links.yaml and REFUSES to let it rot.
#
# WHAT IT ANSWERS, per workflow node:
#   implements   the code this step IS
#   tested_by    the test that proves it
#   probed_by    the API/testbench tool that exercises it live
#   pseudocode   what it means, when no file says so
#
# THREE FAILURE MODES, KEPT DISTINCT. Collapsing them would make the check useless
# in exactly the case it exists for — a link can rot in three different directions
# and the remedy differs each time:
#
#   uid not found in the workflow   a node was renamed or deleted; the link now
#                                   points at nothing. The join is silently EMPTY,
#                                   which reads identically to "no link declared".
#   card not found in the fabric    the component was moved or its card removed;
#                                   the step claims code that is not registered.
#   workflow not found              the designer project was renamed.
#
# AND IT REPORTS UNLINKED NODES. A resolver that printed only its hits would assert
# coverage it does not have — the same PASS-on-no-candidates shape the framework
# already refuses elsewhere ("A PASS here would assert coverage the check does not
# have", T-3105). Coverage is printed as a fraction, always.
#
# Exit: 0 every declared link resolves · 1 at least one is broken · 2 cannot run
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

LINKS="${FABRIC_WORKFLOW_LINKS:-.fabric/workflow-links.yaml}"
CARDS="${FABRIC_COMPONENTS_DIR:-.fabric/components}"
PROJECTS="${DESIGNER_PROJECTS_DIR:-.agentic-framework/.context/designer/projects}"

if [ "${1:-}" = "--self-test" ]; then
  t=$(mktemp -d) || exit 2
  trap 'rm -rf "$t"' EXIT INT TERM HUP
  mkdir -p "$t/cards" "$t/proj/demo"
  printf 'id: src/real.rs\nname: real\nlocation: src/real.rs\npurpose: p\n' > "$t/cards/real.yaml"
  cat > "$t/proj/demo/v1.bpmn" <<'BPMN'
<bpmn:definitions xmlns:bpmn="b" xmlns:aef="a">
  <bpmn:serviceTask id="n1" name="present node">
    <bpmn:extensionElements><aef:uid value="present"/></bpmn:extensionElements>
  </bpmn:serviceTask>
  <bpmn:serviceTask id="n2" name="unlinked node">
    <bpmn:extensionElements><aef:uid value="lonely"/></bpmn:extensionElements>
  </bpmn:serviceTask>
</bpmn:definitions>
BPMN
  run() { FABRIC_WORKFLOW_LINKS="$1" FABRIC_COMPONENTS_DIR="$t/cards" \
          DESIGNER_PROJECTS_DIR="$t/proj" bash "$0" >/dev/null 2>&1; echo $?; }

  printf 'schema_version: 1\nlinks:\n  - workflow: demo\n    uid: present\n    implements: [src/real.rs]\n' > "$t/ok.yaml"
  printf 'schema_version: 1\nlinks:\n  - workflow: demo\n    uid: GONE\n    implements: [src/real.rs]\n' > "$t/baduid.yaml"
  printf 'schema_version: 1\nlinks:\n  - workflow: demo\n    uid: present\n    implements: [src/ghost.rs]\n' > "$t/badcard.yaml"
  printf 'schema_version: 1\nlinks:\n  - workflow: NOPE\n    uid: present\n    implements: [src/real.rs]\n' > "$t/badwf.yaml"

  fail=0
  # All four must be DISTINCT verdicts. "detects something" is satisfied by a
  # checker that fails on everything, so the healthy case is asserted too.
  [ "$(run "$t/ok.yaml")"      = "0" ] || { echo "self-test: FAIL valid link did not pass"; fail=1; }
  [ "$(run "$t/baduid.yaml")"  = "1" ] || { echo "self-test: FAIL uid missing from workflow not caught"; fail=1; }
  [ "$(run "$t/badcard.yaml")" = "1" ] || { echo "self-test: FAIL component with no fabric card not caught"; fail=1; }
  [ "$(run "$t/badwf.yaml")"   = "1" ] || { echo "self-test: FAIL unknown workflow not caught"; fail=1; }
  FABRIC_WORKFLOW_LINKS="$t/missing.yaml" FABRIC_COMPONENTS_DIR="$t/cards" \
    DESIGNER_PROJECTS_DIR="$t/proj" bash "$0" >/dev/null 2>&1
  [ $? -eq 2 ] || { echo "self-test: FAIL absent links file must be NO VERDICT (2), not clean"; fail=1; }
  [ "$fail" = "0" ] || exit 2
  echo "self-test: PASS — valid 0, dead uid 1, dead card 1, dead workflow 1, absent file 2"
  exit 0
fi

command -v python3 >/dev/null 2>&1 || { echo "fabric-workflow-link: no python3 — no verdict"; exit 2; }
[ -r "$LINKS" ] || { echo "fabric-workflow-link: $LINKS unreadable — NO VERDICT (not 'clean')"; exit 2; }
[ -d "$CARDS" ] || { echo "fabric-workflow-link: $CARDS missing — NO VERDICT"; exit 2; }
[ -d "$PROJECTS" ] || { echo "fabric-workflow-link: $PROJECTS missing — NO VERDICT"; exit 2; }

python3 - "$LINKS" "$CARDS" "$PROJECTS" "${1:-}" <<'PY'
import glob, os, re, sys, yaml

links_p, cards_d, proj_d, mode = sys.argv[1:5]

try:
    doc = yaml.safe_load(open(links_p, encoding="utf-8")) or {}
except Exception as e:
    print(f"fabric-workflow-link: {links_p} does not parse ({type(e).__name__}) — NO VERDICT")
    sys.exit(2)

# Card id -> card. The id IS the repo-relative path.
cards = {}
for p in glob.glob(os.path.join(cards_d, "*.yaml")):
    try:
        d = yaml.safe_load(open(p, encoding="utf-8"))
    except Exception:
        continue
    if d and d.get("id"):
        cards[d["id"]] = d

# workflow -> {uid: node_name}, taken from the HIGHEST version file present.
flows = {}
for wf in sorted(os.listdir(proj_d)):
    files = sorted(glob.glob(os.path.join(proj_d, wf, "*.bpmn")))
    if not files:
        continue
    s = open(files[-1], encoding="utf-8").read()
    uids = {}
    for m in re.finditer(
            r'<bpmn:(serviceTask|userTask|scriptTask|task|intermediateCatchEvent)\s+id="([^"]+)"'
            r'\s+name="([^"]*)"(.*?)</bpmn:\1>', s, re.S):
        _k, _nid, name, body = m.groups()
        u = re.search(r'aef:uid value="([^"]+)"', body)
        if u:
            uids[u.group(1)] = name
    flows[wf] = uids

rc = 0
broken = []
by_wf = {}
for ln in doc.get("links") or []:
    by_wf.setdefault(ln.get("workflow"), []).append(ln)

print(f"fabric-workflow-link: {len(cards)} fabric card(s), {len(flows)} designer workflow(s)")
print("  JOIN KEY: aef:uid (stable) -> fabric card id (repo-relative path).")
print("  Reports UNLINKED nodes too — a resolver showing only its hits would assert")
print("  coverage it does not have.")
print()

for wf, uids in flows.items():
    declared = {l["uid"]: l for l in by_wf.get(wf, [])}
    linked = [u for u in uids if u in declared]
    print(f"  {wf}  —  {len(linked)}/{len(uids)} node(s) linked")
    for u, name in uids.items():
        ln = declared.get(u)
        if not ln:
            print(f"    unlinked  {u:<18} {name[:52]}")
            continue
        parts = []
        for kind in ("implements", "tested_by", "probed_by"):
            for tgt in ln.get(kind) or []:
                if tgt in cards:
                    parts.append(f"{kind}={tgt}")
                else:
                    parts.append(f"{kind}={tgt}  <-- NO FABRIC CARD")
                    broken.append(f"{wf}/{u}: component '{tgt}' has no card in {cards_d}")
                    rc = 1
        print(f"    LINKED    {u:<18} {name[:52]}")
        for p in parts:
            print(f"                 {p}")
        if ln.get("pseudocode"):
            first = ln["pseudocode"].strip().splitlines()[0]
            print(f"                 pseudocode: {first[:60]}")
    print()

# Declared links whose workflow or uid no longer exists — the SILENT rot case.
for wf, lns in by_wf.items():
    if wf not in flows:
        for l in lns:
            broken.append(f"workflow '{wf}' not found under {proj_d} (link for uid '{l.get('uid')}')")
            rc = 1
        continue
    for l in lns:
        if l.get("uid") not in flows[wf]:
            broken.append(f"{wf}: uid '{l.get('uid')}' declared but not present in the workflow")
            rc = 1

if broken:
    print("  BROKEN LINKS:")
    for b in broken:
        print(f"    {b}")
    print()
    print("  A link pointing at nothing reads exactly like no link at all — that is")
    print("  why these fail rather than being skipped.")
else:
    print("  every declared link resolves.")
sys.exit(rc)
PY
