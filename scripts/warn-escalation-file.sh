#!/usr/bin/env bash
# warn-escalation-file.sh — T-3267: turn a stale or accumulating WARN guard into a
# FILED TASK (the operator's binding condition on the WARN tier, T-3258 rulings:
# "after the 14-day limit it becomes an action. We don't want things to go stale or
# debt to build up."). T-3260 only REPORTED the age; a canary line is not an action.
#
# Usage: scripts/warn-escalation-file.sh [--ledger P] [--tasks-dir D] [--days N]
#                                        [--max-red N] [--dry-run]
#
# Rules:
#   STALE        a member in the WARN ledger red > --days (GUARD_WARN_ESCALATE_DAYS,
#                default 14) files ONE task for that member. Marker in the task body:
#                  <!-- warn-escalation: member=<member> -->
#   ACCUMULATION more than --max-red (GUARD_WARN_MAX_RED, default 5) members red at
#                once files ONE umbrella task naming all of them, immediately,
#                regardless of age. Marker: <!-- warn-escalation: umbrella -->
#   DE-DUP       never file while a task carrying the marker is OPEN — any file in
#                <tasks-dir>/active/, whatever its status (partial-complete included).
#   RE-FILE      after the task is CLOSED (moved to completed/), the clock restarts
#                at its date_finished: a still-red member files again only when it has
#                been red > --days past max(first-red, last close). The umbrella uses
#                the same window after its close. A member that went green drops out
#                of the ledger, so its next red starts a fresh first-red anyway.
#
# It FILES and never FIXES: no member is re-run to heal, no tier is changed, FAIL
# members are never read (only the WARN ledger is). It only runs a member once, to
# quote its current output into the task it files.
#
# Never from CI: under $CI it refuses (prints SKIP, exit 0). Filing belongs to the
# daily host cron path (check-release-publication-freshness.sh step c). The filed
# files are left UNCOMMITTED on purpose (SQ-22 proposal — an unattended cron commit
# races live sessions' index, the T-3231 lesson); they are loud instead: the canary
# names them daily and they appear in every handover's active-task list.
#
# Output (stdout), one line per decision, tab-separated:
#   FILED  <stale|umbrella> <member|umbrella> <T-ID>
#   WOULD  <stale|umbrella> <member|umbrella> -          (--dry-run)
#   OPEN   <stale|umbrella> <member|umbrella> <T-ID>     (already filed, still open)
#   WAIT   <stale|umbrella> <member|umbrella> <days-left> (closed; new window running)
#   SKIP   ci|not-warn      <member|->         -
# Exit 0 = decisions made (filed or not) · 2 = tooling (fail-closed: a ledger that
# cannot be parsed, a create that fails, a body that cannot be written).
#
# Test seams (PL-213): WARN_ESC_NOW (epoch), WARN_ESC_TASKS_DIR, WARN_ESC_CREATE_CMD (replaces
# `.agentic-framework/bin/fw task create`), WARN_ESC_RUNNER (replaces
# scripts/run-guard-layer.sh; GUARD_LAYER_SCRIPTS_DIR/TESTS_DIR pass through to it).
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || (cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd))" || exit 2

LEDGER="${GUARD_WARN_LEDGER:-.context/checks/guard-warn-first-red}"
TASKS_DIR="${WARN_ESC_TASKS_DIR:-.tasks}"
DAYS="${GUARD_WARN_ESCALATE_DAYS:-14}"
MAX_RED="${GUARD_WARN_MAX_RED:-5}"
DRY=0
NOW="${WARN_ESC_NOW:-$(date -u +%s)}"
CREATE_CMD="${WARN_ESC_CREATE_CMD:-.agentic-framework/bin/fw task create}"
RUNNER="${WARN_ESC_RUNNER:-scripts/run-guard-layer.sh}"

while [ $# -gt 0 ]; do
    case "$1" in
        --ledger)    LEDGER="${2:-}"; shift 2 ;;
        --tasks-dir) TASKS_DIR="${2:-}"; shift 2 ;;
        --days)      DAYS="${2:-}"; shift 2 ;;
        --max-red)   MAX_RED="${2:-}"; shift 2 ;;
        --dry-run)   DRY=1; shift ;;
        -h|--help)   sed -n '2,45p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "warn-escalation-file: unknown arg: $1" >&2; exit 2 ;;
    esac
done
for v in "$DAYS" "$MAX_RED" "$NOW"; do
    case "$v" in ''|*[!0-9]*) echo "warn-escalation-file: --days/--max-red/now must be integers" >&2; exit 2 ;; esac
done

if [ -n "${CI:-}" ]; then
    printf 'SKIP\tci\t-\t-\n'
    echo "warn-escalation-file: \$CI is set — filing belongs to the host cron path, never CI" >&2
    exit 0
