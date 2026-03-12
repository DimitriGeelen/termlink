#!/usr/bin/env python3
"""
Extract tool call metadata from Claude Code JSONL transcripts.

Parses main session JSONL and sidechain (sub-agent) files, outputting
metadata-only records as JSONL to stdout. Schema from T-104.

Usage:
  python3 extract-tool-calls.py                          # Current session
  python3 extract-tool-calls.py --session UUID           # Specific session
  python3 extract-tool-calls.py --task T-104             # Tag with task ID
  python3 extract-tool-calls.py --project-dir /path      # Override project root
  python3 extract-tool-calls.py --include-sidechains     # Also parse sub-agent files
  python3 extract-tool-calls.py --stats                  # Print summary stats to stderr
"""

import json
import os
import sys
import argparse
from pathlib import Path


def find_project_dir():
    """Find the Claude Code project transcript directory."""
    project_root = os.environ.get('PROJECT_ROOT') or os.getcwd()
    dir_name = project_root.replace('/', '-')
    return Path.home() / '.claude' / 'projects' / dir_name


def find_session_jsonl(project_dir, session_id=None):
    """Find the session JSONL file, either by ID or most recent."""
    if not project_dir.is_dir():
        print(f'Error: project dir not found: {project_dir}', file=sys.stderr)
        return None

    if session_id:
        path = project_dir / f'{session_id}.jsonl'
        if path.exists():
            return path
        print(f'Error: session not found: {path}', file=sys.stderr)
        return None

    candidates = [
        f for f in project_dir.iterdir()
        if f.suffix == '.jsonl' and f.stem != 'memory' and not f.name.startswith('agent-')
    ]
    if not candidates:
        print('Error: no session JSONL found', file=sys.stderr)
        return None

    return max(candidates, key=lambda f: f.stat().st_mtime)


def find_sidechain_files(project_dir, session_id):
    """Find sidechain (sub-agent) JSONL files for a session."""
    subagents_dir = project_dir / session_id / 'subagents'
    if not subagents_dir.is_dir():
        return []
    return sorted(subagents_dir.glob('agent-*.jsonl'))


def extract_tool_calls_from_file(path, task=None, is_sidechain=False):
    """
    Extract tool call metadata from a single JSONL file.

    Pairs tool_use blocks (in assistant events) with their corresponding
    tool_result blocks (in user events) via tool_use_id matching.
    """
    # First pass: collect all tool_use and tool_result blocks
    tool_uses = {}  # id -> {tool, input_size, ts, model, tokens_in, tokens_out, session_id, agent_id, cwd}
    tool_results = {}  # tool_use_id -> {is_error, output_size, error_summary}

    with open(path, errors='ignore') as f:
        for line in f:
            try:
                event = json.loads(line)
            except (json.JSONDecodeError, ValueError):
                continue

            event_type = event.get('type')
            ts = event.get('timestamp', '')
            session_id = event.get('sessionId', '')
            cwd = event.get('cwd', '')

            if event_type == 'assistant':
                message = event.get('message', {})
                model = message.get('model', '')
                usage = message.get('usage', {})
                tokens_in = usage.get('input_tokens', 0)
                tokens_out = usage.get('output_tokens', 0)
                agent_id = event.get('agentId')

                content = message.get('content', [])
                if not isinstance(content, list):
                    continue

                for block in content:
                    if not isinstance(block, dict):
                        continue
                    if block.get('type') != 'tool_use':
                        continue

                    tool_id = block.get('id', '')
                    tool_name = block.get('name', '')
                    tool_input = block.get('input', {})
                    input_str = json.dumps(tool_input) if tool_input else ''

                    tool_uses[tool_id] = {
                        'ts': ts,
                        'session_id': session_id,
                        'tool': tool_name,
                        'input_size': len(input_str),
                        'model': model,
                        'tokens_in': tokens_in,
                        'tokens_out': tokens_out,
                        'is_sidechain': is_sidechain,
                        'agent_id': agent_id,
                        'cwd': cwd,
                    }

            elif event_type == 'user':
                content = event.get('message', {}).get('content', [])
                if not isinstance(content, list):
                    continue

                for block in content:
                    if not isinstance(block, dict):
                        continue
                    if block.get('type') != 'tool_result':
                        continue

                    tool_use_id = block.get('tool_use_id', '')
                    is_error = bool(block.get('is_error', False))
                    result_content = block.get('content', '')
                    if isinstance(result_content, list):
                        result_content = json.dumps(result_content)
                    elif not isinstance(result_content, str):
                        result_content = str(result_content)

                    error_summary = None
                    if is_error:
                        error_summary = result_content[:200]

                    tool_results[tool_use_id] = {
                        'is_error': is_error,
                        'output_size': len(result_content),
                        'error_summary': error_summary,
                    }

    # Join: emit one record per tool_use, enriched with result data
    records = []
    for tool_id, use_data in tool_uses.items():
        result_data = tool_results.get(tool_id, {})
        record = {
            'ts': use_data['ts'],
            'session_id': use_data['session_id'],
            'task': task,
            'tool': use_data['tool'],
            'tool_use_id': tool_id,
            'is_error': result_data.get('is_error', False),
            'error_summary': result_data.get('error_summary'),
            'input_size': use_data['input_size'],
            'output_size': result_data.get('output_size', 0),
            'model': use_data['model'],
            'tokens_in': use_data['tokens_in'],
            'tokens_out': use_data['tokens_out'],
            'is_sidechain': use_data['is_sidechain'],
            'agent_id': use_data['agent_id'],
            'cwd': use_data['cwd'],
        }
        records.append(record)

    # Sort by timestamp
    records.sort(key=lambda r: r.get('ts', ''))
    return records


