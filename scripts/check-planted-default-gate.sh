#!/usr/bin/env bash
# guard-layer: source
# check-planted-default-gate.sh (T-2855, G-019 prevention for the class 832 named)
#
# ORIGIN: 832-Workflow-designer, agent-chat-arc @769. They found their BVP ranker
# returning THIRTEEN active tasks at a byte-identical score, traced it to a template
# that pre-fills the one field the score is derived from, and asked us to sweep:
# "any field that is BOTH required by a gate AND pre-filled by a template is exposed,
# and we had never audited ours."
#
# THE CLASS
#
#   A PRE-FILLED REQUIRED FIELD CONVERTS A GATE FOR PRESENCE INTO A GATE THAT IS
#   STRUCTURALLY INCAPABLE OF FAILING — it can only ever confirm the default it planted.
#
# Every link behaves as designed. The template supplies the field (which is PL-167,
# and PL-167 is correct). The schema gate requires it present and in range; the planted
# value passes. Something downstream treats it as information. Nothing is red anywhere
# and the output has no information in it.
#
# It reproduced here in full on first look: `.tasks/templates/inception.md` plants
# `voi_score: 0.5`, `check-inception-schema.py` requires it, and
# `estimator.py::_score_inception_voi` makes that ONE field the composite for all nine
# drivers. `int(round(0.5*5))` is 2 under banker's rounding — the same value the
# `voi is None` grandfathered branch returns — so 202 of 203 inceptions score 2 on every
# driver and every active one ranks identically.
#
# WHAT IT DOES
#   1. PLANTED  — frontmatter fields in .tasks/templates/*.md carrying a literal scalar
#                 default (placeholders, empty values, [] and null are not defaults).
#   2. REQUIRED — field names a gate script asserts on, anchored to a line that also
#                 says "missing" or "required". Narrow on purpose: precision over
#                 recall, the same trade the sibling static checks make.
#   3. EXPOSED  — the intersection. Not yet a finding.
#   4. FIRING   — an exposed field whose planted default DOMINATES the corpus: of the
#                 tasks that carry the field at all, >= THRESHOLD (default 0.90) hold
#                 the template's exact value, over a population of >= MIN_POP (10).
#
# WHY DOMINATION AND NOT MERE EXPOSURE. `horizon: now` and `status: captured` are
# planted and gate-checked too, and a dominant default is CORRECT for both — `now` is
# the right horizon for most tasks and `captured` is where a lifecycle starts. Firing on
# exposure alone would make this permanently red, which is how an operator learns to stop
# reading a guard (the T-2818 / T-2833 fatigue lesson, from the other direction).
# Domination measures the actual harm: how often the gate is confirming its own default.
#
# SCOPE, STATED PLAINLY (T-2680). This detects a field whose template default dominates
# the corpus. It does NOT judge whether a default is semantically appropriate, and it
# cannot see a field that is information-bearing but happens to be spread. A clean exit
# means no ledgered-or-unledgered field crossed the domination threshold — never that
# every gate in the tree can fail.
#
# Exit: 0 clean | 1 unacknowledged dominated field | 2 tooling (fail-closed)
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 2

TEMPLATES="${PLANTED_TEMPLATES_DIR:-.tasks/templates}"
GATES="${PLANTED_GATES_DIR:-.agentic-framework/agents/context}"
TASKS="${PLANTED_TASKS_DIR:-.tasks}"
ALLOW="${PLANTED_ALLOWLIST:-.context/checks/planted-default-gate-allowlist}"
THRESHOLD="${PLANTED_THRESHOLD:-0.90}"
MIN_POP="${PLANTED_MIN_POP:-10}"
JSON=0; QUIET=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --quiet) QUIET=1 ;;
    --templates) TEMPLATES="$2"; shift ;;
    --gates) GATES="$2"; shift ;;
    --tasks-dir) TASKS="$2"; shift ;;
    --allowlist) ALLOW="$2"; shift ;;
    --threshold) THRESHOLD="$2"; shift ;;
    --min-pop) MIN_POP="$2"; shift ;;
    *) echo "check-planted-default-gate: unknown flag: $1" >&2; exit 2 ;;
  esac
  shift
