#!/usr/bin/env bash
# canary-escalation-file.sh — T-2985 (value review C-19): turn a firing canary into a
# FILED TASK. Canaries detect; until now the response was one human reading /canaries,
# and canaries fired for days with no task naming them. Sibling of
# scripts/warn-escalation-file.sh (T-3267, the WARN-tier guard members) and built the
# same way, so the two cannot disagree about de-dup, CI or commit behaviour.
#
# Usage: scripts/canary-escalation-file.sh [--tasks-dir D] [--refile-hours N] [--dry-run]
#
# Rules:
#   FILE     a canary whose /canaries status is FIRING, ERRORING or STALE files ONE task.
#            Marker in the task body:  <!-- canary-escalation: canary=<name> -->
#   DE-DUP   never file while a task carrying that marker is OPEN (any file in
#            <tasks-dir>/active/, whatever its status, partial-complete included).
#            This is load-bearing: the canary cron runs daily and a naive filer would
#            open one task per run.
#   RE-FILE  after the task is CLOSED, the canary files again only when it is still bad
#            --refile-hours (CANARY_ESC_REFILE_HOURS, default 24) past the close, so a
#            close that fixed it has one canary cycle to show green.
#   IGNORED  HEALTHY, NO_HEARTBEAT and NOT_SCHEDULED (a canary that is not scheduled is
#            the cron-install-drift check's finding, T-2561, not this one's).
#
# It FILES and never FIXES: no canary is re-run, no log is truncated, no cron touched.
# Never from CI: under $CI it refuses (prints SKIP, exit 0). Filed task files are left
# UNCOMMITTED on purpose (SQ-22 option C: an unattended cron commit races live sessions'
# index, T-3231) and recorded in the pending-commit manifest (scripts/commit-pending.sh,
# T-3269), which the next session commits by name.
#
# Output (stdout), one line per decision, tab-separated:
#   FILED  <status> <canary> <T-ID>
#   WOULD  <status> <canary> -               (--dry-run)
#   OPEN   <status> <canary> <T-ID>          (already filed, still open)
#   WAIT   <status> <canary> <hours-left>    (closed recently; re-file window running)
#   SKIP   ci       -        -
# Exit: 0 decided (filed or nothing to file), 2 tooling (canary status unreadable, no
#       tasks dir, task create or body write failed). Never a silent 0 on "could not look".
#
# Seams (fixtures only): CANARY_ESC_STATUS_CMD (default: bash scripts/canary-status.sh
#   --json), CANARY_ESC_CREATE_CMD (default: .agentic-framework/bin/fw task create),
#   CANARY_ESC_PENDING_CMD (default: scripts/commit-pending.sh), CANARY_ESC_NOW (epoch).
set -u

TASKS_DIR=".tasks"
REFILE_HOURS="${CANARY_ESC_REFILE_HOURS:-24}"
DRY=0
STATUS_CMD="${CANARY_ESC_STATUS_CMD:-bash scripts/canary-status.sh --json}"
CREATE_CMD="${CANARY_ESC_CREATE_CMD:-.agentic-framework/bin/fw task create}"
NOW="${CANARY_ESC_NOW:-$(date +%s)}"

while [ $# -gt 0 ]; do
    case "$1" in
        --tasks-dir)    TASKS_DIR="${2:-}"; shift 2 ;;
        --refile-hours) REFILE_HOURS="${2:-}"; shift 2 ;;
        --dry-run)      DRY=1; shift ;;
        -h|--help)      sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "canary-escalation-file: unknown arg: $1" >&2; exit 2 ;;
    esac
done
for v in "$REFILE_HOURS" "$NOW"; do
    case "$v" in ''|*[!0-9]*) echo "canary-escalation-file: --refile-hours/now must be integers" >&2; exit 2 ;; esac
done

if [ -n "${CI:-}" ]; then
    printf 'SKIP\tci\t-\t-\n'
    echo "canary-escalation-file: \$CI is set — filing belongs to the host cron path, never CI" >&2
    exit 0
fi
[ -d "$TASKS_DIR/active" ] || { echo "canary-escalation-file: no $TASKS_DIR/active — no verdict" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "canary-escalation-file: python3 not found" >&2; exit 2; }

# canary-status exits 1 when anything needs attention — that is the input, not an error.
# Only output that does not parse is a tooling failure.
STATUS_JSON="$($STATUS_CMD 2>/dev/null)"
python3 -c 'import json,sys; json.loads(sys.argv[1])["canaries"]' "$STATUS_JSON" 2>/dev/null \
    || { echo "canary-escalation-file: cannot read canary status ($STATUS_CMD) — no verdict" >&2; exit 2; }

# ---- decide (pure: status + task corpus → plan lines) ------------------------
PLAN="$(python3 - "$STATUS_JSON" "$TASKS_DIR" "$NOW" "$REFILE_HOURS" <<'PYEOF'
import sys, os, re, glob, json
from datetime import datetime, timezone
status, tdir, now, hours = json.loads(sys.argv[1]), sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
win = hours * 3600
BAD = ("FIRING", "ERRORING", "STALE")
def ep(s):
    s = (s or "").strip().strip("'\"")
    if not s: return None
    try: dt = datetime.fromisoformat(s.replace("Z", "+00:00"))
    except Exception: return None
    if dt.tzinfo is None: dt = dt.replace(tzinfo=timezone.utc)
    return int(dt.timestamp())
MK = re.compile(r'<!--\s*canary-escalation:\s*canary=(\S+)\s*-->')
def tid(f):
    b = os.path.basename(f).split("-")
    return b[0] + "-" + b[1]
open_, closed = {}, {}
for f in glob.glob(os.path.join(tdir, "active", "*.md")):
    m = MK.search(open(f, errors="replace").read())
    if m: open_[m.group(1)] = tid(f)
