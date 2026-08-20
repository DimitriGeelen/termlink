#!/usr/bin/env bash
# guard-layer: source
# T-2801 — stranded / stale auto-deferred pickup envelope check.
#
# The pickup pipeline routes an inbound envelope to .context/pickup/auto-deferred/
# when it is blocked on a local inception task (G-059). T-1425 added
# `pickup_write_breadcrumb()`, dropping a `<envelope>.breadcrumb.yaml` naming the
# blocking task — explicitly so operators and `fw pickup auto-deferred list` could
# trace WHY. T-2072 then made `fw pickup promote-deferred` resolve the blocking task
# from that same file in order to promote the envelope once the blocker ships.
#
# Both halves depend on the breadcrumb. Neither checks that it exists. With no
# breadcrumb there is no blocking task to re-evaluate, so the envelope is not delayed
# — it is PERMANENTLY STRANDED, and the only surface that shows it prints:
#
#     P-043-bug-report.yaml    blocked-by=?   reason=?   at=?
#
# — three question marks and no warning. `fw pickup status` counts it as one more
# deferred item, indistinguishable from one deferred yesterday for a good reason.
#
# The cost, measured on the one envelope that was stuck: P-043 (2026-06-08) is a
# careful report of two framework bugs blocking every inception decision, with
# file:line references and recommended fixes. It was never filed. Two and a half
# months later BUG 2 had been independently re-discovered and re-fixed as T-2304 —
# with the fix the report already recommended — and BUG 1 was still live. So the
# real cost of a stranded envelope is not a late report; it is the same bug solved
# twice while its sibling stayed open.
#
# CLASSES:
#   STRANDED  — no `.breadcrumb.yaml` sibling. Un-promotable by construction. FIRES.
#   STALE     — breadcrumb present, deferred longer than --threshold-days (default
#               30). Either the blocker shipped and promotion did not notice, or it
#               is never going to ship. FIRES.
#   deferred  — breadcrumb present, within the threshold. Healthy: the mechanism is
#               doing its job and the envelope is genuinely waiting.
#
# Both firing classes print the envelope's `summary:`, because the entire failure
# mode is that nobody knows what is in the file.
#
# This DETECTS; it never drains. Promotion belongs to `fw pickup promote-deferred`
# and discarding is a human judgement about whether the work still matters. A
# checker that auto-drained would turn a visible backlog into a silent one — the
# exact trade this exists to reverse.
#
# Deploy-time / ad-hoc check, NOT a cron canary (same tier as
# check-cron-install-drift.sh). Pair with the framework-pickup canary (T-2231),
# which guards the inbound RAIL; this guards the QUEUE behind it.
#
# Exit codes: 0 healthy · 1 stranded/stale envelope(s) · 2 tooling error
set -u

PICKUP_DIR="${PICKUP_DEFERRED_DIR:-.context/pickup/auto-deferred}"
THRESHOLD_DAYS="${PICKUP_DEFERRED_THRESHOLD_DAYS:-30}"
QUIET=0
FORMAT=human

usage() {
    sed -n '3,46p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-pickup-deferred-freshness.sh [OPTIONS]
  --threshold-days N   Age beyond which a breadcrumbed envelope is STALE (default 30)
  --dir PATH           auto-deferred directory (default .context/pickup/auto-deferred)
  --json               Emit a JSON envelope
  --quiet              Print only when firing (cron-friendly)
  -h, --help           This help

Test hooks: PICKUP_DEFERRED_DIR, PICKUP_DEFERRED_THRESHOLD_DAYS.

Exit: 0 healthy · 1 firing · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --threshold-days) shift; [ $# -ge 1 ] || { echo "check-pickup-deferred: --threshold-days requires a value" >&2; exit 2; }; THRESHOLD_DAYS="$1" ;;
        --dir)            shift; [ $# -ge 1 ] || { echo "check-pickup-deferred: --dir requires a value" >&2; exit 2; }; PICKUP_DIR="$1" ;;
        --json)           FORMAT=json ;;
        --quiet)          QUIET=1 ;;
        -h|--help)        usage; exit 0 ;;
        *) echo "check-pickup-deferred: unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

case "$THRESHOLD_DAYS" in
    ''|*[!0-9]*) echo "check-pickup-deferred: --threshold-days must be an integer" >&2; exit 2 ;;
esac

export PICKUP_DEFERRED_DIR="$PICKUP_DIR"
export PICKUP_DEFERRED_THRESHOLD_DAYS="$THRESHOLD_DAYS"
export PICKUP_DEFERRED_FORMAT="$FORMAT"
export PICKUP_DEFERRED_QUIET="$QUIET"

python3 - <<'PYEOF'
import glob, json, os, re, sys, time

DIRPATH   = os.environ["PICKUP_DEFERRED_DIR"]
THRESHOLD = int(os.environ["PICKUP_DEFERRED_THRESHOLD_DAYS"])
FORMAT    = os.environ.get("PICKUP_DEFERRED_FORMAT", "human")
QUIET     = os.environ.get("PICKUP_DEFERRED_QUIET") == "1"

NOW = time.time()
DAY = 86400.0

# An absent directory is healthy, not an error: a project that has never deferred
# an envelope has nothing to report. Distinguish it from a project whose pickup
# tree is missing entirely, which is a tooling problem worth naming.
if not os.path.isdir(DIRPATH):
    parent = os.path.dirname(DIRPATH.rstrip("/"))
    if parent and not os.path.isdir(parent):
        print("check-pickup-deferred: pickup tree not found: %s" % parent, file=sys.stderr)
        sys.exit(2)
    if FORMAT == "json":
        print(json.dumps({"ok": True, "dir": DIRPATH, "exists": False,
                          "total": 0, "stranded_count": 0, "stale_count": 0,
                          "envelopes": [], "threshold_days": THRESHOLD}))
    elif not QUIET:
        print("check-pickup-deferred: healthy — no auto-deferred directory "
              "(nothing has ever been deferred)")
    sys.exit(0)


