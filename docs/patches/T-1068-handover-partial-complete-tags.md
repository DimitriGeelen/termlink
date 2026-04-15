# T-1068 — Upstream framework patch: handover partial-complete section with tags + age sort

**Status:** Applied locally to `.agentic-framework/agents/handover/handover.sh` (vendored, gitignored). Needs upstream propagation to the framework repo at `DimitriGeelen/agentic-engineering-framework` so it survives `fw upgrade`.

**Tracks:** T-1068 (build), follow-up to T-1066 (`fw task review-queue`), G-008 (concern — stuck partial-complete backlog).

## Why

The existing partial-complete section in handover.sh (lines 523–566) lists awaiting tasks in `sorted(glob)` order (alphabetical by filename, not by age) and without priority tagging. When the backlog reaches 60+ tasks, reading the handover gives no signal about what's easy-to-clear (RUBBER-STAMP) vs what needs real judgment (REVIEW), and no signal about oldest-first priority. The enhancement keeps the section markdown-ready (no ANSI codes) and adds tags + sort + summary counts.

## Insertion point

Replace the entire `PARTIAL_COMPLETE_SECTION=$(python3 << 'PCEOF' ... PCEOF` heredoc in `agents/handover/handover.sh` (currently at step `2.1`) with the version below.

## Replacement block

```bash
# Step 2.1: Surface partial-complete tasks (T-372 — blind completion anti-pattern)
# Tasks that are work-completed but have unchecked Human ACs
# T-1068: enhanced — sort by date_finished ASC, tag RUBBER-STAMP/REVIEW/MIXED/UNTAGGED, summary counts
PARTIAL_COMPLETE_SECTION=$(python3 << 'PCEOF'
import glob, re, os
from datetime import datetime

tasks_dir = os.environ.get("TASKS_DIR", ".tasks")
partial = []
for f in sorted(glob.glob(os.path.join(tasks_dir, "active", "*.md"))):
    with open(f) as fh:
        content = fh.read()
    if "status: work-completed" not in content:
        continue
    human_match = re.search(r'### Human\n(.*?)(?=\n### |\n## |\Z)', content, re.DOTALL)
    if not human_match:
        continue
    human_section = human_match.group(1)
    unchecked = len(re.findall(r'^\s*-\s*\[ \]', human_section, re.M))
    if unchecked == 0:
        continue
    tid = re.search(r'^id:\s*(\S+)', content, re.M)
    tname = re.search(r'^name:\s*"?(.+?)"?\s*$', content, re.M)
    df = re.search(r'^date_finished:\s*(\S+)', content, re.M)
    if not tid:
        continue
    first_ac = re.search(r'^\s*-\s*\[ \]\s*(.+)', human_section, re.M)
    ac_preview = first_ac.group(1)[:60] if first_ac else "?"
    has_rubber = '[RUBBER-STAMP]' in human_section
    has_review = '[REVIEW]' in human_section
    if has_rubber and not has_review: tag = 'RUBBER-STAMP'
    elif has_review and not has_rubber: tag = 'REVIEW'
    elif has_rubber and has_review: tag = 'MIXED'
    else: tag = 'UNTAGGED'
    date_finished = df.group(1) if df else ''
    age_days = None
    if date_finished and date_finished.startswith('20'):
        try:
            d = datetime.fromisoformat(date_finished.replace('Z', '+00:00'))
            age_days = (datetime.now(d.tzinfo) - d).days
        except Exception:
            pass
    partial.append({
        'id': tid.group(1),
        'name': tname.group(1) if tname else "?",
        'unchecked': unchecked,
        'preview': ac_preview,
        'tag': tag,
        'date_finished': date_finished,
        'age_days': age_days,
    })

partial.sort(key=lambda r: r['date_finished'] or '9999')

if partial:
    print("## Awaiting Your Action (Human)")
    print()
    counts = {}
    for r in partial:
        counts[r['tag']] = counts.get(r['tag'], 0) + 1
    summary = ', '.join(f"{k}: {v}" for k, v in sorted(counts.items()))
    print(f"**{len(partial)} task(s) with unchecked Human ACs** ({summary}). Sorted oldest first.")
    print("Review when ready — no urgency implied. Use `fw task review-queue` for the live list.")
    print()
    for r in partial:
        age = f" ({r['age_days']}d old)" if r['age_days'] is not None else ""
        print(f"- `[{r['tag']}]` **{r['id']}**: {r['name']} ({r['unchecked']} unchecked{age})")
        print(f"  - e.g.: {r['preview']}")
    print()
PCEOF
)
```

## Observed output (010-termlink @ 2026-04-15)

```
## Awaiting Your Action (Human)

**63 task(s) with unchecked Human ACs** (REVIEW: 22, RUBBER-STAMP: 41). Sorted oldest first.
Review when ready — no urgency implied. Use `fw task review-queue` for the live list.

- `[REVIEW]` **T-789**: Worktree isolation for TermLink-dispatched agents (1 unchecked (16d old))
  - e.g.: [REVIEW] Review exploration findings and approve go/no-go de
- `[REVIEW]` **T-160**: Pickup prompt: fix declare -A macOS bash 3.2 bug (1 unchecked (11d old))
  - e.g.: [REVIEW] Paste prompt into framework Claude Code session
  ... (truncated for brevity)
```

## Behavior differences vs. prior version

| Aspect | Before | After |
|---|---|---|
| Sort order | Alphabetical (filename glob) | `date_finished` ASC — oldest first |
| Priority signal | None | `[RUBBER-STAMP]` / `[REVIEW]` / `[MIXED]` / `[UNTAGGED]` prefix |
| Summary line | "N tasks" | "N tasks (REVIEW: X, RUBBER-STAMP: Y, ...)" |
| Age indicator | None | `(Nd old)` per task |
| Cross-ref | None | Points to `fw task review-queue` |

## Compatibility

- Python-only logic, no new dependencies.
- Same markdown structure — downstream consumers reading `## Awaiting Your Action (Human)` unaffected.
- Backward-compatible: tasks without `date_finished` sort to the end; tasks without `[RUBBER-STAMP]`/`[REVIEW]` tags show `[UNTAGGED]`.

## Related

- T-1066 (shipped): `fw task review-queue` CLI — same logic exposed as a standalone command.
- G-008 (concern): 64 tasks stuck in partial-complete, no review-queue surface — this patch addresses the handover-visibility half.
- Future: a PostSessionEnd hook could auto-run a digest email; out of scope here.
