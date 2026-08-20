# Merge-resolution scripts — `worktree-t2687-pickup-failopen` → `main`

These produced the resolution recorded in [`.context/merge-plan-t2687.md`](../../.context/merge-plan-t2687.md).

They were written in a scratch job directory, which is deleted when the session's job is
cleaned — and the merge plan told the reader to reproduce the work with them. A plan that
points at files which will not exist is the same "obligation recorded where nothing reads it"
shape this branch has been documenting all session, so they are committed here instead.

**They hardcode a scratch path.** `TRIAL=/root/.claude/jobs/<job-id>/tmp/merge-trial` appears
in most of them. Point that at your own scratch worktree before running:

```bash
git worktree add --detach /tmp/merge-trial HEAD
```

## Order

| script | does |
|---|---|
| `trial-merge.sh` | `git merge --no-commit origin/main` in the scratch tree; captures the conflicted paths |
| `classify-conflicts.sh` | per path: line counts and content hashes for both sides, so the resolution is a reading rather than a guess |
| `resolve.sh` | the mechanical `--ours` / `--theirs` calls, grouped by class with the reasoning inline |
| `union-registers.py` | collision-safe union of `learnings.yaml` / `decisions.yaml` (`--apply`) |
| `union-claudemd.py` | heading-set union of `CLAUDE.md`; **refuses to write if any heading would be lost** (`--apply`) |
| `verify-trial.sh` | fixture suites + YAML sanity on the resolved tree |
| `id-collisions-registers.py` | standalone: same-id/different-content records across two branches |

## The two worth keeping past this merge

**`union-registers.py`** — `learnings.yaml` and `decisions.yaml` allocate IDs by max+1 against
the branch's own copy, so the same ID can name two different records. That is the task-ID
defect (T-2800) in registers nothing checks. Measured on this merge: **1 learning and 10
decisions** collided with different content. Deduping on ID — the obvious implementation, and
the first one written here — silently drops 11 records. This keeps both, renumbering ours above
the combined ceiling and stamping `renumbered_from`.

**`union-claudemd.py`** — refuses to write unless the union preserves every heading from *both*
sides. Take-ours would have dropped 7 of main's sections; take-main would have dropped 8 of
ours. The refusal is the point: it cannot silently lose a section.

`id-collisions-registers.py` is the detector half and is worth running before any future
cross-branch merge, alongside `scripts/check-task-id-collisions.sh` — which covers `.tasks/`
only and is structurally blind to the registers.
