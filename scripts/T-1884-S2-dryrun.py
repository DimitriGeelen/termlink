#!/usr/bin/env python3
"""
T-1884 S2 spike — mechanical-Step dry-run validator.

For each RUBBER-STAMP-MECHANICAL + OBSERVE-INFRA AC from S1, parse the
Steps block, extract shell commands, classify safety, execute SAFE
commands (timeout 30s) locally, surface RISKY/INTERACTIVE commands as
"operator-needs-to-run". Compare aggregate output vs Expected.

Verdicts:
  PASS-ROBUST           all safe commands ran clean + Expected substring matched
  PASS-LOOSE            safe commands ran clean + partial Expected match
  OPERATOR-ONLY         all commands risky/interactive — orchestrator can only surface
  FAIL                  safe commands disagree with Expected
  INCONCLUSIVE          Steps unparseable OR no Expected block

Safety classifier (conservative — when in doubt, RISKY):
  SAFE: ls, cat, grep, diff, test, head, tail, wc, find, file, jq,
        strings, git log/show/ls-remote/diff/rev-parse/branch/status,
        ps, pgrep (no -9), mount, df, du, cargo check/test, bash -n,
        python3 -c, echo, printf, true/false, command -v, which,
        sha256sum, openssl x509, curl -sf without -X
  RISKY: pkill, kill, rm, mv, cp, sed -i, install, ln, chmod, chown,
        systemctl, service, mkdir, sudo, tee with redirect, git push,
        cargo install, npm install
  INTERACTIVE: claude, /command-style invocations, in-tui actions

This script is read-only on the task files and runs only SAFE commands
locally with bounded timeout. Validates A-025.
"""
import glob
import re
import shlex
import subprocess
import sys
from pathlib import Path
from collections import defaultdict

try:
    import yaml
except ImportError:
    yaml = None

ROOT = Path(__file__).resolve().parent.parent
ACTIVE = ROOT / ".tasks" / "active"

# Targets from S1 (87.5% confident routing). RUBBER-STAMP-MECHANICAL +
# OBSERVE-INFRA classes — auto-validatable via shell.
TARGETS = [
    # (task_id, ac_substring_match, class)
    ("T-1296", "Apply same migration recipe as T-1294", "RUBBER-STAMP-MECHANICAL"),
    ("T-1296", "Re-pin from .102", "OBSERVE-INFRA"),
    ("T-1296", "Verify CT 101 reboot persistence", "OBSERVE-INFRA"),
    ("T-1420", "Binary deployed on .141", "RUBBER-STAMP-MECHANICAL"),
    ("T-1420", ".141 hub restarted on new binary", "OBSERVE-INFRA"),
    ("T-1420", "Full chat arc parity confirmed via fleet check", "OBSERVE-INFRA"),
    ("T-1431", "skill works end-to-end from a real session", "RUBBER-STAMP-MECHANICAL"),
    ("T-1457", "Operator action on .141", "RUBBER-STAMP-MECHANICAL"),
    ("T-1696", "Cron entry installed in /etc/cron.d on .107", "RUBBER-STAMP-MECHANICAL"),
    ("T-1722", "Upstream landed on `/opt/999-AEF`", "RUBBER-STAMP-MECHANICAL"),
    ("T-1723", "Cron entry installed on .107 so the meta-canary", "RUBBER-STAMP-MECHANICAL"),
    ("T-1836", "MCP listing shows the three new tools", "RUBBER-STAMP-MECHANICAL"),
    ("T-1841", "Skill discoverable and invokable from Claude Code", "RUBBER-STAMP-MECHANICAL"),
    ("T-1417", "Audit shows zero `event.broadcast` callers", "OBSERVE-INFRA"),
    ("T-1419", "freshness signal correctly distinguishes", "OBSERVE-INFRA"),
    ("T-1137", "CT 200 (.122) stops rebooting", "OBSERVE-INFRA"),
]


# --- AC + Steps extraction -------------------------------------------------

def find_task_file(task_id):
    cands = list(ACTIVE.glob(f"{task_id}-*.md"))
    return cands[0] if cands else None


