# T-3489: upgrade vendored AEF to v1.8.13 — read-only preparation

Prepared 2026-10-20. Nothing under `.agentic-framework/`, `CLAUDE.md` or `.vendor-divergence.yaml` was touched.

## 1. Versions

1. Vendored now: `1.8.10+516.g5c886336b` (`.agentic-framework/VERSION`). Last vendor event: commit `570c2e262`, 2026-10-17 (T-3434).
2. Target: AEF source repo `VERSION` = `1.8.13`.
3. Ancestry check (checklist step 0) has NOT been done. It needs a git clone of the AEF repo and `git merge-base --is-ancestor 5c886336b <target>`. The T-3434 lesson applies: a newer version string can be older code (v1.8.10 lacked 227 of our commits). The `upgrade` run must be from a full clone, and the target commit must descend from `5c886336b`.

## 2. Divergence check (verbatim)

`bash scripts/check-vendor-divergence.sh` → rc 0:

```
check-vendor-divergence: 2 commit(s) touch vendored code since 570c2e262, all registered
```

`--json` → rc 0, `ok: true`, `baseline: 570c2e262`, `scanned: 2`, `unregistered_count: 0`, `unregistered: []`. 46 task ids registered (T-2015 … T-3472).

## 3. Divergence table (19 not yet landed upstream)

Evidence limits: the Bash tool refuses commands that name the AEF repo path (T-559 boundary hook), so I could only use `Read` on AEF files, with no grep. I read `lib/pickup.sh` in full and the first 934 of 1667 lines of `bin/claude-fw`. All other rows are "unknown": they need a grep or diff, which the next session can do after the upgrade rehearsal. The `.fwvendor-preserve.yaml` mechanism (1.8.5+) helps here: `fw vendor` keeps our copy of each listed file and reports upstream changes.

| # | Task | File | Register status | In 1.8.13? | Evidence |
|---|------|------|-----------------|-----------|----------|
| 1 | T-3472 | lib/arc.sh | filed-upstream (pickup 332) | unknown | Read first 5 lines only; `_arc_persist_scoring` not searched for |
| 2 | T-3464 | bin/claude-fw | filed-upstream (326) | unknown, leaning not carried | no `TERMLINK_SIDECAR_SOCKET` in lines 1–934; the injection code may sit in the unread remainder |
| 3 | T-3337 | web/blueprints/discovery.py | local-only | unknown | not read |
| 4 | T-3336 | web/embeddings.py | local-only (parallel embedding) | unknown | not read |
| 5 | T-3270 | agents/context/check-active-task.sh | filed-upstream | unknown | not read |
| 6 | T-3269 | agents/handover/handover.sh | filed-upstream | unknown | not read |
| 7 | T-3189 | agents/termlink/bvp-estimator/estimator.py | filed-upstream | unknown | not read |
| 8 | T-3188 | same file | filed-upstream | unknown | not read |
| 9 | T-3185 | lib/bvp.sh | filed-upstream | unknown | not read |
| 10 | T-3184 | lib/bvp.sh | filed-upstream | unknown | not read |
| 11 | T-3178 | agents/context/lib/safe-commands.sh | filed-upstream | unknown | not read |
| 12 | T-3176 | lib/bvp.sh | filed-upstream | unknown | not read |
| 13 | T-2882 | agents/handover/handover.sh | filed-upstream | unknown | not read |
| 14 | T-2866 | bin/fw | filed-upstream | unknown | not read |
| 15 | T-2687 | lib/pickup.sh | filed-upstream | probably not carried | upstream `pickup_dedup_hash` still hashes `type|summary|source_project` from greps; I found no change aimed at the "constant dedup hash" fail-open (not byte-diffed against our copy) |
| 16 | T-3039 | lib/inception.sh | filed-upstream | unknown | not read |
| 17 | T-3316 | agents/context/checkpoint.sh | local-only | register says upstream has a real `budget` verb; the trap-on-usage-error half is unconfirmed | `.fwvendor-preserve.yaml` lists it; take upstream's `budget`, re-check the usage-error `trap - EXIT` |
| 18 | T-3370 | agents/context/lib/safe-commands.sh (`/dev/stdout`,`/dev/stderr` strip) | local-only | unknown | not read |
| 19 | T-3370 | web/blueprints/discovery.py (`_db_opened_at`) | local-only | unknown | not read |

Rows 3/19 and 11/18 are the same files as other rows, so 15 distinct files. The 9 files in `.fwvendor-preserve.yaml` are: check-active-task.sh, checkpoint.sh, safe-commands.sh, handover.sh, estimator.py, lib/bvp.sh, lib/pickup.sh, discovery.py, embeddings.py. **Not in the manifest although registered as at-risk: `lib/arc.sh` (T-3472), `bin/claude-fw` (T-3464), `bin/fw` (T-2866), `lib/inception.sh` (T-3039).** A plain `fw upgrade` will overwrite those four with upstream; whether that is acceptable depends on rows 1, 2, 14 and 16 above. Either add them to the manifest before the upgrade, or accept the loss knowingly (T-3464 breaks the W1 join for fleet agents on termlink ≥ 0.x per the register).

