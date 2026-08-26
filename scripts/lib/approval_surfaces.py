"""Shared logic for check-approval-surfaces.sh.

ONE definition of the verdict, called by the live path AND by --self-test. Not for
tidiness: a self-test that re-implements its subject proves only that the copy works.
check-receiver-ack-lag.sh shipped with exactly that defect until 2026-08-26 — its
self-test defined its own classifier while the live loop had the same branches inlined,
so reordering the shipped code left the self-test green.

Every predicate that decides whether a HUMAN sees something is the framework's own:
  * count_unchecked_human_acs      web/shared.py       — drives /approvals
  * audit_inception_recommendation lib/task-audit.sh   — BLOCKS review emission
Re-implementing either is how a task looks ready in our check and blank in the operator's.
"""
import importlib.util
import os
import re
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
            marker = os.path.exists(os.path.join(ROOT, ".context/working", f".reviewed-{tid}"))
            code = _http(f"{base}/inception/{tid}")
            rows.append({
                "id": tid, "surface": "inception", "pending": 1, "gate": marker,
                "http": code, "why": f"decision={dm.group(1)}",
                "blocker": ("no .reviewed marker — `fw inception decide` is gated (T-973)"
                            if not marker else f"route returned {code}"),
            })
            continue

        n = shared.count_unchecked_human_acs(body)
        if not n:
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
