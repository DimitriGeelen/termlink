#!/usr/bin/env bash
# T-3310 (arc-012 step 5, operator ruling D1 = D+, layers 2 and 3) — the
# "forever needs an owner" backstop canary.
#
# D1: a forever create with no owner and no reason is accepted and labelled
# while old clients upgrade; the hub starts REFUSING it by itself once 14 days
# pass with no such create (auto mode). The operator called the risk out
# explicitly: a switch that waits for something can wait forever. This canary is
# what stops that from happening silently. It FIRES (exit 1) when:
#   (a) the local hub runs with forever_requires_owner=never (an opt-out stays
#       visible every day it is set), or
#   (b) today is on or after the backstop date (default 2026-11-15) and the hub
#       is still not enforcing — and names who is still sending bare forever.
# On firing it FILES A TASK (the T-3267 "escalation is an action" rule), once:
# de-duplicated by a body marker against every open .tasks/active/ file, and
# recorded in the pending-commit manifest (T-3269) for the next session to commit.
#
# Exit 0 healthy · 1 firing · 2 tooling (hub down, unparseable status, or a hub
# too old to report the T-3310 fields — never a false "healthy").
#
# Usage: bash scripts/check-forever-owner-freshness.sh [--json] [--quiet]
#        [--no-heartbeat] [--no-file-task] [--dry-run] [--backstop YYYY-MM-DD]
# Seams (PL-213): FOREVER_OWNER_TEST_JSON=<file> (canned `hub status --governor
# --json`), FOREVER_OWNER_TEST_NOW=<epoch secs>, FOREVER_OWNER_TASKS_DIR=<dir>,
# FOREVER_OWNER_CREATE_CMD, FOREVER_OWNER_PENDING_CMD, FOREVER_OWNER_HEARTBEAT_FILE.
# Runtime canary, not a guard-layer member (it reads the live hub). Fixtures: tests/forever-owner-canary-fixtures.sh
set -uo pipefail

JSON=0 QUIET=0 HEARTBEAT=1 FILE_TASK=1 DRY=0
BACKSTOP="${FOREVER_OWNER_BACKSTOP:-2026-11-15}"
while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1 ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --no-file-task) FILE_TASK=0 ;;
        --dry-run) DRY=1 ;;
        --backstop) BACKSTOP="${2:-}"; shift ;;
        -h|--help) sed -n '2,32p' "$0"; exit 0 ;;
        *) echo "check-forever-owner: unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

HB="${FOREVER_OWNER_HEARTBEAT_FILE:-.context/working/.forever-owner-canary.heartbeat}"
# T-2843: the heartbeat proves the run COMPLETED, so it is written on exit.
on_exit() { [ "$HEARTBEAT" -eq 1 ] && { mkdir -p "$(dirname "$HB")" 2>/dev/null; date -u +%FT%TZ > "$HB" 2>/dev/null; }; }
trap on_exit EXIT

if [ -n "${FOREVER_OWNER_TEST_JSON:-}" ]; then
    status="$(cat "$FOREVER_OWNER_TEST_JSON")" || { echo "check-forever-owner: cannot read test JSON" >&2; exit 2; }
else
    status="$(timeout 15 termlink hub status --governor --json 2>&1)" \
        || { echo "check-forever-owner: hub status failed: $(tail -2 <<< "$status")" >&2; exit 2; }
fi
now="${FOREVER_OWNER_TEST_NOW:-$(date +%s)}"
backstop_epoch="$(date -u -d "$BACKSTOP" +%s 2>/dev/null)" \
    || { echo "check-forever-owner: bad --backstop date: $BACKSTOP" >&2; exit 2; }

verdict="$(STATUS_JSON="$status" python3 - "$now" "$backstop_epoch" "$BACKSTOP" <<'PY'
import json, sys, datetime, os
now, backstop, backstop_s = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
try:
    d = json.loads(os.environ["STATUS_JSON"])
except Exception as e:
    print(json.dumps({"rc": 2, "error": f"unparseable hub status: {e}"})); sys.exit()
g = d.get("governor") if isinstance(d, dict) else None
if not isinstance(g, dict):
    print(json.dumps({"rc": 2, "error": "no governor block (hub not running, or status without --governor)"})); sys.exit()
if "forever_requires_owner" not in g or "forever_enforced" not in g:
    print(json.dumps({"rc": 2, "error": "hub predates T-3310 (no forever_requires_owner / forever_enforced) — upgrade it"})); sys.exit()
mode, enforced = g["forever_requires_owner"], bool(g["forever_enforced"])
senders = g.get("bare_forever_senders") or []
def ts(ms):
    return datetime.datetime.fromtimestamp(ms / 1000, datetime.timezone.utc).strftime("%Y-%m-%d %H:%M") if ms else "-"
names = [f"{s.get('sender')} ({s.get('creates')} create(s), last {ts(s.get('last_ms'))})" for s in senders[:10]]
firing = []
if mode == "never":
    firing.append("opt-out: forever_requires_owner=never — bare forever topics are accepted indefinitely")
if now >= backstop and not enforced:
    firing.append(f"backstop {backstop_s} passed and the hub is still not enforcing (mode {mode})")