fi
[ -f "$LEDGER" ] || exit 0                       # nothing recorded red → nothing to file
[ -d "$TASKS_DIR/active" ] || { echo "warn-escalation-file: no $TASKS_DIR/active — no verdict" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "warn-escalation-file: python3 not found" >&2; exit 2; }

# The runner's member list is the tier source of truth (scope fence: a member that is
# not WARN is never filed on). Unreadable ⇒ fail closed, never "file without checking".
META="$(bash "$RUNNER" --list --json 2>/dev/null)" \
    && python3 -c 'import json,sys; json.loads(sys.argv[1])["members"]' "$META" 2>/dev/null \
    || { echo "warn-escalation-file: cannot read the guard-layer member list ($RUNNER --list --json) — no verdict" >&2; exit 2; }

# ---- decide (pure: ledger + task corpus + meta → plan lines) -----------------
PLAN="$(python3 - "$LEDGER" "$TASKS_DIR" "$NOW" "$DAYS" "$MAX_RED" "$META" <<'PYEOF'
import sys, os, re, glob, json
from datetime import datetime, timezone
ledger, tdir, now, days, max_red, meta = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5]), sys.argv[6]
win = days * 86400
def ep(s):
    s = (s or "").strip().strip("'\"")
    if not s: return None
    try: dt = datetime.fromisoformat(s.replace("Z", "+00:00"))
    except Exception: return None
    if dt.tzinfo is None: dt = dt.replace(tzinfo=timezone.utc)
    return int(dt.timestamp())
red = {}
for ln in open(ledger):
    if re.match(r'\s*#', ln) or len(ln.split()) < 2: continue
    m, ts = ln.split()[:2]
    t = ep(ts)
    if t is None: print("BAD\t%s\t%s" % (m, ts)); sys.exit(0)
    red[m] = t
members = {x["name"]: x for x in json.loads(meta)["members"]}
MK = re.compile(r'<!--\s*warn-escalation:\s*(member=(\S+)|umbrella)\s*-->')
def key_of(txt):
    m = MK.search(txt)
    if not m: return None
    return "umbrella" if m.group(1) == "umbrella" else m.group(2)
open_, closed = {}, {}
for f in glob.glob(os.path.join(tdir, "active", "*.md")):
    k = key_of(open(f, errors="replace").read())
    if k: open_[k] = os.path.basename(f).split("-")[0] + "-" + os.path.basename(f).split("-")[1]
for f in glob.glob(os.path.join(tdir, "completed", "*.md")):
    txt = open(f, errors="replace").read()
    k = key_of(txt)
    if not k: continue
    fm = txt.split("\n---", 1)[0]
    d = re.search(r'^date_finished:\s*(.*)$', fm, re.M)
    t = ep(d.group(1)) if d else None
    if t is None:
        lu = re.search(r'^last_update:\s*(.*)$', fm, re.M)
        t = ep(lu.group(1)) if lu else None
    if t is None: t = 0          # close time unknown → never delays a re-file (loud side)
    closed[k] = max(closed.get(k, 0), t)
