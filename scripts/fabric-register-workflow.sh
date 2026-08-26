#!/usr/bin/env bash
# fabric-register-workflow.sh — register a workflow's steps as fabric components.
#
# "A process step and a code snippet IS a component ID, in the component fabric."
# This is the adoption operation: every node of a workflow gets a card, so the workflow
# is fully represented and its unlinked steps are VISIBLE rather than absent.
#
# WHY THIS IS A SHIPPED SCRIPT AND NOT A ONE-OFF. The generator that registered
# aef-dispatch-loop lived in an ephemeral job directory. T-2824 already had to commit a
# merge toolkit "so the plan does not point at an ephemeral job dir", and I was about to
# repeat that: the recipe for registering a workflow existed only in a scratch file and in
# a commit message. Anyone adopting their own workflow — 001-CashWeb's phase-1 map is the
# live case — needs the operation, not the transcript of me running it.
#
# IDEMPOTENT AND NON-DESTRUCTIVE. Re-running never clobbers an existing card's
# depends_on/pseudocode: edges are authored by hand after adoption, and a regeneration
# that silently dropped them would be the same silent-loss class this arc keeps finding.
# Only missing cards are created; present ones are left alone and reported.
#
# EDGES ARE NOT INVENTED. This registers steps. Relations are added afterwards with
# --link, which REFUSES a target that has no fabric card — a link to an unregistered
# component is a claim about something the fabric does not know.
#
# THE RECIPROCAL EDGE IS WRITTEN AUTOMATICALLY, because forgetting it is the failure that
# looks like success: `fw fabric deps` reads the TARGET's depended_by list and does NOT
# scan for inbound depends_on, so a forward-only edge is complete in the file and invisible
# at the terminal.
#
# Usage:
#   fabric-register-workflow.sh <workflow>                      adopt: card every node
#   fabric-register-workflow.sh <workflow> --link UID=PATH:TYPE  add one relation
#   fabric-register-workflow.sh --list                          workflows and coverage
#   fabric-register-workflow.sh --self-test
#
# TYPE is one of implements | tested_by | probed_by.
# Exit: 0 ok · 1 refused (bad uid / uncarded target) · 2 cannot run
set -uo pipefail

# FW_PROJECT_ROOT lets one shipped tool serve every project. Without it the relpath
# for a card's `location:` is computed against THIS repo, so adopting another
# project's workflow would write a location that resolves nowhere — a card that
# looks registered and is invisible to `fw fabric drift`.
cd "${FW_PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)}" || exit 1

CARDS="${FABRIC_COMPONENTS_DIR:-.fabric/components}"
PROJECTS="${DESIGNER_PROJECTS_DIR:-.agentic-framework/.context/designer/projects}"

command -v python3 >/dev/null 2>&1 || { echo "fabric-register-workflow: no python3 — no verdict"; exit 2; }

if [ "${1:-}" = "--self-test" ]; then
  t=$(mktemp -d) || exit 2
  trap 'rm -rf "$t"' EXIT INT TERM HUP
  mkdir -p "$t/cards" "$t/proj/demo"
  printf 'id: src/real.rs\nname: real\nlocation: src/real.rs\npurpose: p\n' > "$t/cards/real.yaml"
  cat > "$t/proj/demo/v1.bpmn" <<'BPMN'
<bpmn:definitions xmlns:bpmn="b" xmlns:aef="a">
  <bpmn:serviceTask id="n1" name="one"><bpmn:extensionElements><aef:uid value="one"/></bpmn:extensionElements></bpmn:serviceTask>
  <bpmn:serviceTask id="n2" name="two"><bpmn:extensionElements><aef:uid value="two"/></bpmn:extensionElements></bpmn:serviceTask>
