"""Shared logic for check-approval-surfaces.sh.

ONE definition of the verdict, called by the live path AND by --self-test. Not for
tidiness: a self-test that re-implements its subject proves only that the copy works.
check-receiver-ack-lag.sh shipped with exactly that defect until 2026-08-26 — its
self-test defined its own classifier while the live loop had the same branches inlined,
so reordering the shipped code left the self-test green.

Every predicate that decides whether a HUMAN sees something is the framework's own:
  * count_unchecked_human_acs      web/shared.py       — drives /approvals
  * audit_inception_recommendation lib/task-audit.sh   — BLOCKS review emission

Three operator surfaces, not one:
  build             a Human AC is unchecked -> /review/<id>
  inception         a Decision is recorded + the .reviewed marker exists -> /inception/<id>
  owner-completion  owner: human, every Agent AC ticked, NO Human AC outstanding, still
                    started-work. R-033 forbids an agent closing these at all, and
                    count_unchecked_human_acs returns 0, so they are gated on the operator
                    and appear in NO queue. Found by attempting the closure and reading the
                    refusal -- not by reading the ACs, which is what got it wrong first.
Re-implementing either is how a task looks ready in our check and blank in the operator's.
"""
import datetime as _dt
import importlib.util
import os
import re
import re as re_mod
import subprocess
import sys
import urllib.request

ROOT = os.environ.get("FW_PROJECT_ROOT", "/opt/termlink")
SHARED = os.path.join(ROOT, ".agentic-framework/web/shared.py")
AUDIT = os.path.join(ROOT, ".agentic-framework/lib/task-audit.sh")


def classify(surface, pending, gate_ok, http_code):
    """-> 'PENDING' | 'BROKEN' | 'NONE'.

    NONE     nothing is waiting on the operator for this task.
    PENDING  something is waiting AND the card is usable. Healthy; not a failure.
    BROKEN   something is waiting and the operator could not act on it — no
             Recommendation to read (build), no .reviewed marker to unblock the
             decide verb (inception), or the route does not answer.

    Order is load-bearing: a task with nothing pending is NONE even when its gate is
    unsatisfied, because an absent Recommendation on a task nobody is being asked about
    harms nobody. Checking `pending` first is what keeps this quiet enough to be worth
    running. Anything that reorders these branches must turn --self-test red.
    """
    if not pending:
        return "NONE"
    if not gate_ok or http_code != 200:
        return "BROKEN"
    return "PENDING"


def defer_state(body):
    """-> (label, is_pending) for a task whose recorded decision is DEFER.

    G-053 / T-1451 added `revisit_at` so that a DEFER is a PAUSE rather than a silent
    drop. That makes the field, not the verdict, the thing worth reading:

      future date   parked deliberately and correctly     -> not pending
      date today/past  the return date arrived; decide now -> PENDING
      absent        deferred with no way back              -> PENDING
      unparseable   a date that never arrives              -> PENDING, and loudly:
                    T-2090 carries `revisit_at: Not`, on which every date-based scan
                    fails silent. An unparseable date is worse than a missing one
                    because it LOOKS like a return path.
    """
    m = re_mod.search(r"^revisit_at:\s*(\S+)", body, re_mod.MULTILINE)
    if not m:
        return "DEFER, no revisit_at — no way back", True
    raw = m.group(1)
    try:
        when = _dt.date.fromisoformat(raw)
    except ValueError:
        return f"DEFER, revisit_at={raw!r} UNPARSEABLE — never fires", True
    today = _dt.date.today()
    if when <= today:
        return f"DEFER, revisit DUE {raw} ({(today - when).days}d overdue)", True
    return f"DEFER, parked until {raw}", False

def _load_shared():
    spec = importlib.util.spec_from_file_location("fw_shared", SHARED)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["fw_shared"] = mod
    spec.loader.exec_module(mod)
    return mod


def _rec_gate(path):
    r = subprocess.run(
        ["bash", "-c", f'. "{AUDIT}" >/dev/null 2>&1; audit_inception_recommendation "{path}"'],
        capture_output=True, text=True)
    return r.returncode == 0


