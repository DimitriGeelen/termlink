#!/usr/bin/env python3
"""
fw independent-review (T-1885 v0.1) — drain the review-waiting queue.

Consumer-side orchestrator that walks tasks with unchecked ### Human ACs,
classifies each AC by content + prefix, runs the right local validator
(REVIEW-CLI / CLI-WATCH / RUBBER-STAMP-RELEASE), surfaces evidence into the
source task's ## Updates block, and auto-files investigate-and-fix follow-up
tasks on FAIL / INCONCLUSIVE (anti-pile-up per inception D4).

CONSTITUTIONAL RAIL (T-1950 D36/113/213): the orchestrator NEVER auto-ticks
### Human ACs by default. The producer agent NEVER classifies its own work
— each AC is validated by a separate subprocess (independent-reviewer rail
per inception D1). `--tick-mechanical-pass` opt-in (Tier-2 logged per
session) ticks RUBBER-STAMP-* PASS-ROBUST checkboxes; default is OFF.

USAGE

    fw independent-review                       batch over all unchecked Human ACs
    fw independent-review --task T-XXX          one task only
    fw independent-review --class REVIEW-CLI    one class only
    fw independent-review --dry-run             classify + validate, do NOT modify task files
    fw independent-review --resume              continue from journaled state
    fw independent-review --tick-mechanical-pass  opt-in to tick PASS-ROBUST mechanical (Tier-2 logged)

Invoke via:
    python3 scripts/independent-review.py [args]
"""
import argparse
import json
import os
import re
import sys
import subprocess
import datetime
from pathlib import Path
from collections import Counter, defaultdict

# Resolve project root from script location
ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

from lib.review_classifier import (
    extract_ac_entries,
    classify_ac,
    extract_steps_commands,
    extract_expected,
    parse_frontmatter,
    v01_class_routes_to,
)
from lib.review_validators import validate


STATE_FILE = ROOT / ".context" / "working" / ".independent-review-state.json"
ACTIVE_DIR = ROOT / ".tasks" / "active"


# ---------- AC discovery ----------------------------------------------------

def list_target_tasks(filter_task=None, since_days=None):
    """Return list of task file paths with unchecked Human ACs."""
    paths = sorted(ACTIVE_DIR.glob("T-*.md"))
    if filter_task:
        paths = [p for p in paths if p.name.startswith(f"{filter_task}-")]

    # since-days filter on last_update
    if since_days:
        cutoff = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(days=since_days)
        filtered = []
        for p in paths:
            text = p.read_text(encoding="utf-8", errors="replace")
            fm = parse_frontmatter(text)
            last = fm.get("last_update", "")
            try:
                ts = datetime.datetime.fromisoformat(last.replace("Z", "+00:00"))
                if ts >= cutoff:
                    filtered.append(p)
            except Exception:
                pass
        paths = filtered

    return paths


def find_acs_to_review(paths, filter_class=None):
    """Yield {task_id, path, ac_idx, ac, klass, confident, reason} for each unchecked Human AC."""
    for path in paths:
        text = path.read_text(encoding="utf-8", errors="replace")
        fm = parse_frontmatter(text)
        task_id = fm.get("id", path.stem.split("-", 2)[0])
        if isinstance(task_id, list):
            task_id = "-".join(map(str, task_id))
        acs = extract_ac_entries(text, include_checked=False)
        for i, ac in enumerate(acs):
            klass, confident, reason = classify_ac(ac["prefix"], ac["body"], ac["rest"])
            if filter_class and klass != filter_class:
                continue
            yield {
                "task_id": task_id,
                "path": str(path),
                "ac_idx": i,
                "ac": ac,
                "klass": klass,
                "confident": confident,
                "reason": reason,
            }


# ---------- state journal --------------------------------------------------

def load_state():
    if STATE_FILE.exists():
        try:
            return json.loads(STATE_FILE.read_text())
        except Exception:
            return {}
    return {}


def save_state(state):
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    tmp = STATE_FILE.with_suffix(".tmp")
    tmp.write_text(json.dumps(state, indent=2))
    tmp.replace(STATE_FILE)


def state_key(task_id, ac_idx):
    return f"{task_id}#{ac_idx}"


# ---------- Updates writer -------------------------------------------------