The 11 `landed-upstream` entries need no action, but each one that is still in `.fwvendor-preserve.yaml` would keep our stale copy (the register's `preserve_manifest` note). Currently none of the 9 manifest files is marked landed-upstream, so no removals are due.

## 4. CLAUDE.md loss risk

1. `## Core Principle` is now at line **2822** of 3686 (the register's 1650 is stale). Upgrade replaces lines 2822–3686 (865 lines) wholesale; lines 1–2821 survive.
2. I scanned the 865 lines for TermLink-specific text (`termlink`, task ids T-2xxx/T-3xxx, `/opt/`, ring20, runme). Hits are only template content: the "Operator Commands Ship as ONE runme.sh Line" directive (T-3675), "AC Classification Guidance" and a few framework-internal references. I found **no project-specific lines below the boundary** that would be lost. The Quick Reference table there is the framework template's.
3. Mitigation regardless: `cp CLAUDE.md CLAUDE.md.prevendor` first, and diff afterwards. The pre-upgrade file has the same project half, so the diff should show only template changes.

## 5. Behaviour change: `cd X || git status` refused

`grep -nE 'cd [^;&|]+\|\| *git'` over `scripts/` and `.claude/` → **0 hits**. Consistent with the earlier worker's finding. Not checked: `runme.sh`, `.context/cron/*.crontab`, `tests/`, `docs/` snippets that operators paste.

## 6. Ordered upgrade procedure

1. Confirm a clean state for vendored code: `git status --short .agentic-framework` empty, and `bash scripts/check-vendor-divergence.sh` rc 0 (it is today).
2. Run `bash scripts/check-task-id-collisions.sh` (CLAUDE.md says to run it before a merge).
3. Ancestry: in a full AEF clone, `git merge-base --is-ancestor 5c886336b <1.8.13 commit>` must succeed. If not, stop and escalate; do not upgrade by version string.
4. Resolve the table's unknowns for rows 1, 2, 14, 16 (the four files not in the preserve manifest) and decide: add to `.fwvendor-preserve.yaml`, or accept loss. For rows 5–13, 15, 17–19 (in the manifest) read the "upstream changes reported" section of the upgrade output.
5. `cp CLAUDE.md CLAUDE.md.prevendor` (keep it out of commits).
6. Run the upgrade from the full clone (rehearse first, as in T-3434): `.agentic-framework/bin/fw upgrade` with the clone as source. Do not run it from this report's session; it is an operator-approved step.
7. `diff CLAUDE.md.prevendor CLAUDE.md`: expect changes only below the `## Core Principle` line. Re-apply anything above it that changed.
8. Re-check `mode-only` exec bits (register step 6): `git diff --summary` for mode changes, and `bash scripts/check-framework-tracking-drift.sh`.
9. Re-run `bash scripts/check-vendor-divergence.sh`; update `last_vendor_event` (commit, date, description, 1.8.13), flip any carried rows to `landed-upstream` and remove them from `.fwvendor-preserve.yaml`.
10. Run the register's DETECT: lines (hazards H1…), and the guard layer: `bash scripts/run-guard-layer.sh` (about 16 minutes).
11. Run again a day later (register step 9).
12. Commit under T-3489 with `fw git commit`; push only to OneDev.

## 7. Recommendation

Conditional go. The two blockers are unresolved: the ancestry check and the four registered-but-unpreserved files (`arc.sh`, `claude-fw`, `bin/fw`, `inception.sh`), so add those to the preserve manifest (or confirm upstream carries them) before running the upgrade. CLAUDE.md is safe (no project-specific content below line 2822), and the `cd X || git status` change has no hits in `scripts/` or `.claude/`.

## 8. Outcome (2026-10-20, operator ruling A)

**Carry status** — from AEF's own carry map (AEF T-4463, msg 78e60ff5, evidence = commits in v1.8.13's history), plus one correction measured here:

| # | Fix | v1.8.13 | Action |
|---|---|---|---|
| 1 | T-3472 lib/arc.sh | not carried | preserved |
| 2 | T-3464 bin/claude-fw | not carried; conflicts with T-4249 | preserved, 3-way merged (both kept) |
| 3, 19 | T-3337, T-3370 discovery.py | not carried | preserved, 3-way merged clean |
| 4 | T-3336 embeddings.py | not carried | preserved |
| 5 | T-3270 check-active-task.sh | not carried | preserved, 3-way merged clean |
| 6 | T-3269 handover.sh | not carried | preserved |
| 7 | T-3189 estimator.py | not carried | preserved |
| 8 | T-3188 estimator.py | superseded (51ec84ea3) | landed-upstream |
| 9 | T-3185 lib/bvp.sh | not carried | preserved |
| 10, 12 | T-3184, T-3176 lib/bvp.sh | partly carried | preserved (uncovered halves) |
| 11, 18 | T-3178, T-3370 safe-commands.sh | partly carried | preserved, 3-way merged (one conflict; both kept) |
| 13 | T-2882 handover.sh | partly carried | preserved |
| 14 | T-2866 bin/fw | superseded (80499503b) | upstream taken |
| 15 | T-2687 lib/pickup.sh | not carried | preserved, 3-way merged clean |
| 16 | T-3039 lib/inception.sh | map said not carried; **ours is byte-identical to v1.8.13** | landed-upstream, unpreserved |
| 17 | T-3316 checkpoint.sh | not in 1.8.13 (fixed in 1.8.14 pre-releases) | preserved, 3-way merged clean |

**Done:** ancestry verified; rehearsed on a throwaway clone; upgrade b0b2348b2; register baseline bd1082a3e;
follow-ups b1c7ffd96 (two new upstream write rules as herestrings, fixture-caught) and 0ca79c3b5
(budget-ladder guard reads `lib/budget_ladder.py`). Not-carried fixes proposed back to AEF at
`framework:pickup` offsets 341–350 (AEF triage T-4472). Guard layer after the upgrade: every FAIL-tier red explained
(write-pattern and budget-ladder fixtures fixed; standing-controls red by design until PH-2).