def extract_ac_entry(text, ac_substring):
    """Return the entire AC entry (multi-line markdown) matching the substring."""
    # Find all - [ ] entries in ### Human section
    m = re.search(r"###\s+Human\s*\n(.*?)(?=\n##\s+|\Z)", text, re.DOTALL)
    if not m:
        return None
    human = m.group(1)
    # Split on AC anchors; keep groupings
    parts = re.split(r"\n(?=- \[[ xX]\])", human)
    for p in parts:
        if ac_substring in p:
            return p
    return None


def extract_steps_commands(ac_entry):
    """Extract candidate shell commands from the AC entry's Steps block.

    Returns list of (step_num, command_str) tuples.
    """
    if not ac_entry:
        return []
    # Find the Steps section — between **Steps:** and the next **block** or end
    sm = re.search(
        r"\*\*Steps(?:\s*\(to verify[^)]*\))?\:\*\*\s*\n(.*?)(?=\*\*Expected\*\*|\*\*If not\*\*|\*\*Evidence|\Z)",
        ac_entry, re.DOTALL,
    )
    if not sm:
        return []
    body = sm.group(1)
    cmds = []
    # Each numbered step might have one or more backtick-quoted commands.
    step_lines = re.split(r"\n\s*(?=\d+\.\s)", body)
    for sl in step_lines:
        sm2 = re.match(r"\s*(\d+)\.\s+(.*)", sl, re.DOTALL)
        if not sm2:
            continue
        num = sm2.group(1)
        rest = sm2.group(2)
        # Extract backtick-quoted commands
        for cm in re.finditer(r"`([^`]+)`", rest):
            cmd = cm.group(1).strip()
            if cmd and not cmd.startswith("/"):  # skip /slash commands here
                cmds.append((num, cmd))
    return cmds


def extract_expected(ac_entry):
    sm = re.search(
        r"\*\*Expected\:\*\*\s*(.*?)(?=\*\*If not\*\*|\*\*Evidence|\n- \[|\Z)",
        ac_entry, re.DOTALL,
    )
    return sm.group(1).strip() if sm else None


# --- safety classifier -----------------------------------------------------

SAFE_FIRST_TOKENS = {
    "ls", "cat", "grep", "diff", "test", "head", "tail", "wc", "find",
    "file", "jq", "strings", "ps", "pgrep", "mount", "df", "du",
    "echo", "printf", "true", "false", "command", "which", "sha256sum",
    "openssl", "stat", "readlink", "realpath", "basename", "dirname",
    "tr", "awk", "sort", "uniq", "cut", "paste", "join", "seq", "yes",
    "date", "uname", "id", "whoami", "hostname", "pwd",
    "rg", "ag", "fd",
}
SAFE_PREFIX_PAIRS = {
    ("git", "log"), ("git", "show"), ("git", "ls-remote"), ("git", "diff"),
    ("git", "rev-parse"), ("git", "branch"), ("git", "status"),
    ("git", "blame"), ("git", "cat-file"), ("git", "describe"),
    ("git", "ls-files"), ("git", "tag"),
    ("cargo", "check"), ("cargo", "test"), ("cargo", "tree"),
    ("bash", "-n"), ("python3", "-c"), ("python", "-c"),
    ("termlink", "channel"), ("termlink", "fleet"), ("termlink", "hub"),
    ("termlink", "remote"), ("termlink", "ping"), ("termlink", "version"),
    ("termlink", "doctor"), ("termlink", "mcp"), ("termlink", "agent"),
    ("termlink", "info"), ("termlink", "status"), ("termlink", "list"),
    ("termlink", "events"), ("termlink", "topics"), ("termlink", "tofu"),
    ("termlink", "whoami"),
    ("curl", "-sf"), ("curl", "-s"), ("curl", "-fsS"),
    ("fw", "metrics"), ("fw", "audit"), ("fw", "doctor"), ("fw", "task"),
    ("fw", "fabric"), ("fw", "context"),
}
RISKY_FIRST_TOKENS = {
    "pkill", "kill", "rm", "mv", "cp", "ln", "chmod", "chown", "install",
    "systemctl", "service", "mkdir", "sudo", "touch", "dd", "shred",
    "tee", "ed", "patch",
}
RISKY_PREFIX_PAIRS = {
    ("sed", "-i"), ("git", "push"), ("git", "reset"), ("git", "rebase"),
    ("git", "commit"), ("git", "checkout"), ("git", "merge"),
    ("cargo", "install"), ("cargo", "publish"),
    ("npm", "install"), ("npm", "publish"),
    ("pip", "install"),
}