done

command -v python3 >/dev/null 2>&1 || {
  echo "check-planted-default-gate: python3 not found — cannot parse frontmatter" >&2
  echo "  (exiting 2, not 0: a checker that reports clean because it failed to load" >&2
  echo "   converts an unknown into a false assurance)" >&2
  exit 2
}

TEMPLATES="$TEMPLATES" GATES="$GATES" TASKS="$TASKS" ALLOW="$ALLOW" \
THRESHOLD="$THRESHOLD" MIN_POP="$MIN_POP" JSON="$JSON" QUIET="$QUIET" \
python3 - <<'PY'
import os, re, sys, json, glob

TEMPLATES = os.environ["TEMPLATES"]
GATES     = os.environ["GATES"]
TASKS     = os.environ["TASKS"]
ALLOW     = os.environ["ALLOW"]
THRESHOLD = float(os.environ["THRESHOLD"])
MIN_POP   = int(os.environ["MIN_POP"])
JSON      = os.environ["JSON"] == "1"
QUIET     = os.environ["QUIET"] == "1"

SCOPE = ("detects a field whose TEMPLATE DEFAULT DOMINATES the corpus; it does not "
         "judge whether a default is semantically appropriate, nor see an "
         "information-bearing field that happens to be spread.")

def die(msg):
    if JSON:
        print(json.dumps({"ok": False, "error": msg, "scope": SCOPE}))
    else:
        print(f"check-planted-default-gate: {msg}", file=sys.stderr)
    sys.exit(2)

def frontmatter(path):
    try:
        s = open(path, encoding="utf8", errors="replace").read()
    except OSError:
        return None
    m = re.match(r"^---\n(.*?)\n---", s, re.S)
    return m.group(1) if m else None

# A default is a LITERAL SCALAR. These are not defaults:
#   empty (author must fill), [] / {} (empty collection), null, block scalars (>, |),
#   and {PLACEHOLDER} / T-XXX style stand-ins that no real task retains.
def literal_default(val):
    v = val.strip()
    if not v or v in ("[]", "{}", "null", "~", ">", "|", ">-", "|-"):
        return None
    if v.startswith("{") or v == "T-XXX":
        return None
    return v

# ---------- 1. PLANTED ----------
tpl_files = sorted(glob.glob(os.path.join(TEMPLATES, "*.md")))
if not tpl_files:
    die(f"no templates found under {TEMPLATES} — refusing to report clean")

planted = {}   # field -> {value -> [template, ...]}
for t in tpl_files:
    fm = frontmatter(t)
    if fm is None:
        continue
    for line in fm.split("\n"):
        if line.startswith("#") or line.startswith(" "):
            continue
        m = re.match(r"^([a-z_][a-z0-9_]*):(.*)$", line)
        if not m:
            continue
        name, raw = m.group(1), m.group(2)
        raw = re.sub(r"\s+#.*$", "", raw)          # strip trailing comment
        d = literal_default(raw)
        if d is None:
            continue
        planted.setdefault(name, {}).setdefault(d, []).append(os.path.basename(t))

# ---------- 2. REQUIRED ----------
# Narrow anchor: the field name must appear on a gate line that ALSO says
# missing/required. A gate that merely reads a field does not require it.
gate_files = []
for pat in ("*.py", "*.sh"):
    gate_files += glob.glob(os.path.join(GATES, pat))
if not gate_files:
    die(f"no gate scripts found under {GATES} — refusing to report clean")

required = {}   # field -> [gatefile, ...]
for g in gate_files:
    try:
        lines = open(g, encoding="utf8", errors="replace").read().split("\n")
    except OSError:
        continue
    for line in lines:
        low = line.lower()
        if "missing" not in low and "required" not in low:
            continue
        for name in planted:
            if re.search(r"\b" + re.escape(name) + r"\b", line):
                required.setdefault(name, [])
                if os.path.basename(g) not in required[name]:
                    required[name].append(os.path.basename(g))

exposed = sorted(set(planted) & set(required))