def print_stats(records, file=sys.stderr):
    """Print summary statistics."""
    total = len(records)
    errors = sum(1 for r in records if r['is_error'])
    sidechain = sum(1 for r in records if r['is_sidechain'])
    tools = {}
    for r in records:
        tools[r['tool']] = tools.get(r['tool'], 0) + 1

    print(f'\n--- Tool Call Extraction Stats ---', file=file)
    print(f'Total calls: {total}', file=file)
    print(f'Errors: {errors} ({errors*100//total if total else 0}%)', file=file)
    print(f'Sidechain: {sidechain}', file=file)
    print(f'Tool breakdown:', file=file)
    for tool, count in sorted(tools.items(), key=lambda x: -x[1]):
        print(f'  {tool}: {count}', file=file)
    total_bytes = sum(len(json.dumps(r)) for r in records)
    print(f'Output size: {total_bytes:,} bytes ({total_bytes/1024:.1f} KB)', file=file)


def main():
    parser = argparse.ArgumentParser(
        description='Extract tool call metadata from Claude Code JSONL transcripts'
    )
    parser.add_argument('--session', type=str, default=None,
                        help='Session UUID (default: most recent)')
    parser.add_argument('--task', type=str, default=None,
                        help='Task ID to tag records with (e.g. T-104)')
    parser.add_argument('--project-dir', type=str, default=None,
                        help='Override Claude project transcript directory')
    parser.add_argument('--include-sidechains', action='store_true',
                        help='Also extract from sub-agent sidechain files')
    parser.add_argument('--stats', action='store_true',
                        help='Print summary stats to stderr')
    args = parser.parse_args()

    if args.project_dir:
        project_dir = Path(args.project_dir)
    else:
        project_dir = find_project_dir()

    session_path = find_session_jsonl(project_dir, args.session)
    if not session_path:
        sys.exit(1)

    session_id = session_path.stem
    print(f'Extracting from: {session_path.name}', file=sys.stderr)

    # Extract from main JSONL
    all_records = extract_tool_calls_from_file(
        session_path, task=args.task, is_sidechain=False
    )
    print(f'Main: {len(all_records)} tool calls', file=sys.stderr)

    # Extract from sidechains if requested
    if args.include_sidechains:
        sidechain_files = find_sidechain_files(project_dir, session_id)
        for sc_path in sidechain_files:
            sc_records = extract_tool_calls_from_file(
                sc_path, task=args.task, is_sidechain=True
            )
            all_records.extend(sc_records)
            if sc_records:
                agent_id = sc_records[0].get('agent_id', sc_path.stem)
                print(f'Sidechain {agent_id}: {len(sc_records)} tool calls', file=sys.stderr)

    # Sort all by timestamp
    all_records.sort(key=lambda r: r.get('ts', ''))

    if args.stats:
        print_stats(all_records)

    # Output JSONL to stdout
    for record in all_records:
        print(json.dumps(record, ensure_ascii=False))


if __name__ == '__main__':
    main()