def _http(url):
    try:
        with urllib.request.urlopen(url, timeout=8) as r:
            return r.status
    except Exception:  # noqa: BLE001
        return 0


def scan(base):
    """Walk active tasks and report what is pending on each surface."""
    try:
        shared = _load_shared()
    except Exception:  # noqa: BLE001
        return None
    d = os.path.join(ROOT, ".tasks/active")
    if not os.path.isdir(d):
        return None

    rows = []
    for fn in sorted(os.listdir(d)):
        if not fn.endswith(".md"):
            continue
        tid = fn.split("-")[0] + "-" + fn.split("-")[1]
        p = os.path.join(d, fn)
        try:
            body = open(p, encoding="utf-8", errors="replace").read()
        except OSError:
            continue

        wf = re.search(r"^workflow_type:\s*(\S+)", body, re.M)
        wf = wf.group(1) if wf else ""

        if wf == "inception":
            dm = re.search(r"^\*\*Decision\*\*:\s*(GO|NO-GO|DEFER)", body, re.M)
            if not dm:
                continue  # no decision recorded -> nothing pending on this surface
            verdict = dm.group(1)
            why, pend = f"decision={verdict}", True
            if verdict == "DEFER":
                # A recorded DEFER is a decision ALREADY MADE. Surfacing it as one to make
                # is manufacturing work — and it buried the two whose revisit date had
                # actually arrived among nine that needed nothing. DEFER is pending only
                # when its return path is due, absent, or unusable.
                why, pend = defer_state(body)
            if not pend:
                continue
            marker = os.path.exists(os.path.join(ROOT, ".context/working", f".reviewed-{tid}"))
            code = _http(f"{base}/inception/{tid}")
            rows.append({
                "id": tid, "surface": "inception", "pending": 1, "gate": marker,
                "http": code, "why": why,
                "blocker": ("no .reviewed marker — `fw inception decide` is gated (T-973)"
                            if not marker else f"route returned {code}"),
            })
            continue

        n = shared.count_unchecked_human_acs(body)
        if not n:
            # No Human AC pending — but that is not the same as nothing pending.
            # A human-OWNED task whose Agent ACs are all ticked cannot be closed by an
            # agent at all (R-033 sovereignty gate, which keys on `owner:` and not on
            # ACs). Its work is finished and only the operator can end it, yet
            # count_unchecked_human_acs reports 0, so it appears in no queue. The
            # framework prints /review/<id> when it refuses the close; that route is
            # the surface, and this is what enumerates it.
            st = re.search(r"^status:\s*(\S+)", body, re.M)
            st = st.group(1) if st else ""
            owner = re.search(r"^owner:\s*(\S+)", body, re.M)
            owner = owner.group(1) if owner else ""
            if st not in ("started-work", "issues") or owner != "human":
                continue
            stripped = re.sub(r"<!--.*?-->", "", body, flags=re.S)
            am = re.search(r"^### Agent\b(.*?)(?=^### |^## |\Z)", stripped, re.S | re.M)
            sec = am.group(1) if am else ""
            done = len(re.findall(r"^- \[x\]", sec, re.M | re.I))
            openn = len(re.findall(r"^- \[ \]", sec, re.M))
            if openn or not done:
                continue
            code = _http(f"{base}/review/{tid}")
            rows.append({
                "id": tid, "surface": "owner-completion", "pending": 1, "gate": True,
                "http": code, "why": f"{done} Agent AC(s) done",
                "blocker": f"route returned {code}",
            })
            continue
        gate = _rec_gate(p)
        code = _http(f"{base}/review/{tid}")
        rows.append({
            "id": tid, "surface": "build", "pending": n, "gate": gate,
            "http": code, "why": f"{n} Human AC(s)",
            "blocker": ("no substantive ## Recommendation — the card asks the operator "
                        "to approve a blank form" if not gate else f"route returned {code}"),
        })
    return rows