for f in glob.glob(os.path.join(tdir, "completed", "*.md")):
    txt = open(f, errors="replace").read()
    m = MK.search(txt)
    if not m: continue
    fm = txt.split("\n---", 1)[0]
    d = re.search(r'^date_finished:\s*(.*)$', fm, re.M)
    t = ep(d.group(1)) if d else None
    if t is None: t = 0      # close time unknown -> never delays a re-file (loud side)
    closed[m.group(1)] = max(closed.get(m.group(1), 0), t)
for c in sorted(status.get("canaries", []), key=lambda x: x.get("name", "")):
    name, st = c.get("name", ""), c.get("status", "")
    if st not in BAD or not name: continue
    if name in open_: print("OPEN\t%s\t%s\t%s" % (st, name, open_[name])); continue
    if name in closed and now - closed[name] < win:
        print("WAIT\t%s\t%s\t%d" % (st, name, -(-(closed[name] + win - now) // 3600))); continue
    print("FILE\t%s\t%s\t-" % (st, name))
PYEOF
)" || { echo "canary-escalation-file: decision step failed" >&2; exit 2; }

canary_field() { # <canary> <field>
    python3 -c 'import json,sys
d={x.get("name"):x for x in json.loads(sys.argv[1]).get("canaries",[])}
v=d.get(sys.argv[2],{}).get(sys.argv[3],"")
print(v if v is not None else "")' "$STATUS_JSON" "$1" "$2" 2>/dev/null
}

record_pending() { # <T-ID> <task-file> <canary>
    local rel="${2#"$PWD"/}"
    case "$rel" in .tasks/*) ;; *) return 0 ;; esac
    "${CANARY_ESC_PENDING_CMD:-scripts/commit-pending.sh}" add "$1" "canary escalation filed ($3) by canary-escalation-file.sh" "$rel" \
        || echo "canary-escalation-file: $1 filed but NOT recorded in the pending-commit manifest — commit $rel by hand" >&2
}

write_body() { # <task-file> <context-text> <ac-text> — replace the template placeholders
    python3 - "$1" "$2" "$3" <<'PYEOF'
import sys, re
p, ctx, ac = sys.argv[1:4]
s = open(p).read()
s2, n1 = re.subn(r'(## Context\n)(.*?)(?=\n## Acceptance Criteria)', lambda m: m.group(1) + "\n" + ctx + "\n", s, count=1, flags=re.S)
s3, n2 = re.subn(r'- \[ \] \[First criterion\]\n(- \[ \] \[Second criterion\]\n)?', lambda m: ac + "\n", s2, count=1)
if n1 != 1 or n2 != 1: sys.exit(3)
open(p, "w").write(s3)
PYEOF
}

create_task() { # <name> <description> → prints "<T-ID>\t<file>" or fails
    local out id file
    out="$($CREATE_CMD --name "$1" --description "$2" --type build --owner agent --horizon now --tags "canary-escalation" 2>&1)" || {
        echo "canary-escalation-file: task create failed: $(tail -3 <<< "$out")" >&2; return 1; }
    id="$(sed -n 's/^ID:[[:space:]]*//p' <<< "$out" | head -1)"
    file="$(sed -n 's/^File:[[:space:]]*//p' <<< "$out" | head -1)"
    [ -n "$id" ] && [ -f "$file" ] || { echo "canary-escalation-file: task create gave no ID/File: $(tail -3 <<< "$out")" >&2; return 1; }
    printf '%s\t%s\n' "$id" "$file"
}

rc_all=0
while IFS=$'\t' read -r act st who extra; do
    [ -n "$act" ] || continue
    case "$act" in
        OPEN|WAIT) printf '%s\t%s\t%s\t%s\n' "$act" "$st" "$who" "$extra"; continue ;;
        FILE) ;;
        *) continue ;;
    esac
    if [ "$DRY" -eq 1 ]; then printf 'WOULD\t%s\t%s\t-\n' "$st" "$who"; continue; fi
    entry="$(canary_field "$who" latest_entry)"
    name="Canary $who is $st — investigate and fix"
    desc="The $who canary reports $st on /canaries. Filed automatically by scripts/canary-escalation-file.sh (T-2985, value review C-19) so a firing canary always has a task naming it."
    ctx="<!-- canary-escalation: canary=$who -->
Filed automatically by scripts/canary-escalation-file.sh (T-2985) from the daily canary cron.

- **Canary:** \`$who\` — status **$st** (\`bash scripts/canary-status.sh\`)
- **Findings log:** \`.context/working/.$who.log\` (tooling errors: \`.log.stderr\`)
- **What each status means:** FIRING = the watched thing is broken; ERRORING = the canary could not run; STALE = its cron has stopped completing. Per-canary operator action: \`docs/operations/guard-layer-and-canaries.md\`.

**Latest signal line:**

\`\`\`
${entry:-(no entry recorded)}
\`\`\`

Closing this task while the canary is still bad files a new one ${REFILE_HOURS}h after the close."
    ac="- [ ] \`$who\` reports HEALTHY on \`bash scripts/canary-status.sh\` (cause fixed, or the canary corrected if the finding was false)"
    created="$(create_task "$name" "$desc")" || { rc_all=2; continue; }
    tid="${created%%	*}"; tfile="${created#*	}"
    if ! write_body "$tfile" "$ctx" "$ac"; then
        echo "canary-escalation-file: $tid created at $tfile but its body could not be written — edit it by hand (the de-dup marker is missing, so it will be re-filed)" >&2
        rc_all=2; continue
    fi
    record_pending "$tid" "$tfile" "$who"
    printf 'FILED\t%s\t%s\t%s\n' "$st" "$who" "$tid"
done <<< "$PLAN"
exit "$rc_all"
