#!/usr/bin/env python3
"""Portability lint for the arc-008 role cards (T-2204).

A role card must not name a harness, vendor, model, tool, command, path or programming language; those
belong in the adapter and the profile. This test fails on any such term in
docs/design/roles/cards/*.md, and checks every card has the required sections.
"""
import re
import sys
from pathlib import Path

CARDS = Path(__file__).resolve().parent.parent / "docs/design/roles/cards"
PASS = FAIL = 0

# term -> why it is not portable
DENY = {
    r"\bclaude\b": "vendor/harness", r"\banthropic\b": "vendor", r"\bopenai\b": "vendor", r"\bcodex\b": "harness",
    r"\bgemini\b": "vendor", r"\bopencode\b": "harness", r"\bgpt-?\d": "model", r"\bsonnet\b": "model",
    r"\bTodoWrite\b": "harness tool", r"\bmodel:": "model pin", r"\btools:": "tool list",
    r"\bfw\b": "framework CLI", r"\.agentic-framework\b": "framework path", r"\bAEF\b": "framework name",
    r"\bpython\d?\b": "language", r"\bbash\b": "shell", r"\bpuppeteer\b": "tool", r"\bplaywright\b": "tool",
    r"\bMCP\b": "transport", r"\btermlink\b": "transport", r"\bwatchtower\b": "estate UI", r"\bring20\b": "estate",
    r"CLAUDE\.md": "harness file", r"\bgit\b": "VCS command", r"/tmp\b": "path", r"/opt/": "path",
    r"\.tasks/": "framework path", r"\.context/": "framework path", r"scripts/": "path", r"\.py\b": "language",
    r"\bgrep\b": "shell tool", r"192\.168\.": "address", r"\bSMA\b": "estate agent", r"\bmermaid\b": "diagram format (adapter)", r"\bPenelope\b": "estate agent",
}
REQUIRED = ["Purpose", "Inputs", "Output", "Decision rights", "Completion conditions"]


def check(name, cond, detail=""):
    global PASS, FAIL
    if cond:
        PASS += 1
        print(f"PASS {name}")
    else:
        FAIL += 1
        print(f"FAIL {name}  {detail}")


cards = sorted(CARDS.glob("*.md"))
check("cards directory has the common rules and at least 8 role cards",
      (CARDS / "_common.md").exists() and len([c for c in cards if c.name != "_common.md"]) >= 8,
      [c.name for c in cards])
for card in cards:
    text = card.read_text()
    hits = []
    for i, line in enumerate(text.splitlines(), 1):
        for pat, why in DENY.items():
            m = re.search(pat, line, re.I)
            if m:
                hits.append(f"{card.name}:{i} '{m.group(0)}' ({why})")
    check(f"{card.name}: no harness/vendor/tool/path/language terms", not hits, "; ".join(hits[:6]))
    if card.name != "_common.md":
        missing = [s for s in REQUIRED if not re.search(rf"^##\s+\d*\s*{re.escape(s)}", text, re.M | re.I)]
        check(f"{card.name}: has sections {', '.join(REQUIRED)}", not missing, f"missing {missing}")
        # T-2219: drawings are required output (common rules 3.6)
        ids = re.findall(r"\*\*(D-\d+)\*\*", text)
        check(f"{card.name}: lists required drawings and a completion condition for them",
              bool(re.search(r"^###\s+\S+\s+Required drawings", text, re.M)) and len(ids) >= 1
              and re.search(rf"required drawing \(D-1 to D-{len(ids)}\)", text) is not None, f"ids {ids}")
common = (CARDS / "_common.md").read_text() if (CARDS / "_common.md").exists() else ""
check("_common.md defines the hand-back record role-handback/1", "role-handback/1" in common)
check("_common.md defines interview and batch modes", "Interview mode" in common and "Batch mode" in common)

print(f"\n{PASS} passed, {FAIL} failed")
sys.exit(1 if FAIL else 0)
