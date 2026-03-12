#!/usr/bin/env python3
"""
Error analysis from telemetry store — escalation ladder auto-detection.

Reads .context/telemetry/tool-calls.jsonl, filters errors, classifies
into patterns, and maps frequency to escalation ladder levels (A→B→C→D).

Usage:
  python3 analyze-errors.py                    # Analyze current store
  python3 analyze-errors.py --store PATH       # Custom store path
  python3 analyze-errors.py --session UUID     # Filter to one session
  python3 analyze-errors.py --json             # Machine-readable output
"""

import json
import os
import re
import sys
import argparse
from collections import Counter, defaultdict


# ── Error pattern classification ─────────────────────────────────────────────

ANSI_RE = re.compile(r'\x1b\[[0-9;]*[mABCDEFGHJKSTfhilmnprsu]')

PATTERNS = [
    ('fw-help-instead-of-execute', lambda s: 'fw' in s and 'usage' in s,
     'fw CLI commands print help instead of executing'),
    ('task-gate-blocked', lambda s: 'blocked' in s and 'task' in s and 'inception' not in s,
     'Write/Edit blocked by task gate (no active task)'),
    ('inception-gate-blocked', lambda s: 'blocked' in s and 'inception' in s,
     'Commit blocked by inception gate (no go/no-go decision)'),
    ('sovereignty-gate', lambda s: 'sovereignty' in s or 'r-033' in s,
     'Human-owned task blocked by sovereignty gate'),
    ('hook-blocked', lambda s: 'pretooluse' in s or ('hook' in s and ('error' in s or 'blocked' in s)),
     'Operation blocked by PreToolUse hook'),
    ('file-not-found', lambda s: 'file does not exist' in s or 'no such file' in s,
     'File not found (Read/Glob on missing path)'),
    ('edit-before-read', lambda s: 'file has not been read' in s,
     'Edit attempted before reading the file'),
    ('edit-not-unique', lambda s: 'not unique' in s or ('found' in s and 'matches' in s and 'replace_all' in s),
     'Edit string matched multiple locations'),
    ('edit-stale', lambda s: 'modified since read' in s,
     'Edit on stale file (modified by hook/linter since read)'),
    ('edit-not-found', lambda s: 'string to replace not found' in s,
     'Edit target string not found in file'),
    ('permission-denied', lambda s: 'permission' in s and 'denied' in s,
     'Tool permission denied by user'),
    ('parallel-cancel', lambda s: 'cancelled' in s and 'parallel' in s,
     'Parallel tool call cancelled due to sibling error'),
    ('agent-still-running', lambda s: 'cannot resume' in s and 'still running' in s,
     'Attempted to resume agent that is still running'),
    ('file-too-large', lambda s: 'exceeds maximum' in s or 'exceeds' in s and 'size' in s,
     'File too large to read without offset/limit'),
    ('dir-not-file', lambda s: 'eisdir' in s or 'illegal operation on a directory' in s,
     'Read attempted on a directory instead of a file'),
    ('search-timeout', lambda s: 'timed out' in s,
     'Search timed out'),
]


def classify_error(summary):
    """Classify an error summary into a named pattern."""
    if not summary:
        return 'unknown', 'No error summary available'
    s = ANSI_RE.sub('', summary).lower()
    for name, matcher, desc in PATTERNS:
        if matcher(s):
            return name, desc
    # Generic bash exit code
    m = re.match(r'exit code (\d+)', s)
    if m:
        return f'bash-exit-{m.group(1)}', f'Bash command exited with code {m.group(1)}'
    return 'other', 'Unclassified error'


# ── Escalation ladder mapping ────────────────────────────────────────────────

def escalation_level(count, sessions):
    """
    Map error frequency to escalation ladder level.
    A: 1 occurrence — don't repeat
    B: 2-4 occurrences — improve technique
    C: 5-9 or across 2+ sessions — improve tooling
    D: 10+ or across 3+ sessions — change ways of working
    """
    if sessions >= 3 or count >= 10:
        return 'D', 'Change ways of working'
    if sessions >= 2 or count >= 5:
        return 'C', 'Improve tooling'
    if count >= 2:
        return 'B', 'Improve technique'
    return 'A', "Don't repeat"


# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description='Analyze tool call errors and map to escalation ladder'
    )
    parser.add_argument('--store', type=str, default=None,
                        help='Path to tool-calls.jsonl (default: .context/telemetry/tool-calls.jsonl)')
    parser.add_argument('--session', type=str, default=None,
                        help='Filter to specific session UUID')
    parser.add_argument('--json', action='store_true',
                        help='Machine-readable JSON output')
    args = parser.parse_args()

    store_path = args.store or os.path.join(
        os.environ.get('PROJECT_ROOT', os.getcwd()),
        '.context', 'telemetry', 'tool-calls.jsonl'
    )

    if not os.path.exists(store_path):
        print(f'Error: telemetry store not found at {store_path}', file=sys.stderr)
        print('Run extract-tool-calls.py first to populate the store.', file=sys.stderr)
        sys.exit(1)

    # Load and filter
    records = []
    with open(store_path) as f:
        for line in f:
            try:
                r = json.loads(line)
                if args.session and r.get('session_id') != args.session:
                    continue
                records.append(r)
            except (json.JSONDecodeError, ValueError):
                continue

    errors = [r for r in records if r.get('is_error')]
    total = len(records)
    error_count = len(errors)

    if not errors:
        if args.json:
            print(json.dumps({'total': total, 'errors': 0, 'patterns': []}))
        else:
            print(f'No errors found in {total} tool calls.')
        return

    # Classify and aggregate
    pattern_data = defaultdict(lambda: {
        'count': 0, 'sessions': set(), 'tools': Counter(), 'examples': []
    })

    for e in errors:
        pattern_name, pattern_desc = classify_error(e.get('error_summary'))
        pd = pattern_data[pattern_name]
        pd['count'] += 1
        pd['desc'] = pattern_desc
        pd['sessions'].add(e.get('session_id', ''))
        pd['tools'][e.get('tool', '')] += 1
        if len(pd['examples']) < 3:
            summary = ANSI_RE.sub('', e.get('error_summary', ''))[:120]
            pd['examples'].append(summary)

    # Build results with escalation levels
    results = []
    for name, pd in sorted(pattern_data.items(), key=lambda x: -x[1]['count']):
        level, level_desc = escalation_level(pd['count'], len(pd['sessions']))
        results.append({
            'pattern': name,
            'description': pd['desc'],
            'count': pd['count'],
            'sessions': len(pd['sessions']),
            'level': level,
            'level_desc': level_desc,
            'tools': dict(pd['tools'].most_common()),
            'examples': pd['examples'],
        })

    if args.json:
        print(json.dumps({
            'total_calls': total,
            'total_errors': error_count,
            'error_rate': round(error_count / total * 100, 1) if total else 0,
            'patterns': results,
        }, indent=2))
        return

    # Terminal report
    print(f'\n{"=" * 60}')
    print(f'  ERROR ANALYSIS — Escalation Ladder Report')
    print(f'{"=" * 60}')
    print(f'  Total tool calls: {total:,}')
    print(f'  Total errors:     {error_count:,} ({error_count*100//total}%)')
    print(f'  Patterns found:   {len(results)}')
    print(f'{"=" * 60}\n')

    for r in results:
        level_badge = f'[{r["level"]}]'
        tools_str = ', '.join(f'{t}:{c}' for t, c in r['tools'].items())
        print(f'  {level_badge} {r["pattern"]} — {r["count"]}x across {r["sessions"]} session(s)')
        print(f'      {r["level_desc"]}')
        print(f'      {r["description"]}')
        print(f'      Tools: {tools_str}')
        if r['examples']:
            print(f'      Example: {r["examples"][0][:100]}')
        print()

    # Actionable recommendations
    level_d = [r for r in results if r['level'] == 'D']
    level_c = [r for r in results if r['level'] == 'C']

    if level_d or level_c:
        print(f'{"─" * 60}')
        print(f'  RECOMMENDED ACTIONS')
        print(f'{"─" * 60}\n')

        for r in level_d:
            print(f'  [D] {r["pattern"]}: {r["count"]}x — needs systemic fix')
            print(f'      Consider: framework rule, CLAUDE.md guidance, or tooling change')
            print()

        for r in level_c:
            print(f'  [C] {r["pattern"]}: {r["count"]}x — improve tooling')
            print(f'      Consider: better error handling, validation, or helper script')
            print()


if __name__ == '__main__':
    main()