def append_to_updates(task_path, task_id, ac_body, verdict, reason, evidence):
    """Append a timestamped Updates entry. Never modifies AC checkboxes."""
    text = Path(task_path).read_text(encoding="utf-8", errors="replace")
    ts = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    entry = (
        f"\n### {ts} — independent-review [{verdict}] [agent]\n"
        f"- **AC:** {ac_body[:140]}\n"
        f"- **Verdict:** {verdict} ({reason})\n"
        f"- **Evidence:**\n\n"
    )
    for line in evidence.splitlines():
        entry += f"  {line}\n"

    # Append to ## Updates section (insert before next ## or EOF)
    m = re.search(r"\n##\s+Updates\s*\n", text)
    if not m:
        # No Updates section — append one
        text = text.rstrip() + "\n\n## Updates\n" + entry
    else:
        # Find end of Updates section
        start = m.end()
        next_h2 = re.search(r"\n##\s+", text[start:])
        if next_h2:
            insert_at = start + next_h2.start()
        else:
            insert_at = len(text)
        text = text[:insert_at] + entry + text[insert_at:]

    Path(task_path).write_text(text, encoding="utf-8")


def tick_human_ac(task_path, ac_idx):
    """Tick the ac_idx-th unchecked Human AC. ONLY callable when --tick-mechanical-pass
    is set AND verdict is PASS-ROBUST AND class is RUBBER-STAMP-*."""
    text = Path(task_path).read_text(encoding="utf-8", errors="replace")
    # Find Human section
    hm = re.search(r"(###\s+Human\s*\n)", text)
    if not hm:
        return False
    # Walk unchecked ACs in order, replace the ac_idx-th one
    human_start = hm.end()
    # Find end of Human section
    next_h = re.search(r"\n##\s+|\n###\s+(?!Human)", text[human_start:])
    human_end = human_start + next_h.start() if next_h else len(text)
    human_body = text[human_start:human_end]

    # Find all unchecked AC bullet starts (line position)
    matches = [m for m in re.finditer(r"^- \[ \]", human_body, re.MULTILINE)]
    if ac_idx >= len(matches):
        return False
    pos = matches[ac_idx].start()
    new_body = human_body[:pos] + "- [x]" + human_body[pos + 5:]
    text = text[:human_start] + new_body + text[human_end:]
    Path(task_path).write_text(text, encoding="utf-8")
    return True


# ---------- follow-up filer ------------------------------------------------

def file_followup(source_task, ac_body, verdict, reason):
    """Create T-XXXX investigate-T-<src> via fw task create. Anti-pile-up D4."""
    fw_bin = ROOT / ".agentic-framework" / "bin" / "fw"
    if not fw_bin.is_file():
        return None
    name = f"Investigate {source_task} AC '{ac_body[:60]}' — independent-review {verdict}"
    description = (
        f"Auto-filed by fw independent-review (T-1885 v0.1) on a {verdict} verdict.\n\n"
        f"Source task: {source_task}\n"
        f"AC: {ac_body}\n"
        f"Verdict: {verdict} ({reason})\n\n"
        f"Next action: diagnose whether the AC's verification expectation is wrong "
        f"or the system regressed. See source task's ## Updates block for evidence."
    )
    try:
        r = subprocess.run(
            [str(fw_bin), "task", "create",
             "--name", name,
             "--description", description,
             "--type", "build", "--owner", "agent",
             "--tags", "bug,auto-filed",
             "--related", source_task],
            capture_output=True, text=True, timeout=15, cwd=str(ROOT),
        )
        if r.returncode != 0:
            return None
        m = re.search(r"\b(T-\d+)\b", r.stdout)
        return m.group(1) if m else None
    except Exception:
        return None