</bpmn:definitions>
BPMN
  R() { FABRIC_COMPONENTS_DIR="$t/cards" DESIGNER_PROJECTS_DIR="$t/proj" bash "$0" "$@" >/dev/null 2>&1; echo $?; }
  fail=0
  [ "$(R demo)" = "0" ] || { echo "self-test: FAIL adoption did not succeed"; fail=1; }
  [ -f "$t/cards/workflow-demo-one.yaml" ] || { echo "self-test: FAIL card not written"; fail=1; }
  # idempotent: a hand-authored edge must survive re-registration
  printf 'id: workflow/demo/one\nname: one\ntype: workflow-step\nlocation: demo/v1.bpmn\nworkflow: demo\naef_uid: one\ndepends_on:\n- target: src/real.rs\n  type: implements\n' > "$t/cards/workflow-demo-one.yaml"
  R demo >/dev/null
  grep -q 'src/real.rs' "$t/cards/workflow-demo-one.yaml" || { echo "self-test: FAIL re-run clobbered a hand-authored edge"; fail=1; }
  # refuses an unknown uid and an uncarded target — the two ways a link can be a lie
  [ "$(R demo --link NOPE=src/real.rs:implements)" = "1" ] || { echo "self-test: FAIL unknown uid not refused"; fail=1; }
  [ "$(R demo --link one=src/ghost.rs:implements)" = "1" ] || { echo "self-test: FAIL uncarded target not refused"; fail=1; }
  # and the healthy link writes BOTH directions
  [ "$(R demo --link two=src/real.rs:implements)" = "0" ] || { echo "self-test: FAIL valid link refused"; fail=1; }
  grep -q 'workflow/demo/two' "$t/cards/real.yaml" || { echo "self-test: FAIL reciprocal edge not written to the target"; fail=1; }
  # DISCRIMINATING LEG for the version sort. Every fixture above has ONE .bpmn, so none
  # of them can tell a lexical sort from a numeric one. With v2 and v10 present, lexical
  # picks v2 (wrong) and numeric picks v10. Caught in the field, not in review: adopting
  # a real map registered 8 of 14 nodes and pinned every card location to v9 while the
  # workflow was at v12 — silent, and drift saw no orphan because v9 still exists.
  mkdir -p "$t/proj/vers"
  printf '%s\n' '<bpmn:definitions xmlns:bpmn="b" xmlns:aef="a"><bpmn:serviceTask id="a" name="old"><bpmn:extensionElements><aef:uid value="from_v2"/></bpmn:extensionElements></bpmn:serviceTask></bpmn:definitions>' > "$t/proj/vers/v2.bpmn"
  printf '%s\n' '<bpmn:definitions xmlns:bpmn="b" xmlns:aef="a"><bpmn:serviceTask id="a" name="new"><bpmn:extensionElements><aef:uid value="from_v10"/></bpmn:extensionElements></bpmn:serviceTask></bpmn:definitions>' > "$t/proj/vers/v10.bpmn"
  R vers >/dev/null
  [ -f "$t/cards/workflow-vers-from-v10.yaml" ] || { echo "self-test: FAIL v10 not chosen over v2 — lexical version sort"; fail=1; }
  if [ -f "$t/cards/workflow-vers-from-v2.yaml" ]; then echo "self-test: FAIL registered from the OLDER version"; fail=1; fi

  [ "$fail" = "0" ] || exit 2
  echo "self-test: PASS — adopt 0, re-run preserves edges, bad uid 1, uncarded target 1, link writes both directions, v10 beats v2"
  exit 0
fi

[ -d "$CARDS" ] || { echo "fabric-register-workflow: $CARDS missing — NO VERDICT"; exit 2; }
[ -d "$PROJECTS" ] || { echo "fabric-register-workflow: $PROJECTS missing — NO VERDICT"; exit 2; }

python3 - "$CARDS" "$PROJECTS" "$@" <<'PY'
import glob, os, re, sys, yaml

cards_d, proj_d = sys.argv[1], sys.argv[2]
args = sys.argv[3:]
def _vkey(p):
    """Sort .bpmn by NUMERIC version. Lexically v9 > v12, which silently pins every
    card to a stale workflow: nodes added since are absent, and the stale location
    still exists so drift reports no orphan."""
    m = re.search(r"v(\d+)\.bpmn$", p)
    return (int(m.group(1)) if m else -1, p)

INVERSE = {"implements": "implemented_by", "tested_by": "tests", "probed_by": "probes"}

def load():
    by_id = {}
    for p in glob.glob(os.path.join(cards_d, "*.yaml")):
        try:
            d = yaml.safe_load(open(p, encoding="utf-8"))
        except Exception:
            continue
        if d and d.get("id"):
            by_id[d["id"]] = (p, d)
    return by_id

def nodes_of(wf):
    fs = sorted(glob.glob(os.path.join(proj_d, wf, "*.bpmn")), key=_vkey)
    if not fs:
        return None, []
    s = open(fs[-1], encoding="utf-8").read()
    out = []
    for m in re.finditer(
            r'<bpmn:(serviceTask|userTask|scriptTask|task|intermediateCatchEvent)\s+id="([^"]+)"'
            r'\s+name="([^"]*)"(.*?)</bpmn:\1>', s, re.S):
        kind, nid, name, body = m.groups()
        u = re.search(r'aef:uid value="([^"]+)"', body)
        if u:
            out.append((u.group(1), nid, kind, name))
    return fs[-1], out

