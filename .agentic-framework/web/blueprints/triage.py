"""Triage blueprint: six-button backlog triage page (T-3649, TermLink local; to be offered to AEF).

Pattern from 055 (their T-682, cockpit /triage). An agent writes a batch of
proposals to `.context/triage/batch.json`:

    {"batch": "<id>", "rows": [{"task", "title", "summary", "evidence",
                                "proposed", "details", ["group"]}]}

The operator rules on each card with one of six fixed buttons. Each click
appends one line `{ts, batch, task, choice}` to `.context/triage/answers.jsonl`
(the last line per task wins). The page NEVER writes a task file: an agent
applies the answers afterwards through the normal gates
(`scripts/triage-apply.sh` in TermLink). Nothing is pre-selected, because
055's operator overrode 5 of 7 proposals; the proposal is only highlighted.
"""

import fcntl
import json
import logging
import re
from datetime import datetime, timezone

from flask import Blueprint, jsonify, request

from web.shared import NAV_GROUPS, NAV_ITEMS, PROJECT_ROOT, render_page

logger = logging.getLogger(__name__)

bp = Blueprint("triage", __name__)

# Resolved from the project root (never cwd). Tests repoint TRIAGE_DIR.
TRIAGE_DIR = PROJECT_ROOT / ".context" / "triage"

# Order is the button order on the card.
CHOICES = ("prioritize", "keep", "park", "drop", "close", "more")
CHOICE_LABELS = {
    "prioritize": "Prioritize",
    "keep": "Keep",
    "park": "Park",
    "drop": "Drop",
    "close": "Close",
    "more": "Tell me more",
}

_TASK_RE = re.compile(r"^T-\d+$")
_GROUP_IN_DETAILS = re.compile(r"\bgroup\s+([A-Za-z0-9_-]+)")


def _batch_file():
    return TRIAGE_DIR / "batch.json"


def _answers_file():
    return TRIAGE_DIR / "answers.jsonl"


def _row_group(row):
    """Explicit `group` field wins; else a `group X` token in details; else ''."""
    g = row.get("group")
    if isinstance(g, str) and g.strip():
        return g.strip()
    m = _GROUP_IN_DETAILS.search(str(row.get("details") or ""))
    return m.group(1) if m else ""


def load_batch():
    """Return (batch_id, rows, error). Never raises on bad input."""
    path = _batch_file()
    if not path.exists():
        return "", [], f"No batch file at {path}"
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return "", [], f"Cannot read {path}: {exc}"
    if not isinstance(data, dict) or not isinstance(data.get("rows"), list):
        return "", [], f"{path} has no 'rows' list"
    rows = []
    for raw in data["rows"]:
        if not isinstance(raw, dict) or not _TASK_RE.match(str(raw.get("task", ""))):
            continue
        row = {k: ("" if raw.get(k) is None else str(raw.get(k)))
               for k in ("task", "title", "summary", "evidence", "proposed", "details")}
        row["proposed"] = row["proposed"].strip().lower()
        row["group"] = _row_group(raw)
        rows.append(row)
    return str(data.get("batch") or ""), rows, ""


def load_answers(batch_id=None):
    """Last answer per task, optionally restricted to one batch. Bad lines are skipped."""
    path = _answers_file()
    out = {}
    if not path.exists():
        return out
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        logger.warning("triage: cannot read %s: %s", path, exc)
        return out
    for line in lines:
        try:
            rec = json.loads(line)
        except ValueError:
            continue
        if not isinstance(rec, dict) or rec.get("choice") not in CHOICES:
            continue
        if batch_id is not None and rec.get("batch") != batch_id:
            continue
        task = rec.get("task")
        if isinstance(task, str):
            out[task] = rec
    return out


def _append_answer(rec):
    path = _answers_file()
    path.parent.mkdir(parents=True, exist_ok=True)
    line = json.dumps(rec, ensure_ascii=False) + "\n"
    with open(path, "a", encoding="utf-8") as f:
        fcntl.flock(f, fcntl.LOCK_EX)
        try:
            f.write(line)
            f.flush()
        finally:
            fcntl.flock(f, fcntl.LOCK_UN)


@bp.route("/triage")
def triage_page():
    batch_id, rows, error = load_batch()
    answers = load_answers(batch_id)

    groups = {}
    for r in rows:
        groups.setdefault(r["group"], [0, 0])
        groups[r["group"]][0] += 1
        if r["task"] in answers:
            groups[r["group"]][1] += 1
    group_list = [{"name": g, "total": t, "answered": a}
                  for g, (t, a) in sorted(groups.items()) if g]

    group = request.args.get("group", "")
    show = request.args.get("show", "open")
    if show not in ("open", "all"):
        show = "open"

    visible = [r for r in rows if not group or r["group"] == group]
    in_scope = len(visible)
    answered_in_scope = sum(1 for r in visible if r["task"] in answers)
    if show == "open":
        visible = [r for r in visible if r["task"] not in answers]
    for r in visible:
        a = answers.get(r["task"])
        r["answer"] = a.get("choice") if a else ""

    return render_page(
        "triage.html",
        page_title="Triage",
        batch_id=batch_id,
        error=error,
        rows=visible,
        total=len(rows),
        answered_total=sum(1 for r in rows if r["task"] in answers),
        in_scope=in_scope,
        answered_in_scope=answered_in_scope,
        groups=group_list,
        group=group,
        show=show,
        choices=CHOICES,
        choice_labels=CHOICE_LABELS,
    )


@bp.route("/api/v1/triage/answer", methods=["POST"])
def triage_answer():
    body = request.get_json(silent=True) if request.is_json else request.form
    body = body or {}
    task = str(body.get("task") or "").strip()
    choice = str(body.get("choice") or "").strip().lower()

    if choice not in CHOICES:
        return jsonify({"error": f"choice must be one of {', '.join(CHOICES)}"}), 400
    if not _TASK_RE.match(task):
        return jsonify({"error": "task must look like T-123"}), 400

    batch_id, rows, error = load_batch()
    if error:
        return jsonify({"error": error}), 409
    if task not in {r["task"] for r in rows}:
        return jsonify({"error": f"{task} is not in batch {batch_id}"}), 400

    rec = {
        "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "batch": batch_id,
        "task": task,
        "choice": choice,
    }
    try:
        _append_answer(rec)
    except OSError as exc:
        logger.error("triage: cannot append answer: %s", exc)
        return jsonify({"error": f"cannot record answer: {exc}"}), 500
    return jsonify({"status": "recorded", "answer": rec})


def _register_nav():
    """Add a Govern > Approvals & Decisions nav leaf without editing shared.py.

    Kept inside this new file so the vendored divergence stays new-files-only
    (T-3649 / .vendor-divergence.yaml). Idempotent; a missing section is a no-op.
    """
    leaf = ("Triage", "triage.triage_page", None)
    for gname, items in NAV_GROUPS:
        if gname != "Govern":
            continue
        for item in items:
            if len(item) == 2 and isinstance(item[1], list) and item[0] == "Approvals & Decisions":
                if leaf not in item[1]:
                    item[1].append(leaf)
                    NAV_ITEMS.append(leaf)
                return


_register_nav()