for m in sorted(red):
    info = members.get(m)
    if info is not None and info.get("class") != "warn":
        print("SKIP\tnot-warn\t%s\t-" % m); continue
    start = max(red[m], closed.get(m, 0))
    if m in open_: print("OPEN\tstale\t%s\t%s" % (m, open_[m])); continue
    if now - start > win: print("FILE\tstale\t%s\t%d" % (m, (now - red[m]) // 86400))
    elif m in closed and now - red[m] > win: print("WAIT\tstale\t%s\t%d" % (m, -(-(start + win - now) // 86400)))
if len(red) > max_red:
    k = "umbrella"
    if k in open_: print("OPEN\tumbrella\tumbrella\t%s" % open_[k])
    elif k in closed and now - closed[k] <= win: print("WAIT\tumbrella\tumbrella\t%d" % (-(-(closed[k] + win - now) // 86400)))
    else: print("FILE\tumbrella\tumbrella\t%d" % len(red))
PYEOF
)" || { echo "warn-escalation-file: decision step failed" >&2; exit 2; }

if grep -q '^BAD' <<< "$PLAN"; then
    echo "warn-escalation-file: unparseable first-red time in $LEDGER: $(grep '^BAD' <<< "$PLAN" | head -1 | cut -f2-)" >&2
    exit 2
fi

# member_field <member> <field> — from the runner's --list --json
member_field() {
    python3 -c 'import json,sys
d={x["name"]:x for x in json.loads(sys.argv[1]).get("members",[])}
print(d.get(sys.argv[2],{}).get(sys.argv[3],""))' "$META" "$1" "$2" 2>/dev/null
}
first_red() { awk -v m="$1" '!/^[[:space:]]*#/ && $1==m {print $2; exit}' "$LEDGER"; }

# write_body <task-file> <context-text> <ac-text> — replace the template placeholders
write_body() {
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

# create_task <name> <description> → prints "<T-ID>\t<file>" or fails
create_task() {
    local out id file
    out="$($CREATE_CMD --name "$1" --description "$2" --type build --owner agent --horizon now --tags "warn-escalation,guard-layer" 2>&1)" || {
        echo "warn-escalation-file: task create failed: $(tail -3 <<< "$out")" >&2; return 1; }
    id="$(sed -n 's/^ID:[[:space:]]*//p' <<< "$out" | head -1)"
    file="$(sed -n 's/^File:[[:space:]]*//p' <<< "$out" | head -1)"
    [ -n "$id" ] && [ -f "$file" ] || { echo "warn-escalation-file: task create gave no ID/File: $(tail -3 <<< "$out")" >&2; return 1; }
    printf '%s\t%s\n' "$id" "$file"
}

member_output() { # <member> → last 40 lines of its current run, with rc
    local cmd out rc
    cmd="$(member_field "$1" cmd)"
    [ -n "$cmd" ] || { echo "(member not found in the guard layer — renamed, removed or reclassified? not run)"; return; }
    out="$(timeout 120 bash -c "$cmd" 2>&1)"; rc=$?
    printf '$ %s   # rc=%s\n%s\n' "$cmd" "$rc" "$(tail -40 <<< "$out")"
}

rc_all=0
while IFS=$'\t' read -r act kind who extra; do
    [ -n "$act" ] || continue
    case "$act" in
        OPEN|WAIT|SKIP) printf '%s\t%s\t%s\t%s\n' "$act" "$kind" "$who" "$extra"; continue ;;
        FILE) ;;
        *) continue ;;
    esac
    if [ "$DRY" -eq 1 ]; then printf 'WOULD\t%s\t%s\t-\n' "$kind" "$who"; continue; fi
    if [ "$kind" = stale ]; then
        fr="$(first_red "$who")"; reason="$(member_field "$who" reason)"
        name="WARN escalation: $who red ${extra}d (since $fr) — fix or reclassify"
        desc="WARN-tier guard member $who has been red for more than ${DAYS} days (first seen red $fr). Operator binding condition (T-3258 rulings): a stale WARN becomes an action."
        ctx="<!-- warn-escalation: member=$who -->
Filed automatically by scripts/warn-escalation-file.sh (T-3267) from the host release canary.

- **Member:** \`$who\` (tier WARN — never gates CI or a release)
- **First seen red:** $fr (${extra}d ago; escalation threshold ${DAYS}d)
- **Tier reason (operator classification):** ${reason:-(no reason recorded)}

**Current output:**

\`\`\`
$(member_output "$who")
\`\`\`

Do not hand-edit the ledger date (.context/checks/guard-warn-first-red) to silence this. Closing this task
while the member is still red restarts a ${DAYS}-day window from the close date, then files again."
        ac="- [ ] \`$who\` is green, or the operator has reclassified it (tier change recorded in docs/reports/T-3258-guard-classification-draft.md)"
    else
        members_list="$(awk '!/^[[:space:]]*#/ && NF>=2 {printf "- `%s` red since %s\n", $1, $2}' "$LEDGER")"
        name="WARN accumulation: $extra WARN guard members red at once (> $MAX_RED) — pay down"
        desc="$extra WARN-tier guard members are red at the same time, above the accumulation limit of $MAX_RED. Operator binding condition (T-3258 rulings): WARN debt must not build up."
        ctx="<!-- warn-escalation: umbrella -->
Filed automatically by scripts/warn-escalation-file.sh (T-3267) from the host release canary.

$extra WARN-tier members are red at once (limit $MAX_RED). Each is also escalated on its own once it
passes ${DAYS} days. Red now (from .context/checks/guard-warn-first-red):

$members_list"
        ac="- [ ] At most $MAX_RED WARN members are red at once (each fixed, or reclassified by the operator)"
    fi
    created="$(create_task "$name" "$desc")" || { rc_all=2; continue; }
    tid="${created%%	*}"; tfile="${created#*	}"
    if ! write_body "$tfile" "$ctx" "$ac"; then
        echo "warn-escalation-file: $tid created at $tfile but its body could not be written — edit it by hand (the de-dup marker is missing, so it will be re-filed)" >&2
        rc_all=2; continue
    fi
    printf 'FILED\t%s\t%s\t%s\n' "$kind" "$who" "$tid"
done <<< "$PLAN"
exit "$rc_all"