out = {
    "rc": 1 if firing else 0, "mode": mode, "enforced": enforced,
    "enforce_at": ts(g.get("forever_enforce_at_ms")), "quiet_since": ts(g.get("forever_quiet_since_ms")),
    "backstop": backstop_s, "firing": firing, "senders": names,
}
print(json.dumps(out))
PY
)"
rc="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["rc"])' <<< "$verdict" 2>/dev/null)" || rc=2
if [ "$rc" = 2 ]; then
    python3 -c 'import json,sys; print("check-forever-owner:", json.load(sys.stdin).get("error","tooling error"))' <<< "$verdict" >&2 2>/dev/null \
        || echo "check-forever-owner: tooling error" >&2
    exit 2
fi

filed=""
if [ "$rc" = 1 ] && [ "$FILE_TASK" -eq 1 ]; then
    if [ -n "${CI:-}" ]; then
        filed="skip (CI)"
    else
        tasks_dir="${FOREVER_OWNER_TASKS_DIR:-.tasks/active}"
        marker="<!-- forever-owner-backstop -->"
        if grep -rlF -- "$marker" "$tasks_dir" >/dev/null 2>&1; then
            filed="open task already exists ($(grep -rlF -- "$marker" "$tasks_dir" | head -1 | xargs -n1 basename | cut -c1-6))"
        elif [ "$DRY" -eq 1 ]; then
            filed="WOULD file a task"
        else
            create="${FOREVER_OWNER_CREATE_CMD:-.agentic-framework/bin/fw task create}"
            out="$($create --name "Forever-owner enforcement not on: bare forever topics still accepted (T-3310 backstop)" \
                --description "Filed by scripts/check-forever-owner-freshness.sh (T-3310 D1 backstop). The hub accepts forever topics with no owner and no reason. Upgrade the named senders, or record an operator decision to keep the opt-out." \
                --type build --owner agent --horizon now --tags "arc:arc-012,forever-owner-backstop" 2>&1)"
            id="$(sed -n 's/^ID:[[:space:]]*//p' <<< "$out" | head -1)"
            file="$(sed -n 's/^File:[[:space:]]*//p' <<< "$out" | head -1)"
            if [ -z "$id" ] || [ ! -f "$file" ]; then
                filed="FAILED to file: $(tail -2 <<< "$out" | tr '\n' ' ')"
            else
                ctx="$marker
Why: $(python3 -c 'import json,sys; print("; ".join(json.load(sys.stdin)["firing"]))' <<< "$verdict")
Still sending bare forever: $(python3 -c 'import json,sys; s=json.load(sys.stdin)["senders"]; print("; ".join(s) or "none recorded")' <<< "$verdict")
Ruling: T-3310 § Decisions D1 (D+). Clock: 14 days with no bare-forever create flips enforcement on."
                ac="- [ ] The hub reports forever_enforced=true, or the operator recorded a decision to keep forever_requires_owner=never"
                python3 - "$file" "$ctx" "$ac" <<'PYEOF' || filed="filed $id but could not write its body"
import sys, re
p, ctx, ac = sys.argv[1:4]
s = open(p).read()
s, n1 = re.subn(r'(## Context\n)(.*?)(?=\n## Acceptance Criteria)', lambda m: m.group(1) + "\n" + ctx + "\n", s, count=1, flags=re.S)
s, n2 = re.subn(r'- \[ \] \[First criterion\]\n(- \[ \] \[Second criterion\]\n)?', lambda m: ac + "\n", s, count=1)
if n1 != 1 or n2 != 1: sys.exit(3)
open(p, "w").write(s)
PYEOF
                [ -z "$filed" ] && filed="filed $id UNCOMMITTED"
                rel="${file#"$PWD"/}"
                case "$rel" in .tasks/*)
                    "${FOREVER_OWNER_PENDING_CMD:-scripts/commit-pending.sh}" add "$id" "forever-owner backstop filed by check-forever-owner-freshness.sh" "$rel" >/dev/null 2>&1 \
                        || echo "check-forever-owner: $id filed but NOT recorded in the pending-commit manifest — commit $rel by hand" >&2 ;;
                esac
            fi
        fi
    fi
fi

if [ "$JSON" -eq 1 ]; then
    python3 -c 'import json,sys; d=json.loads(sys.argv[1]); d["filed"]=sys.argv[2]; print(json.dumps(d))' "$verdict" "$filed"
elif [ "$rc" = 1 ] || [ "$QUIET" -eq 0 ]; then
    VERDICT_JSON="$verdict" python3 - "$filed" <<'PY'
import json, sys, os
d = json.loads(os.environ["VERDICT_JSON"])
ts = __import__("datetime").datetime.now(__import__("datetime").timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
if d["firing"]:
    print(f"{ts} FIRING forever-owner: mode={d['mode']} enforced={d['enforced']}")
    for f in d["firing"]: print(f"  - {f}")
    for s in d["senders"]: print(f"  sender: {s}")
    if sys.argv[1]: print(f"  task: {sys.argv[1]}")
else:
    when = f"enforces from {d['enforce_at']}" if not d["enforced"] else "enforcing"
    print(f"forever-owner: healthy (mode={d['mode']}, {when}; backstop {d['backstop']})")
PY
fi
exit "$rc"