# ---------- 3. CORPUS DOMINATION ----------
task_files = [f for f in glob.glob(os.path.join(TASKS, "active", "*.md"))
                        + glob.glob(os.path.join(TASKS, "completed", "*.md"))]
if not task_files:
    die(f"no task files under {TASKS}/active or {TASKS}/completed — refusing to report clean")

def field_of(fm, name):
    m = re.search(r"^%s:\s*([^\n#]*)" % re.escape(name), fm, re.M)
    return m.group(1).strip() if m else None

corpus = []
for f in task_files:
    fm = frontmatter(f)
    if fm is not None:
        corpus.append(fm)

# ---------- 4. ALLOWLIST ----------
acknowledged = {}
if os.path.exists(ALLOW):
    try:
        for line in open(ALLOW, encoding="utf8"):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            name, _, reason = line.partition("#")
            acknowledged[name.strip()] = reason.strip()
    except OSError:
        die(f"allowlist {ALLOW} exists but could not be read")

firing, acked, spared = [], [], []
for name in exposed:
    values = planted[name]
    if len(values) != 1:
        # the templates disagree on the default; not the shape this detects
        spared.append({"field": name, "why": "templates plant differing defaults"})
        continue
    default = next(iter(values))
    carriers = [field_of(fm, name) for fm in corpus]
    carriers = [c for c in carriers if c is not None and c != ""]
    pop = len(carriers)
    if pop < MIN_POP:
        spared.append({"field": name, "why": f"population {pop} < min {MIN_POP}"})
        continue
    same = sum(1 for c in carriers if c == default)
    ratio = same / pop
    rec = {"field": name, "default": default, "population": pop,
           "at_default": same, "ratio": round(ratio, 4),
           "templates": values[default], "gates": required[name]}
    if ratio < THRESHOLD:
        spared.append({"field": name, "why": f"ratio {ratio:.2f} < threshold {THRESHOLD:.2f}"})
    elif name in acknowledged:
        rec["reason"] = acknowledged[name]
        acked.append(rec)
    else:
        firing.append(rec)

if JSON:
    print(json.dumps({
        "ok": not firing, "scope": SCOPE, "threshold": THRESHOLD, "min_population": MIN_POP,
        "planted_fields": len(planted), "gate_required_fields": len(required),
        "exposed": exposed, "firing": firing, "acknowledged": acked,
        "acknowledged_count": len(acked), "spared": spared,
        "templates_scanned": len(tpl_files), "tasks_scanned": len(corpus),
    }, indent=2))
    sys.exit(1 if firing else 0)

if not firing:
    if not QUIET:
        print(f"check-planted-default-gate: clean — 0 unacknowledged dominated field(s) "
              f"({len(exposed)} exposed, {len(acked)} acknowledged, "
              f"{len(tpl_files)} template(s), {len(corpus)} task(s) scanned).")
        print(f"  SCOPE: {SCOPE}")
        for a in acked:
            print(f"    acknowledged: {a['field']}={a['default']} "
                  f"({a['at_default']}/{a['population']}, {a['ratio']*100:.1f}%) — {a['reason']}")
    sys.exit(0)

print("check-planted-default-gate: template-planted field(s) whose default dominates the corpus")
print(f"  SCOPE: {SCOPE}")
print("")
print(f"  {len(firing)} field(s) both PLANTED by a template and REQUIRED by a gate, whose")
print("  planted value dominates. The gate can only confirm the default it supplied:")
print("")
for r in firing:
    print(f"    {r['field']}: {r['at_default']}/{r['population']} tasks "
          f"({r['ratio']*100:.1f}%) hold the planted value {r['default']!r}")
    print(f"        planted by: {', '.join(r['templates'])}")
    print(f"        required by: {', '.join(r['gates'])}")
print("")
print("  Fix: stop planting the value (but see PL-167 — a gate that demands a shape needs")
print("       the template to supply it, so removing it outright re-breaks creation), or")
print("       give the field a proposed-lane an agent may legitimately write, or — if the")
print("       dominant default is genuinely correct — acknowledge it in")
print(f"       {ALLOW} with a cited reason.")
sys.exit(1)
PY
