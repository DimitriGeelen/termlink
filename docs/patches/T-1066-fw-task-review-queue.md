# T-1066 — Upstream framework patch: `fw task review-queue`

**Status:** Applied locally to `.agentic-framework/bin/fw` (vendored, gitignored). Needs upstream propagation to the framework repo at `DimitriGeelen/agentic-engineering-framework` so it survives `fw upgrade`.

**Tracks:** T-1066 (build), G-008 (concern — 61 tasks stuck in partial-complete)

## Insertion point

In `bin/fw`, inside the `task)` subcommand dispatch, **insert the `review-queue)` case immediately before the `""|help|-h|--help)` case**. Also add three lines to the help output (shown below).

## Added subcommand block

```bash
        review-queue)
            # T-1066: List partial-complete tasks awaiting human signature (G-008 mitigation)
            FW_REVIEW_QUEUE_ARGS="$*" python3 - << 'PYTASK_REVIEW_QUEUE'
import os, re, shlex, sys
from datetime import datetime

args = shlex.split(os.environ.get('FW_REVIEW_QUEUE_ARGS', ''))
count_only = '--count' in args
rubber_only = '--rubber-stamp-only' in args

project_root = os.environ.get('PROJECT_ROOT', '.')
active_dir = os.path.join(project_root, '.tasks', 'active')

BOLD = '\033[1m'; GREEN = '\033[0;32m'; YELLOW = '\033[1;33m'
CYAN = '\033[0;36m'; DIM = '\033[2m'; NC = '\033[0m'

rows = []
if os.path.isdir(active_dir):
    for fn in sorted(os.listdir(active_dir)):
        if not fn.endswith('.md'):
            continue
        with open(os.path.join(active_dir, fn)) as f:
            text = f.read()
        fm = {}
        if text.startswith('---'):
            try:
                end = text.index('---', 3)
                import yaml
                fm = yaml.safe_load(text[3:end]) or {}
            except Exception:
                pass
        if fm.get('status') != 'work-completed':
            continue
        if '### Human' not in text:
            continue
        hm = re.search(r'### Human\s*\n(.*?)(?=\n### |\n## |\Z)', text, re.DOTALL)
        if not hm:
            continue
        human_block = hm.group(1)
        total = len(re.findall(r'^\s*-\s*\[[ xX]\]', human_block, re.MULTILINE))
        checked = len(re.findall(r'^\s*-\s*\[[xX]\]', human_block, re.MULTILINE))
        unchecked = total - checked
        if unchecked <= 0:
            continue
        has_rubber = '[RUBBER-STAMP]' in human_block
        has_review = '[REVIEW]' in human_block
        if has_rubber and not has_review: tag = 'RUBBER-STAMP'
        elif has_review and not has_rubber: tag = 'REVIEW'
        elif has_rubber and has_review: tag = 'MIXED'
        else: tag = 'UNTAGGED'
        if rubber_only and tag != 'RUBBER-STAMP':
            continue
        date_finished = fm.get('date_finished') or fm.get('last_update') or ''
        rows.append({
            'id': fm.get('id', fn[:6]),
            'name': str(fm.get('name', ''))[:55],
            'owner': fm.get('owner', '?'),
            'unchecked': unchecked,
            'total': total,
            'tag': tag,
            'date_finished': str(date_finished),
        })

rows.sort(key=lambda r: r['date_finished'] or '9999')

if count_only:
    print(len(rows))
    sys.exit(0)

if not rows:
    msg = 'No tasks awaiting human verification'
    if rubber_only: msg += ' (--rubber-stamp-only filter applied)'
    print(f'{GREEN}{msg}{NC}')
    sys.exit(0)

print(f'{BOLD}Review Queue{NC}  {CYAN}{len(rows)} task(s) awaiting human signature{NC}')
if rubber_only:
    print(f'{DIM}  filter: --rubber-stamp-only{NC}')
print()

tag_color = {'RUBBER-STAMP': GREEN, 'REVIEW': YELLOW, 'MIXED': YELLOW, 'UNTAGGED': DIM}
for r in rows:
    c = tag_color.get(r['tag'], NC)
    age = ''
    if r['date_finished'] and r['date_finished'].startswith('20'):
        try:
            d = datetime.fromisoformat(r['date_finished'].replace('Z', '+00:00'))
            days = (datetime.now(d.tzinfo) - d).days
            age = f' ({days}d old)'
        except Exception:
            pass
    print(f'  {c}[{r["tag"]:<12}]{NC} {r["id"]} [{r["owner"]}] {r["unchecked"]}/{r["total"]} — {r["name"]}{DIM}{age}{NC}')

tag_counts = {}
for r in rows:
    tag_counts[r['tag']] = tag_counts.get(r['tag'], 0) + 1
print()
print(f'{DIM}Summary: ' + '  '.join(f'{k}: {v}' for k, v in sorted(tag_counts.items())) + f'{NC}')
print(f'{DIM}Next: fw task verify T-XXX (detail) | fw task review T-XXX (QR for mobile ticking){NC}')
PYTASK_REVIEW_QUEUE
            ;;
```

## Help output — 3 added lines

In the `""|help|-h|--help)` block of `task)`:

Under "Subcommands:" (after the `review T-XXX` line), add:
```
            echo "  review-queue [--count] [--rubber-stamp-only]  List partial-complete tasks awaiting human signature (T-1066/G-008)"
```

Under "Usage:" (after the `review T-631 --poll` example), add:
```
            echo '  fw task review-queue              # Full sorted list of partial-complete tasks'
            echo '  fw task review-queue --count      # Just the count (for handover digest)'
            echo '  fw task review-queue --rubber-stamp-only  # Trivial ticks only'
```

## Testing

```bash
fw task review-queue --count          # → integer count
fw task review-queue --rubber-stamp-only   # → only RUBBER-STAMP tagged
fw task review-queue                  # → full sorted list with tags + age
fw task help | grep review-queue      # → shows help entries
```

## Observed output on 010-termlink @ commit ab2ad3d2 (2026-04-15)

61 tasks awaiting signature, sorted oldest first:
- 39 `[RUBBER-STAMP]` — trivial ticks (pattern: "Verify X in doctor / termlink list")
- 22 `[REVIEW]` — genuine human judgment (UX, architecture, subjective quality)
- Oldest: 18d old

## Why this helps (G-008 narrative)

Before: `fw task verify` (no args) lists awaiting tasks but unsorted, untagged, and mixed with no priority signal. Operators don't know which are cheap to clear.

After: sorted by age, tagged by effort-class, counts visible at a glance. A handover or digest script can call `fw task review-queue --count` to surface the backlog size. A dedicated 15-min "rubber-stamp run" session can use `--rubber-stamp-only` and work the list.

## Related

- G-008 (concern): 64 stuck tasks observed in session S-2026-0415-1917
- T-1066 (this task): ships the short-term mitigation
- Future: medium-term additions (handover digest, batch rubber-stamp command) are deferred to separate tasks
