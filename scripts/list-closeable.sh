#!/usr/bin/env bash
# scripts/list-closeable.sh — surface agent-eligible closeable tasks (T-2207).
#
# Symmetric companion to `fw task verify` (which surfaces Human-AC pending).
# Answers the agent-side question: "Which agent-owned tasks have 0 unchecked
# Agent ACs and are ready to close right now?"
#
# Two classes:
#   - full-close-ready     Agent ✓ + Human ✓ both 0 unchecked → fully closeable
#   - partial-complete-ready  Agent ✓ but Human ✗ > 0 → close to partial-complete
#
# Usage:
#   bash scripts/list-closeable.sh [--json]
#   bash scripts/list-closeable.sh --help

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: list-closeable.sh [--json] [--help]

Walks .tasks/active/ and lists tasks where the agent can advance closure RIGHT NOW.

Filters applied (agent-eligible):
  owner=agent
  status in {captured, started-work, issues}
  workflow_type != inception
  horizon != later
  Agent unchecked = 0

Output sections:
  Full-close-ready          Human unchecked = 0   → fully closeable
  Partial-complete-ready    Human unchecked > 0   → close to partial-complete pending Human click

Flags:
  --json    Emit JSON envelope { ok, full_close_ready[], partial_complete_ready[] }
  --help    Show this message

Exit codes:
  0   ran successfully (sections may be empty — empty is healthy)
  2   tooling error (missing python3, malformed task tree)
EOF
}

WANT_JSON=0
for arg in "$@"; do
  case "$arg" in
    --help|-h) usage; exit 0 ;;
    --json) WANT_JSON=1 ;;
    *) printf 'list-closeable: unknown arg: %s\n' "$arg" >&2; usage >&2; exit 2 ;;
  esac
done

if ! command -v python3 >/dev/null 2>&1; then
  printf 'list-closeable: python3 not found on PATH\n' >&2
  exit 2
fi

TASK_DIR=".tasks/active"
if [ ! -d "$TASK_DIR" ]; then
  printf 'list-closeable: %s not found — run from project root\n' "$TASK_DIR" >&2
  exit 2
fi

WANT_JSON="$WANT_JSON" TASK_DIR="$TASK_DIR" python3 <<'PY'
import os, re, sys, json, glob

try:
    import yaml
except ImportError:
    sys.stderr.write("list-closeable: PyYAML not installed\n")
    sys.exit(2)

want_json = os.environ.get("WANT_JSON", "0") == "1"
task_dir = os.environ.get("TASK_DIR", ".tasks/active")

ALLOWED_STATUS = {"captured", "started-work", "issues"}

full = []
partial = []

for path in sorted(glob.glob(os.path.join(task_dir, "*.md"))):
    try:
        text = open(path).read()
    except OSError:
        continue
    m = re.match(r"---\n(.*?)\n---", text, re.S)
    if not m:
        continue
    try:
        fm = yaml.safe_load(m.group(1)) or {}
    except yaml.YAMLError:
        continue

    owner = fm.get("owner")
    status = fm.get("status")
    horizon = fm.get("horizon", "now")
    wf = fm.get("workflow_type")
    tid = fm.get("id", "?")
    name = (fm.get("name") or "").strip()

    if owner != "agent": continue
    if status not in ALLOWED_STATUS: continue
    if wf == "inception": continue
    if horizon == "later": continue

    agent_block = re.search(r"\n### Agent\n(.*?)(?=\n### |\n^## )", text, re.S | re.M)
    ac = agent_block.group(1) if agent_block else ""
    agent_unchecked = len(re.findall(r"^- \[ \]", ac, re.M))
    agent_checked = len(re.findall(r"^- \[x\]", ac, re.M))

    if agent_unchecked != 0:
        continue
    # No agent work outstanding — closeable in some form.

    human_block = re.search(r"\n### Human\n(.*?)(?=\n## )", text, re.S | re.M)
    hac = human_block.group(1) if human_block else ""
    # Strip HTML comments before counting (G-047 lesson: template examples
    # inside <!-- ... --> blocks were counted as real unchecked ACs).
    hac_stripped = re.sub(r"<!--.*?-->", "", hac, flags=re.S)
    human_unchecked = len(re.findall(r"^- \[ \]", hac_stripped, re.M))
    human_checked = len(re.findall(r"^- \[x\]", hac_stripped, re.M))

    entry = {
        "id": tid,
        "name": name[:80],
        "status": status,
        "horizon": horizon,
        "agent_checked": agent_checked,
        "human_unchecked": human_unchecked,
        "human_checked": human_checked,
    }
    if human_unchecked == 0:
        full.append(entry)
    else:
        partial.append(entry)

if want_json:
    json.dump(
        {
            "ok": True,
            "full_close_ready": full,
            "partial_complete_ready": partial,
            "summary": {
                "full_count": len(full),
                "partial_count": len(partial),
                "total": len(full) + len(partial),
            },
        },
        sys.stdout,
        indent=2,
    )
    sys.stdout.write("\n")
    sys.exit(0)

# Human-readable rendering.
def render(section, entries, suggested_cmd):
    print(f"\n{section} ({len(entries)})")
    if not entries:
        print("  (none)")
        return
    for e in entries:
        suffix = ""
        if e["human_unchecked"]:
            suffix = f"  Human: {e['human_checked']}✓ {e['human_unchecked']}✗"
        print(f"  {e['id']:8} a={e['agent_checked']}✓{suffix}  {e['name']}")
        print(f"           fw task update {e['id']} --status work-completed")

print(f"=== closeable tasks (T-2207) ===")
render("Full-close-ready (no Human ACs blocking)", full, "fw task update T-XXX --status work-completed")
render("Partial-complete-ready (Human ACs pending operator click)", partial, "fw task update T-XXX --status work-completed")

total = len(full) + len(partial)
print()
if total == 0:
    print("No agent-closeable tasks right now — backlog is drained or fully blocked on humans.")
    print("Next reads:")
    print("  fw task verify             # see human-AC-pending pile")
    print("  /peers                     # see who else is reachable to dispatch")
else:
    print(f"Total: {total} closeable ({len(full)} full + {len(partial)} partial)")
    print("Next step: pick one + run the suggested command, OR drive via substrate kit.")
PY
