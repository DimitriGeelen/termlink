"""Tests for web/blueprints/triage.py (T-3649, TermLink local; to be offered to AEF).

Run: python3 -m pytest web/test_triage.py -q
"""

import json
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from web.app import app  # noqa: E402
from web.blueprints import triage  # noqa: E402

ROWS = [
    {"task": "T-1", "title": "Alpha", "summary": "sum <b>bold</b>", "evidence": "ev1",
     "proposed": "park", "details": "group C; status captured"},
    {"task": "T-2", "title": "Beta", "summary": "s2", "evidence": "",
     "proposed": "close", "details": "", "group": "operator"},
    {"task": "T-3", "title": "Gamma", "summary": "s3", "evidence": "",
     "proposed": "more", "details": "group C"},
]


@pytest.fixture
def tdir(tmp_path, monkeypatch):
    d = tmp_path / "triage"
    d.mkdir()
    (d / "batch.json").write_text(json.dumps({"batch": "b1", "rows": ROWS}))
    monkeypatch.setattr(triage, "TRIAGE_DIR", d)
    return d


@pytest.fixture
def client():
    app.config["TESTING"] = True
    app.config["SECRET_KEY"] = "test-secret-key"
    with app.test_client() as c:
        c.get("/")
        with c.session_transaction() as sess:
            token = sess.get("_csrf_token", "")
        yield c, token


def _post(c, token, **body):
    return c.post("/api/v1/triage/answer", json=body, headers={"X-CSRF-Token": token})


def _answers(d):
    p = d / "answers.jsonl"
    return [json.loads(x) for x in p.read_text().splitlines()] if p.exists() else []


def test_page_renders_cards_with_six_buttons_and_nothing_selected(client, tdir):
    c, _ = client
    r = c.get("/triage")
    assert r.status_code == 200
    html = r.get_data(as_text=True)
    for t in ("T-1", "T-2", "T-3"):
        assert f'id="card-{t}"' in html
    for label in ("Prioritize", "Keep", "Park", "Drop", "Close", "Tell me more"):
        assert label in html
    assert html.count('data-choice="') == 18
    assert html.count("triage-proposed") >= 3
    assert 'class="triage-btn triage-chosen' not in html
    assert 'aria-pressed="true"' not in html


def test_batch_text_is_escaped(client, tdir):
    c, _ = client
    html = c.get("/triage").get_data(as_text=True)
    assert "<b>bold</b>" not in html
    assert "&lt;b&gt;bold&lt;/b&gt;" in html


def test_post_appends_answer_and_last_wins(client, tdir):
    c, token = client
    assert _post(c, token, task="T-1", choice="keep").status_code == 200
    assert _post(c, token, task="T-1", choice="drop").status_code == 200
    lines = _answers(tdir)
    assert len(lines) == 2
    assert set(lines[0]) == {"ts", "batch", "task", "choice"}
    assert lines[0]["batch"] == "b1"
    assert triage.load_answers("b1")["T-1"]["choice"] == "drop"


def test_answered_state_shown_and_hidden_by_default(client, tdir):
    c, token = client
    _post(c, token, task="T-1", choice="keep")
    open_html = c.get("/triage").get_data(as_text=True)
    assert 'id="card-T-1"' not in open_html
    all_html = c.get("/triage?show=all").get_data(as_text=True)
    assert 'id="card-T-1"' in all_html
    assert "Answered: <strong>Keep</strong>" in all_html


def test_bad_choice_is_400_and_nothing_written(client, tdir):
    c, token = client
    assert _post(c, token, task="T-1", choice="delete").status_code == 400
    assert _post(c, token, task="../x", choice="keep").status_code == 400
    assert _post(c, token, task="T-999", choice="keep").status_code == 400
    assert _answers(tdir) == []


def test_post_without_csrf_is_refused(client, tdir):
    c, _ = client
    r = c.post("/api/v1/triage/answer", json={"task": "T-1", "choice": "keep"})
    assert r.status_code == 403
    assert _answers(tdir) == []


def test_group_filter_and_details_fallback(client, tdir):
    c, _ = client
    html = c.get("/triage?group=operator").get_data(as_text=True)
    assert 'id="card-T-2"' in html
    assert 'id="card-T-1"' not in html
    html = c.get("/triage?group=C").get_data(as_text=True)
    assert 'id="card-T-1"' in html and 'id="card-T-3"' in html
    assert 'id="card-T-2"' not in html


def test_missing_or_bad_batch_does_not_500(client, tdir):
    c, token = client
    (tdir / "batch.json").write_text("{not json")
    r = c.get("/triage")
    assert r.status_code == 200
    assert "No batch to triage" in r.get_data(as_text=True)
    assert _post(c, token, task="T-1", choice="keep").status_code == 409


def test_page_never_writes_a_task_file(client, tdir, tmp_path):
    c, token = client
    task_file = tmp_path / "tasks-fixture" / "T-2-beta.md"
    task_file.parent.mkdir(parents=True)
    task_file.write_text("---\nid: T-2\nstatus: started-work\n---\n")
    before = {p: p.stat().st_mtime_ns for p in tmp_path.rglob("*") if p.is_file()}
    _post(c, token, task="T-2", choice="close")
    c.get("/triage")
    after = {p for p in tmp_path.rglob("*") if p.is_file()}
    changed = {p for p in after if p not in before or p.stat().st_mtime_ns != before[p]}
    assert changed == {tdir / "answers.jsonl"}


def test_nav_has_triage_leaf():
    from web.shared import nav_group_labels
    assert "Triage" in nav_group_labels("Govern")


def test_compact_rows_details_hidden_and_keys_documented(client, tdir):
    """T-3658: one grid row per task, details collapsed, sticky bar with progress, keys on the page."""
    c, _ = client
    html = c.get("/triage").get_data(as_text=True)
    assert 'id="triage-bar"' in html and "position: sticky" in html
    assert '<progress id="triage-progress" value="0" max="3">' in html
    assert html.count('class="triage-more" hidden') == 3
    assert 'id="triage-keys"' in html
    for key in ("<kbd>j</kbd>", "<kbd>k</kbd>", "<kbd>1</kbd>", "<kbd>6</kbd>", "<kbd>o</kbd>", "<kbd>n</kbd>"):
        assert key in html
    # digit hints follow the button order, one per button
    assert html.count('title="Tell me more (6)"') == 3
    assert "window.__triageKeysInstalled" in html


def test_group_pills_carry_progress(client, tdir):
    c, token = client
    _post(c, token, task="T-1", choice="keep")
    html = c.get("/triage").get_data(as_text=True)
    assert 'data-group="C"' in html
    assert '<span class="triage-g-answered">1</span>/2' in html
    assert "width: 50%" in html
    # proposal still only highlighted after an answer elsewhere
    assert 'aria-pressed="true"' not in html