# ---------- main flow ------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(description="fw independent-review v0.1")
    ap.add_argument("--task", help="single task id (e.g. T-1486)")
    ap.add_argument("--class", dest="filter_class", help="restrict to one AC class")
    ap.add_argument("--since", type=int, help="only tasks updated within N days")
    ap.add_argument("--dry-run", action="store_true", help="classify + validate; do NOT write to task files / file followups")
    ap.add_argument("--resume", action="store_true", help="skip ACs already in state journal")
    ap.add_argument("--tick-mechanical-pass", action="store_true",
                    help="Tier-2 opt-in: tick RUBBER-STAMP-* PASS-ROBUST Human ACs (memory [Validate-don't-punt])")
    ap.add_argument("--no-followup", action="store_true", help="skip auto-followup filing on FAIL/INCONCLUSIVE")
    ap.add_argument("--limit", type=int, help="stop after N ACs (for dev/testing)")
    args = ap.parse_args()

    if args.tick_mechanical_pass and not args.dry_run:
        # Tier-2 log
        log_path = ROOT / ".context" / "working" / ".gate-bypass-log.yaml"
        log_path.parent.mkdir(parents=True, exist_ok=True)
        ts = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        with log_path.open("a") as f:
            f.write(f"- ts: {ts}\n  action: tick-mechanical-pass\n  source: scripts/independent-review.py\n  rationale: operator opt-in per T-1884 D1 (independent-reviewer-gated auto-tick)\n")

    state = load_state() if args.resume else {}

    print(f"# fw independent-review v0.1")
    print(f"# {'DRY-RUN' if args.dry_run else 'LIVE'} mode | "
          f"task-filter={args.task or '*'} class-filter={args.filter_class or '*'} "
          f"since={args.since or '*'} tick={args.tick_mechanical_pass}")
    print()

    paths = list_target_tasks(filter_task=args.task, since_days=args.since)
    if not paths:
        print("No tasks match the filter.")
        return 0

    verdicts = Counter()
    followups = []
    ticked = []
    skipped_resume = 0
    processed = 0

    for entry in find_acs_to_review(paths, filter_class=args.filter_class):
        if args.limit and processed >= args.limit:
            break
        key = state_key(entry["task_id"], entry["ac_idx"])
        if args.resume and key in state:
            skipped_resume += 1
            continue

        ac = entry["ac"]
        klass = entry["klass"]
        steps_cmds = extract_steps_commands(ac["full_entry"])
        expected = extract_expected(ac["full_entry"])

        result = validate(ac["full_entry"], klass, steps_cmds, expected)
        verdicts[result["verdict"]] += 1
        processed += 1

        marker = {
            "PASS-ROBUST": "✓",
            "PASS-LOOSE": "~",
            "FAIL": "✗",
            "INCONCLUSIVE": "?",
            "SURFACE": "·",
        }.get(result["verdict"], "·")

        print(f"{marker} {entry['task_id']:8s} [{klass:23s}] {result['verdict']:13s} {ac['body'][:60]}")
        if result["verdict"] in ("FAIL",):
            print(f"            reason: {result['reason']}")

        # Side effects (skipped in --dry-run)
        if not args.dry_run:
            append_to_updates(
                entry["path"], entry["task_id"], ac["body"],
                result["verdict"], result["reason"], result["evidence"],
            )

            # Tick if --tick-mechanical-pass + PASS-ROBUST + RUBBER-STAMP-*
            if (args.tick_mechanical_pass
                    and result["verdict"] == "PASS-ROBUST"
                    and klass.startswith("RUBBER-STAMP")):
                if tick_human_ac(entry["path"], entry["ac_idx"]):
                    ticked.append((entry["task_id"], ac["body"][:60]))

            # Auto-followup on FAIL / INCONCLUSIVE (anti-pile-up D4)
            if result["verdict"] in ("FAIL", "INCONCLUSIVE") and not args.no_followup:
                fu = file_followup(entry["task_id"], ac["body"], result["verdict"], result["reason"])
                if fu:
                    followups.append((entry["task_id"], fu))

            # Journal state for crash-safe resume
            state[key] = {
                "verdict": result["verdict"],
                "ts": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            }
            save_state(state)

    # ---- summary ----
    print()
    print(f"# Summary  (processed={processed}, skipped-resume={skipped_resume})")
    print()
    for v in ["PASS-ROBUST", "PASS-LOOSE", "FAIL", "INCONCLUSIVE", "SURFACE"]:
        if verdicts.get(v):
            print(f"  {v:13s} {verdicts[v]}")
    print()
    if ticked:
        print(f"Auto-ticked {len(ticked)} RUBBER-STAMP PASS-ROBUST ACs (Tier-2):")
        for (t, b) in ticked:
            print(f"  - {t}: {b}")
        print()
    if followups:
        print(f"Auto-filed {len(followups)} follow-ups:")
        for (src, dst) in followups:
            print(f"  - {src} → {dst}")
        print()

    return 0


if __name__ == "__main__":
    sys.exit(main())