by_id = load()

if not args or args[0] == "--list":
    print(f"{'WORKFLOW':<38} {'REGISTERED':<12} STATE")
    print("-" * 62)
    for wf in sorted(os.listdir(proj_d)):
        f, ns = nodes_of(wf)
        if not f:
            continue
        got = sum(1 for u, _n, _k, _m in ns if f"workflow/{wf}/{u}" in by_id)
        state = "ADOPTED" if ns and got >= len(ns) else ("partial" if got else "-")
        print(f"{wf:<38} {got}/{len(ns):<10} {state}")
    sys.exit(0)

wf = args[0]
f, ns = nodes_of(wf)
if not f:
    print(f"fabric-register-workflow: no .bpmn under {proj_d}/{wf}")
    sys.exit(1)
if not ns:
    print(f"fabric-register-workflow: {wf} parsed to ZERO aef:uid nodes — refusing")
    print("  (an empty parse must not read as 'nothing to register')")
    sys.exit(1)
rel = os.path.relpath(f, ".")

# ---- --link UID=PATH:TYPE -------------------------------------------------
if "--link" in args:
    spec = args[args.index("--link") + 1]
    uid, rest = spec.split("=", 1)
    tgt, typ = rest.rsplit(":", 1)
    if typ not in INVERSE:
        print(f"unknown type '{typ}' — use one of {', '.join(INVERSE)}"); sys.exit(1)
    if uid not in {u for u, _n, _k, _m in ns}:
        print(f"REFUSED: uid '{uid}' is not in {wf} — a link to a step that does not exist")
        sys.exit(1)
    if tgt not in by_id:
        print(f"REFUSED: '{tgt}' has no fabric card — register it first")
        sys.exit(1)
    cid = f"workflow/{wf}/{uid}"
    if cid not in by_id:
        print(f"REFUSED: {cid} not registered — adopt the workflow first"); sys.exit(1)

    sp, sd = by_id[cid]
    deps = sd.get("depends_on") or []
    if not any(d.get("target") == tgt and d.get("type") == typ for d in deps):
        deps.append({"target": tgt, "type": typ})
        sd["depends_on"] = deps
        open(sp, "w", encoding="utf-8").write(
            yaml.safe_dump(sd, sort_keys=False, allow_unicode=True, width=120))
    tp, td = by_id[tgt]
    dby = td.get("depended_by") or []
    if not any(e.get("target") == cid for e in dby):
        dby.append({"target": cid, "type": INVERSE[typ]})
        td["depended_by"] = dby
        open(tp, "w", encoding="utf-8").write(
            yaml.safe_dump(td, sort_keys=False, allow_unicode=True, width=120))
    print(f"linked  {cid}  --{typ}-->  {tgt}")
    print(f"        and wrote the reciprocal {INVERSE[typ]} on {tgt} (without it,")
    print(f"        `fw fabric deps {tgt}` would not show the step)")
    sys.exit(0)

# ---- adoption -------------------------------------------------------------
made, kept = [], []
for uid, nid, kind, name in ns:
    cid = f"workflow/{wf}/{uid}"
    if cid in by_id:
        kept.append(cid)
        continue
    card = {
        "id": cid, "name": uid, "type": "workflow-step",
        "subsystem": "workflow-designer", "location": rel,
        "tags": ["workflow-step", wf, "designer"],
        "purpose": (f"Workflow step (aef:uid {uid}, node {nid}, {kind}) — "
                    f"{re.sub(r'&[a-z]+;', '', name).strip()}"),
        "workflow": wf, "aef_uid": uid, "bpmn_node_id": nid,
        "created_by": "T-2839",
    }
    fn = ("workflow-" + wf + "-" + uid.replace("_", "-")).lower() + ".yaml"
    open(os.path.join(cards_d, fn), "w", encoding="utf-8").write(
        yaml.safe_dump(card, sort_keys=False, allow_unicode=True, width=120))
    made.append(cid)

print(f"{wf}: {len(ns)} node(s) — {len(made)} registered, {len(kept)} already present")
for c in made:
    print(f"  + {c}")
if kept:
    print(f"  {len(kept)} left untouched (existing edges preserved)")
print()
print("  Steps are registered WITHOUT relations. Add them with:")
print(f"    scripts/fabric-register-workflow.sh {wf} --link <uid>=<path>:implements")
print("  Edges are never invented — a target with no fabric card is refused.")
PY