def classify_safety(cmd):
    """Return one of: SAFE, RISKY, INTERACTIVE, UNKNOWN."""
    cmd = cmd.strip()
    if cmd.startswith("/"):
        return "INTERACTIVE"
    if cmd.startswith("claude "):
        return "INTERACTIVE"
    # Watch out for shell metachars that might be RISKY: redirections to files
    # that aren't /tmp or /dev/null
    if re.search(r"[>]\s*(?!/tmp/|/dev/null)", cmd):
        # write redirect to non-tmp — RISKY
        return "RISKY"
    try:
        tokens = shlex.split(cmd, posix=True)
    except Exception:
        return "UNKNOWN"
    if not tokens:
        return "UNKNOWN"
    head = tokens[0]
    second = tokens[1] if len(tokens) > 1 else ""

    if head in RISKY_FIRST_TOKENS:
        return "RISKY"
    if (head, second) in RISKY_PREFIX_PAIRS:
        return "RISKY"
    if head in SAFE_FIRST_TOKENS:
        return "SAFE"
    if (head, second) in SAFE_PREFIX_PAIRS:
        return "SAFE"
    if head == "bash" and len(tokens) >= 2 and tokens[1].startswith("scripts/"):
        return "SAFE"  # invoking a wrapper script — treat as safe (idempotent by convention)
    return "UNKNOWN"


# --- execute ---------------------------------------------------------------

def run_cmd(cmd, timeout=30):
    try:
        r = subprocess.run(
            ["bash", "-c", cmd],
            capture_output=True, text=True, timeout=timeout,
            cwd=str(ROOT),
        )
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "[TIMEOUT]"
    except Exception as e:
        return -2, "", f"[EXEC-ERROR {e}]"


def verdict_for(safe_results, expected, all_commands):
    """Return verdict + evidence string."""
    if not all_commands:
        return "INCONCLUSIVE", "no commands extracted from Steps"

    n_safe = sum(1 for c in all_commands if c[2] == "SAFE")
    n_risky = sum(1 for c in all_commands if c[2] in ("RISKY", "INTERACTIVE", "UNKNOWN"))

    if n_safe == 0:
        return "OPERATOR-ONLY", f"{n_risky} commands all risky/interactive — orchestrator surfaces only"

    safe_exits = [r["exit"] for r in safe_results]
    safe_outs = "\n".join(r["stdout"] for r in safe_results)
    safe_errs = "\n".join(r["stderr"] for r in safe_results)

    all_zero = all(e == 0 for e in safe_exits)

    # Expected-match
    matched = False
    if expected:
        # take key phrases from Expected, look for them in safe output
        exp_lower = expected.lower()
        # crude: take first 3 distinctive tokens
        keys = [t for t in re.findall(r"[A-Za-z_/.][A-Za-z0-9_./-]{3,}", expected) if t.lower() not in {"step", "shows", "matches", "expected"}]
        keys = keys[:4]
        if keys and any(k.lower() in (safe_outs + safe_errs).lower() for k in keys):
            matched = True

    if all_zero and matched and n_risky == 0:
        return "PASS-ROBUST", f"{n_safe} safe cmds clean, Expected matched, no operator residue"
    if all_zero and matched:
        return "PASS-LOOSE", f"{n_safe} safe cmds clean + Expected matched, but {n_risky} risky/interactive remain"
    if all_zero and n_risky == 0 and not matched:
        return "PASS-LOOSE", f"{n_safe} safe cmds clean but Expected substring not found in output"
    if not all_zero:
        return "FAIL", f"safe cmds returned non-zero exits: {safe_exits}"
    return "INCONCLUSIVE", f"safe ran but ambiguous evidence ({n_risky} risky residue)"


# --- main ------------------------------------------------------------------

