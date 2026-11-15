# T-1885 Human AC#1 — evidence report (rung-5 panel seat claude-code)

Revision reviewed: 1f4738a8cff03478f4e8c1b53fd054e567fb3785. Verdict: **red**.

## Step 1 — `fw independent-review --dry-run`
- `fw` resolves to `/opt/termlink/.agentic-framework/bin/fw`. Output, exit 1:
  `Unknown command: independent-review` / `Run 'fw help' for usage`.
- `fw help` lists no `independent-review` verb; `grep independent-review .agentic-framework/bin/fw` finds nothing;
  `.agentic-framework/agents/independent-review/` does not exist.
- The command as written therefore does not run. Only `scripts/independent-review.py` exists (executable, 13336 bytes).
- To judge the rest, I ran `python3 scripts/independent-review.py --dry-run` (exit 0, 158 lines).
  Full output: `dryrun-script-output-<my dispatch id>.log` in this directory.
  Summary: processed=141; PASS-ROBUST 1, PASS-LOOSE 28, FAIL 5, INCONCLUSIVE 12, SURFACE 95.

## Step 2 — is each PASS line backed by evidence visible in the task's Updates block?
No, for all 29 PASS lines.
- The writer (`append_to_updates`, script line 133) adds `### <ts> — independent-review [<verdict>] [agent]` entries,
  but only when not in `--dry-run` (line 297).
- `grep -rl "independent-review \[" .tasks/active .tasks/completed` returns 0 files: no live run has ever written an entry.
- Spot-checked T-1415 (PASS-ROBUST), T-1430, T-1489, T-3717 (PASS-LOOSE): zero mentions of independent-review in each file.
- `.context/working/.independent-review-state.json` does not exist.
- The dry-run output prints a reason only for FAIL lines; a PASS line shows no reason and no evidence pointer.
- A PASS-LOOSE is demonstrably not evidence of PASS: the tool rates T-1885's own AC
  (`fw independent-review --dry-run | tee ...`) PASS-LOOSE although that command fails with "Unknown command"
  (the pipe to `tee` hides the exit code).

## Step 3 — spot-check 3 PASS tasks via `fw task review`
Not run: with no evidence entries in any Updates block there is nothing to render, and `fw task review`
may write review markers outside the files I am allowed to touch. The task files are what Watchtower renders,
and they hold no independent-review evidence (step 2).

## Expected: "clear, operator-actionable, no spammy noise"
- Line format itself is scannable (marker, task id, class, verdict, title).
- Noise: 95 of 141 lines are SURFACE; 44 are the identical text "Review exploration findings and approve go/no-go decision".
- AC titles are cut at 60 characters, often mid-word (e.g. "Run `fw independent-review --dry-run` and verify the per-tas").
- 64 lines exceed 110 columns.

## What is needed
1. Wire the `fw independent-review` verb (or restate the criterion to the script path).
2. Do one live (non-dry) run so the Updates entries exist, or have dry-run print the reason/evidence for PASS lines.
3. Print the reason on PASS-LOOSE lines; fix the piped-command exit-code false positive.
4. Collapse or hide SURFACE lines by default (e.g. a count per class), so PASS lines are scannable.
