#!/usr/bin/env python3
"""Remove broken pl007-scanner hook from settings.json (script missing after fw upgrade)."""
import json, os

path = os.path.join(os.path.dirname(__file__), '../../.claude/settings.json')
path = os.path.normpath(path)

with open(path) as f:
    cfg = json.load(f)

before = len(cfg['hooks']['PostToolUse'])
cfg['hooks']['PostToolUse'] = [
    h for h in cfg['hooks']['PostToolUse']
    if 'pl007-scanner' not in h.get('hooks', [{}])[0].get('command', '')
]
after = len(cfg['hooks']['PostToolUse'])

with open(path, 'w') as f:
    json.dump(cfg, f, indent=2)
    f.write('\n')

print(f"Removed {before - after} pl007-scanner hook(s) from {path}")