def summary_of(path):
    """The envelope's `summary:` line — the whole point is knowing what is stuck."""
    try:
        with open(path, "r", errors="replace") as fh:
            for line in fh:
                m = re.match(r'\s*summary:\s*"?(.*?)"?\s*$', line)
                if m and m.group(1):
                    return m.group(1)
    except OSError:
        pass
    return "(no summary: line in envelope)"


def breadcrumb_fields(path):
    fields = {}
    try:
        with open(path, "r", errors="replace") as fh:
            for line in fh:
                m = re.match(r'\s*([a-z_]+):\s*(.*?)\s*$', line)
                if m:
                    fields[m.group(1)] = m.group(2)
    except OSError:
        pass
    return fields


TS_RE = re.compile(r'^\s*(?:timestamp|deferred_at):\s*"?'
                   r'(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z?)"?\s*$')


def recorded_time(path):
    """First RFC3339 `timestamp:`/`deferred_at:` in the file, as epoch seconds."""
    try:
        with open(path, "r", errors="replace") as fh:
            for line in fh:
                m = TS_RE.match(line)
                if m:
                    s = m.group(1).rstrip("Z")
                    try:
                        return time.mktime(time.strptime(s, "%Y-%m-%dT%H:%M:%S")) - time.timezone
                    except ValueError:
                        return None
    except OSError:
        pass
    return None


def age_days_of(env, crumb):
    """How long has this been deferred, and how do we know?

    File mtime is the obvious answer and the wrong one: a git checkout, a fresh
    clone or a worktree stamps every file with the checkout time, so every
    envelope reads as brand new and STALE can never fire. That is precisely the
    environment-dependence PL-213 exists to keep out of these checks — the real
    P-043 is 73 days old and mtime called it 0.

    Prefer what the pipeline actually recorded: the breadcrumb's `deferred_at:`
    (when the deferral happened), then the envelope's own `source.timestamp:`
    (when it was written). Fall back to mtime only when neither exists, and say
    so, because an age derived from mtime is not trustworthy.
    """
    if crumb and os.path.exists(crumb):
        t = recorded_time(crumb)
        if t:
            return (NOW - t) / DAY, "breadcrumb"
    t = recorded_time(env)
    if t:
        return (NOW - t) / DAY, "envelope"
    return (NOW - os.path.getmtime(env)) / DAY, "mtime"


envelopes = []
for env in sorted(glob.glob(os.path.join(DIRPATH, "*.yaml")) +
                  glob.glob(os.path.join(DIRPATH, "*.yml"))):
    if env.endswith(".breadcrumb.yaml") or env.endswith(".breadcrumb.yml"):
        continue
    name = os.path.basename(env)
    crumb = env + ".breadcrumb.yaml"
    age_days, age_src = age_days_of(env, crumb)

    if not os.path.exists(crumb):
        envelopes.append({"file": name, "class": "STRANDED",
                          "age_days": round(age_days, 1), "age_source": age_src,
                          "blocking_task": None,
                          "summary": summary_of(env)})
        continue

    f = breadcrumb_fields(crumb)
    cls = "STALE" if age_days > THRESHOLD else "deferred"
    envelopes.append({"file": name, "class": cls,
                      "age_days": round(age_days, 1), "age_source": age_src,
                      "blocking_task": f.get("blocking_task"),
                      "summary": summary_of(env)})

stranded = [e for e in envelopes if e["class"] == "STRANDED"]
stale    = [e for e in envelopes if e["class"] == "STALE"]
fire = bool(stranded or stale)

if FORMAT == "json":
    print(json.dumps({"ok": not fire, "dir": DIRPATH, "exists": True,
                      "total": len(envelopes),
                      "stranded_count": len(stranded),
                      "stale_count": len(stale),
                      "envelopes": envelopes,
                      "threshold_days": THRESHOLD}))
    sys.exit(1 if fire else 0)

if QUIET and not fire:
    sys.exit(0)

if stranded:
    print("check-pickup-deferred: %d envelope(s) STRANDED — no breadcrumb, so "
          "`fw pickup promote-deferred` can never promote them:" % len(stranded))
    for e in stranded:
        note = "" if e["age_source"] != "mtime" else ", age from mtime — unreliable in a fresh checkout"
        print("  STRANDED: %s  (deferred %.0f days ago%s)" % (e["file"], e["age_days"], note))
        print("      %s" % e["summary"][:160])
    print("  A stranded envelope is not waiting, it is lost. Read it, then either act")
    print("  on its contents or drop it deliberately — `fw pickup auto-deferred list`")
    print("  shows these as `blocked-by=?` and will never say anything is wrong.")

if stale:
    if stranded:
        print("")
    print("check-pickup-deferred: %d envelope(s) STALE — deferred more than %d days:"
          % (len(stale), THRESHOLD))
    for e in stale:
        note = "" if e["age_source"] != "mtime" else ", age from mtime — unreliable"
        print("  STALE: %s  (deferred %.0f days ago%s, blocked-by=%s)"
              % (e["file"], e["age_days"], note, e["blocking_task"] or "?"))
        print("      %s" % e["summary"][:160])
    print("  Either the blocking task shipped and promotion did not notice, or it is")
    print("  not going to ship. Check with `fw pickup promote-deferred`.")

if not fire:
    print("check-pickup-deferred: healthy (%d deferred envelope(s), all breadcrumbed "
          "and within %d days)" % (len(envelopes), THRESHOLD))

sys.exit(1 if fire else 0)
PYEOF
