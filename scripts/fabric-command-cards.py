#!/usr/bin/env python3
"""T-3544 (operator ruling A'1 on T-3103): fabric cards for the slash commands in .claude/commands/.

A command file is a set of instructions the agent executes, so its real dependencies are what it
tells the agent to RUN: repo scripts and `termlink` / `fw` verbs inside code (fenced blocks or inline
backticks). A script named only in prose ("see scripts/x.sh") is a pointer, not a dependency, and
is left out; that is the difference between these cards and header-only ones.

Usage: fabric-command-cards.py [--check] [--root DIR]
  default  write/refresh .fabric/components/claude-commands-<name>.yaml for every command file
  --check  write nothing; exit 1 if a command file has no card or its card's depends_on/interfaces
           differ from what the file calls now (exit 2 on tooling errors)
"""
import argparse
import os
import re
import sys

import yaml

SCRIPT_RE = re.compile(r"(?<![\w/.-])(scripts/[A-Za-z0-9_./-]+\.(?:sh|py))")
VERB_RE = {
    "termlink": re.compile(r"\btermlink ((?:[a-z][a-z0-9-]*)(?: [a-z][a-z0-9-]*)?)"),
    "fw": re.compile(r"\bfw ((?:[a-z][a-z0-9-]*)(?: [a-z][a-z0-9-]*)?)"),
}
FENCE_RE = re.compile(r"^\s*(```|~~~)")
INLINE_RE = re.compile(r"`([^`\n]+)`")
# The termlink CLI's card: a command that runs `termlink <verb>` depends on it. (`fw` is vendored and has
# no card here; its verbs are recorded under interfaces only.)
TERMLINK_CLI = "crates/termlink-cli/src/main.rs"
# termlink subcommands that take a positional or option next, so the second word is not a verb.
SINGLE_WORD = {"list", "ping", "status", "exec", "interact", "attach", "info", "topics", "install",
               "mirror", "whoami", "version", "spawn", "register", "inject", "doctor", "send", "signal"}


def code_spans(text):
    """Every piece of the file that is code: fenced block lines and inline backtick spans."""
    spans, in_fence = [], False
    for line in text.splitlines():
        if FENCE_RE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            spans.append(line)
        else:
            spans.extend(INLINE_RE.findall(line))
    return spans


def calls(text, root):
    scripts, verbs = set(), {"termlink": set(), "fw": set()}
    for span in code_spans(text):
        for m in SCRIPT_RE.findall(span):
            if os.path.isfile(os.path.join(root, m)):
                scripts.add(m)
        for tool, rx in VERB_RE.items():
            for m in rx.findall(span):
                words = m.split()
                if tool == "termlink" and words[0] in SINGLE_WORD:
                    words = words[:1]
                verbs[tool].add(" ".join(words))
    return sorted(scripts), {k: sorted(v) for k, v in verbs.items() if v}


def purpose(text):
    for line in text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return ""


def card_for(path, root):
    rel = os.path.relpath(path, root)
    name = os.path.splitext(os.path.basename(path))[0]
    text = open(path, encoding="utf-8").read()
    scripts, verbs = calls(text, root)
    interfaces: list[dict] = [{"type": "slash-command", "command": f"/{name}"}]
    for tool, vs in verbs.items():
        interfaces.append({"type": "cli", "tool": tool, "commands": vs})
    return {
        "id": rel,
        "name": name,
        "type": "slash-command",
        "subsystem": "claude-commands",
        "location": rel,
        "tags": ["slash-command"],
        "purpose": purpose(text),
        "purpose_source": "header-comment",
        "interfaces": interfaces,
        "depends_on": [{"target": s, "type": "calls"} for s in scripts]
        + ([{"target": TERMLINK_CLI, "type": "calls"}] if verbs.get("termlink") and os.path.isfile(os.path.join(root, TERMLINK_CLI)) else []),
        "depended_by": [],
        "created_by": "T-3544",
    }


def card_path(root, name):
    return os.path.join(root, ".fabric", "components", f"claude-commands-{name}.yaml")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--root", default=os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
    a = ap.parse_args()
    cdir = os.path.join(a.root, ".claude", "commands")
    try:
        files = sorted(os.path.join(cdir, f) for f in os.listdir(cdir) if f.endswith(".md"))
    except OSError as e:
        print(f"fabric-command-cards: cannot read {cdir}: {e}", file=sys.stderr)
        return 2
    if not files:
        print(f"fabric-command-cards: no command files in {cdir}", file=sys.stderr)
        return 2
    drift = []
    for f in files:
        card = card_for(f, a.root)
        out = card_path(a.root, card["name"])
        if a.check:
            try:
                have = yaml.safe_load(open(out)) or {}
            except OSError:
                drift.append(f"{card['location']}: no card ({os.path.relpath(out, a.root)})")
                continue
            for key in ("depends_on", "interfaces"):
                if have.get(key) != card[key]:
                    drift.append(f"{card['location']}: card {key} differs from what the file calls now")
            continue
        old = {}
        if os.path.exists(out):
            old = yaml.safe_load(open(out)) or {}
        card["depended_by"] = old.get("depended_by") or []
        with open(out, "w") as fh:
            yaml.safe_dump(card, fh, sort_keys=False, allow_unicode=True)
    if a.check:
        for d in drift:
            print(d)
        print(f"fabric-command-cards: {len(files)} command file(s), {len(drift)} problem(s)")
        return 1 if drift else 0
    print(f"fabric-command-cards: wrote {len(files)} card(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
