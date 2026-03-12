#!/usr/bin/env python3
"""
Tool call statistics from telemetry store.

Reads .context/telemetry/tool-calls.jsonl and produces a summary report
covering call counts, error rates, token usage, and session breakdown.

Usage:
  python3 tool-stats.py                    # Full report
  python3 tool-stats.py --session UUID     # Single session
  python3 tool-stats.py --compact          # One-line summary (for handover)
  python3 tool-stats.py --json             # Machine-readable output
"""

import json
import os
import sys
import argparse
from collections import Counter, defaultdict
from datetime import datetime


def load_records(store_path, session_filter=None):
    """Load and optionally filter records from the telemetry store."""
    records = []
    with open(store_path) as f:
        for line in f:
            try:
                r = json.loads(line)
                if session_filter and r.get('session_id') != session_filter:
                    continue
                records.append(r)
            except (json.JSONDecodeError, ValueError):
                continue
    return records


def compute_stats(records):
    """Compute statistics from tool call records."""
    if not records:
        return None

    total = len(records)
    errors = sum(1 for r in records if r.get('is_error'))
    sidechain = sum(1 for r in records if r.get('is_sidechain'))

    # Tool breakdown
    tool_counts = Counter()
    tool_errors = Counter()
    for r in records:
        tool = r.get('tool', 'unknown')
        tool_counts[tool] += 1
        if r.get('is_error'):
            tool_errors[tool] += 1

    # Token usage
    tokens_in = sum(r.get('tokens_in', 0) for r in records)
    tokens_out = sum(r.get('tokens_out', 0) for r in records)

    # Session breakdown
    sessions = defaultdict(lambda: {'calls': 0, 'errors': 0})
    for r in records:
        sid = r.get('session_id', 'unknown')
        sessions[sid]['calls'] += 1
        if r.get('is_error'):
            sessions[sid]['errors'] += 1

    # Time range
    timestamps = [r.get('ts', '') for r in records if r.get('ts')]
    time_start = min(timestamps) if timestamps else None
    time_end = max(timestamps) if timestamps else None

    # Input/output sizes
    total_input = sum(r.get('input_size', 0) for r in records)
    total_output = sum(r.get('output_size', 0) for r in records)

    # Most common model
    models = Counter(r.get('model', '') for r in records if r.get('model'))

    return {
        'total': total,
        'errors': errors,
        'error_rate': round(errors / total * 100, 1) if total else 0,
        'sidechain': sidechain,
        'main': total - sidechain,
        'tool_counts': dict(tool_counts.most_common()),
        'tool_errors': dict(tool_errors.most_common()),
        'tokens_in': tokens_in,
        'tokens_out': tokens_out,
        'total_input_bytes': total_input,
        'total_output_bytes': total_output,
        'sessions': len(sessions),
        'session_breakdown': {k: v for k, v in sessions.items()},
        'time_start': time_start,
        'time_end': time_end,
        'top_model': models.most_common(1)[0][0] if models else None,
    }


def format_compact(stats):
    """One-line summary for handover integration."""
    top_tool = max(stats['tool_counts'], key=stats['tool_counts'].get)
    return (
        f"Tool calls: {stats['total']} | "
        f"Errors: {stats['errors']} ({stats['error_rate']}%) | "
        f"Top tool: {top_tool} ({stats['tool_counts'][top_tool]}) | "
        f"Sessions: {stats['sessions']}"
    )


def format_report(stats):
    """Full terminal report."""
    lines = []
    lines.append('')
    lines.append('=' * 60)
    lines.append('  TOOL CALL STATISTICS')
    lines.append('=' * 60)
    lines.append(f'  Total calls:    {stats["total"]:,}')
    lines.append(f'  Main:           {stats["main"]:,}')
    lines.append(f'  Sidechain:      {stats["sidechain"]:,}')
    lines.append(f'  Errors:         {stats["errors"]:,} ({stats["error_rate"]}%)')
    lines.append(f'  Sessions:       {stats["sessions"]}')

    if stats['time_start']:
        lines.append(f'  Time range:     {stats["time_start"][:19]} → {stats["time_end"][:19]}')
    if stats['top_model']:
        lines.append(f'  Model:          {stats["top_model"]}')

    lines.append(f'  Tokens in:      {stats["tokens_in"]:,}')
    lines.append(f'  Tokens out:     {stats["tokens_out"]:,}')
    lines.append(f'  Input bytes:    {stats["total_input_bytes"]:,} ({stats["total_input_bytes"]/1024:.0f} KB)')
    lines.append(f'  Output bytes:   {stats["total_output_bytes"]:,} ({stats["total_output_bytes"]/1024:.0f} KB)')
    lines.append('=' * 60)

    # Tool breakdown
    lines.append('')
    lines.append('  Tool Breakdown:')
    lines.append(f'  {"Tool":<20} {"Calls":>8} {"Errors":>8} {"Rate":>8}')
    lines.append(f'  {"─" * 20} {"─" * 8} {"─" * 8} {"─" * 8}')
    for tool, count in sorted(stats['tool_counts'].items(), key=lambda x: -x[1]):
        errs = stats['tool_errors'].get(tool, 0)
        rate = f'{errs*100//count}%' if count else '0%'
        lines.append(f'  {tool:<20} {count:>8} {errs:>8} {rate:>8}')

    # Session breakdown (if multiple)
    if stats['sessions'] > 1:
        lines.append('')
        lines.append('  Session Breakdown:')
        lines.append(f'  {"Session":<40} {"Calls":>8} {"Errors":>8}')
        lines.append(f'  {"─" * 40} {"─" * 8} {"─" * 8}')
        for sid, sdata in sorted(stats['session_breakdown'].items(),
                                  key=lambda x: -x[1]['calls']):
            short_sid = sid[:36] + '...' if len(sid) > 36 else sid
            lines.append(f'  {short_sid:<40} {sdata["calls"]:>8} {sdata["errors"]:>8}')

    lines.append('')
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(
        description='Tool call statistics from telemetry store'
    )
    parser.add_argument('--store', type=str, default=None,
                        help='Path to tool-calls.jsonl')
    parser.add_argument('--session', type=str, default=None,
                        help='Filter to specific session UUID')
    parser.add_argument('--compact', action='store_true',
                        help='One-line summary (for handover integration)')
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

    records = load_records(store_path, args.session)
    stats = compute_stats(records)

    if not stats:
        print('No records found.', file=sys.stderr)
        sys.exit(1)

    if args.json:
        # Remove session_breakdown for cleaner JSON (can be large)
        output = {k: v for k, v in stats.items() if k != 'session_breakdown'}
        print(json.dumps(output, indent=2))
    elif args.compact:
        print(format_compact(stats))
    else:
        print(format_report(stats))


if __name__ == '__main__':
    main()
