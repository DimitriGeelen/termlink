#!/usr/bin/env bash
# guard-layer: source
# T-3544 (operator ruling A'1 on T-3103) — the designer workflows this project has adopted still match
# their step cards, and every vendored workflow is either adopted or declined on purpose.
#
# Reads .fabric/workflow-adoption.yaml (the project's own manifest; not vendored) against the vendored
# designer projects (.agentic-framework/.context/designer/projects/<name>/v<N>.bpmn) and the
# `type: workflow-step` cards in .fabric/components/.
#
# FAIL (exit 1):
#   - a designer project folder that the manifest does not list (a re-vendor added a workflow silently)
#   - a manifest entry whose project folder is gone, or whose adopted v<N>.bpmn is gone
#   - an adopted workflow whose step cards differ from the adopted diagram's steps (missing or stale)
#   - a step card whose location names a different version than the one the manifest adopted
#   - .fabric/watch-patterns.yaml not watching exactly the adopted (workflow, version) diagrams
# WARN (printed, exit 0): a newer v<N> exists than the adopted one.
# Exit 2: tooling (manifest unreadable, no projects dir, no python3/PyYAML) — never a clean result.
#
# "Step" is the adoption tool's definition (scripts/fabric-register-workflow.sh): serviceTask,
# userTask, scriptTask, task or intermediateCatchEvent carrying an aef:uid. A fixture pins the two
# regexes equal, so the check and the tool cannot drift apart.
# Seams: WORKFLOW_ADOPTION_ROOT (repo root to read), WORKFLOW_ADOPTION_PROJECTS (designer projects dir).
ROOT="${WORKFLOW_ADOPTION_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
PROJ="${WORKFLOW_ADOPTION_PROJECTS:-$ROOT/.agentic-framework/.context/designer/projects}"
command -v python3 >/dev/null 2>&1 || { echo "check-workflow-adoption: python3 not found" >&2; exit 2; }
exec python3 - "$ROOT" "$PROJ" <<'PY'
import glob, os, re, sys
try:
    import yaml
except ImportError:
    print("check-workflow-adoption: PyYAML not installed", file=sys.stderr); sys.exit(2)
root, proj = sys.argv[1], sys.argv[2]
STEP_RE = (r'<bpmn:(serviceTask|userTask|scriptTask|task|intermediateCatchEvent)\s+id="([^"]+)"'
           r'\s+name="([^"]*)"(.*?)</bpmn:\1>')

def steps(path):
    s = open(path, encoding="utf-8").read()
    out = set()
    for m in re.finditer(STEP_RE, s, re.S):
        u = re.search(r'aef:uid value="([^"]+)"', m.group(4))
        if u:
            out.add(u.group(1))
    return out

def versions(d):
    return sorted(int(m.group(1)) for f in os.listdir(d) if (m := re.fullmatch(r"v(\d+)\.bpmn", f)))

man_path = os.path.join(root, ".fabric", "workflow-adoption.yaml")
try:
    man = (yaml.safe_load(open(man_path)) or {}).get("workflows") or {}
except (OSError, yaml.YAMLError) as e:
    print(f"check-workflow-adoption: cannot read {man_path}: {e}", file=sys.stderr); sys.exit(2)
if not os.path.isdir(proj):
    print(f"check-workflow-adoption: no designer projects dir {proj}", file=sys.stderr); sys.exit(2)

cards = {}
for f in glob.glob(os.path.join(root, ".fabric", "components", "*.yaml")):
    try:
        c = yaml.safe_load(open(f))
    except yaml.YAMLError:
        continue
    if isinstance(c, dict) and c.get("type") == "workflow-step" and c.get("workflow"):
        cards.setdefault(c["workflow"], []).append(c)

fails, warns = [], []
folders = sorted(d for d in os.listdir(proj) if os.path.isdir(os.path.join(proj, d)))
for name in folders:
    if name not in man:
        fails.append(f"{name}: designer project not in .fabric/workflow-adoption.yaml (adopt it or record why not)")
for name, entry in sorted(man.items()):
    d = os.path.join(proj, name)
    if not os.path.isdir(d):
        fails.append(f"{name}: listed in the manifest but its designer project folder is gone"); continue
    status = (entry or {}).get("status")
    if status == "not-adopted":
        if not (entry or {}).get("reason"):
            fails.append(f"{name}: not-adopted needs a reason")
        continue
    if status != "adopted":
        fails.append(f"{name}: status must be adopted or not-adopted, got {status!r}"); continue
    v = entry.get("version")
    bpmn = os.path.join(d, f"v{v}.bpmn")
    if not os.path.isfile(bpmn):
        fails.append(f"{name}: adopted v{v} but {os.path.relpath(bpmn, root)} is gone"); continue
    want = steps(bpmn)
    mine = cards.get(name, [])
    have = {c.get("aef_uid") for c in mine}
    for miss in sorted(want - have):
        fails.append(f"{name}: step {miss} in v{v} has no card (run: bash scripts/fabric-register-workflow.sh {name})")
    for stale in sorted(have - want):
        fails.append(f"{name}: card for {stale} names a step that is not in v{v}")
    for c in mine:
        if not str(c.get("location", "")).endswith(f"/v{v}.bpmn"):
            fails.append(f"{name}: card {c.get('aef_uid')} points at {c.get('location')}, not the adopted v{v}")
    vs = versions(d)
    if vs and vs[-1] > v:
        warns.append(f"{name}: v{vs[-1]} exists, adopted v{v}")

# Step 4 of the ruling: the adopted diagrams are watched one exact version each, so their step cards sit
# inside coverage. The watch list must name exactly the adopted (workflow, version) pairs.
wp_path = os.path.join(root, ".fabric", "watch-patterns.yaml")
try:
    globs = [p.get("glob", "") for p in (yaml.safe_load(open(wp_path)) or {}).get("patterns", []) if isinstance(p, dict)]
except (OSError, yaml.YAMLError) as e:
    print(f"check-workflow-adoption: cannot read {wp_path}: {e}", file=sys.stderr); sys.exit(2)
watched = {(m.group(1), int(m.group(2))) for g in globs
           if (m := re.fullmatch(r"\.agentic-framework/\.context/designer/projects/([^/]+)/v(\d+)\.bpmn", g))}
adopted_pairs = {(n, e.get("version")) for n, e in man.items() if (e or {}).get("status") == "adopted"}
for n, v in sorted(adopted_pairs - watched):
    fails.append(f"{n}: adopted v{v} is not in watch-patterns.yaml (its step cards fall outside coverage)")
for n, v in sorted(watched - adopted_pairs):
    fails.append(f"{n}: watch-patterns.yaml watches v{v}, which the manifest does not adopt")

for w in warns:
    print(f"WARN {w}")
for f in fails:
    print(f"FAIL {f}")
adopted = sum(1 for e in man.values() if (e or {}).get("status") == "adopted")
print(f"check-workflow-adoption: {len(folders)} designer project(s), {adopted} adopted, "
      f"{len(fails)} failure(s), {len(warns)} warning(s)")
sys.exit(1 if fails else 0)
PY