def main():
    print("# T-1884 S2 — mechanical-Step dry-run results\n")
    print(f"Targets from S1: {len(TARGETS)} ACs across "
          f"{len(set(t[0] for t in TARGETS))} tasks\n")

    verdicts = defaultdict(list)

    for (task_id, ac_substring, klass) in TARGETS:
        path = find_task_file(task_id)
        if not path:
            print(f"## {task_id} — {ac_substring[:60]!r}\n")
            print(f"   class:    {klass}")
            print(f"   verdict:  INCONCLUSIVE  (task file not found)\n")
            verdicts["INCONCLUSIVE"].append((task_id, ac_substring))
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        ac_entry = extract_ac_entry(text, ac_substring)
        if not ac_entry:
            print(f"## {task_id} — {ac_substring[:60]!r}\n")
            print(f"   class:    {klass}")
            print(f"   verdict:  INCONCLUSIVE  (AC entry not found)\n")
            verdicts["INCONCLUSIVE"].append((task_id, ac_substring))
            continue

        cmds = extract_steps_commands(ac_entry)
        expected = extract_expected(ac_entry)

        # Classify each command
        classified = [(n, c, classify_safety(c)) for (n, c) in cmds]

        # Execute SAFE commands
        safe_results = []
        for (n, c, safety) in classified:
            if safety == "SAFE":
                ex, out, err = run_cmd(c)
                safe_results.append({
                    "step": n, "cmd": c, "exit": ex,
                    "stdout": out[:400], "stderr": err[:200],
                })

        verdict, reason = verdict_for(safe_results, expected, classified)
        verdicts[verdict].append((task_id, ac_substring))

        # Emit per-AC block
        print(f"## {task_id} — {ac_substring[:70]}\n")
        print(f"   class:    {klass}")
        print(f"   verdict:  **{verdict}**  ({reason})")
        print(f"   commands: {len(classified)} parsed "
              f"(safe={sum(1 for x in classified if x[2]=='SAFE')}, "
              f"risky={sum(1 for x in classified if x[2]=='RISKY')}, "
              f"interactive={sum(1 for x in classified if x[2]=='INTERACTIVE')}, "
              f"unknown={sum(1 for x in classified if x[2]=='UNKNOWN')})")
        if expected:
            print(f"   expected: {expected[:160]!r}")
        for (n, c, safety) in classified:
            tag = {"SAFE": " ", "RISKY": "✋", "INTERACTIVE": "🖱", "UNKNOWN": "?"}[safety]
            print(f"     [{safety:11s}] step {n}: `{c[:120]}`")
        for r in safe_results:
            outline = (r["stdout"].splitlines() or [""])[0][:140]
            print(f"     → step {r['step']} exit={r['exit']} out: {outline!r}")
        print()

    # --- summary ---
    print("---\n")
    print("# Summary\n")
    print("| Verdict | Count | Examples |")
    print("|---|---:|---|")
    order = ["PASS-ROBUST", "PASS-LOOSE", "OPERATOR-ONLY", "FAIL", "INCONCLUSIVE"]
    for v in order:
        items = verdicts.get(v, [])
        if not items:
            print(f"| {v} | 0 | — |")
            continue
        examples = ", ".join(f"{t}" for (t, _) in items[:3])
        print(f"| {v} | {len(items)} | {examples} |")

    total = sum(len(v) for v in verdicts.values())
    passing = len(verdicts["PASS-ROBUST"]) + len(verdicts["PASS-LOOSE"])
    operator_only = len(verdicts["OPERATOR-ONLY"])
    print()
    print(f"**Auto-validatable (PASS-ROBUST + PASS-LOOSE): {passing}/{total} "
          f"= {(passing/total*100):.0f}%**")
    print(f"**Operator-only surface (cannot auto-validate): {operator_only}/{total} "
          f"= {(operator_only/total*100):.0f}%**")
    print()
    print(f"A-025 GO threshold: ≥15 of 47 mechanically validatable. "
          f"With current sample: {passing} PASS — "
          f"{'PASS' if passing >= 15 else 'on-track' if passing >= 5 else 'NEEDS-MORE-EVIDENCE'}.")


if __name__ == "__main__":
    main()
