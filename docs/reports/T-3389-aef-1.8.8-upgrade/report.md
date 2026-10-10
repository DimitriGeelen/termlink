# T-3389: AEF 1.8.5 -> 1.8.8 upgrade report (2026-10-10)

Operator ruling A (2026-10-10). Procedure of T-3376. Commit `5bf63128a`; register baseline moved to it.

## 1 Source

1. Upstream tag `v1.8.8` (commit `f0fc5839`) of DimitriGeelen/agentic-engineering-framework, cloned read-only into
   the session scratchpad. Not master (AEF: "upgrade from the release, not from a working checkout").
2. `fw upgrade` run from inside the tag clone. From the consumer it auto-clones upstream master even with
   `--from-upstream`; from inside the clone it uses the clone.
3. A shallow clone was refused as `foreign-source` (the pinned 1.8.5 sha `a06dc1f` was not visible). Fixed by
   `git fetch --unshallow`, not by the override; `a06dc1f` verified as an ancestor of v1.8.8.

## 2 Rehearsal

On a disposable clone of this repo: dry run `rehearsal-dry-run.txt` (25 changes, 2 skipped seed merges, 9
PRESERVE, 16 KEPT customised templates/toolkit, no refusal), then a real run (rc 0, VERSION 1.8.8). No
`--allow-delete-locals` and no `--force`.

## 3 The real run

`upgrade-run.txt`: 24 changes, rc 0. Two preserved files changed upstream since 1.8.5 and were KEPT-LOCAL by the
upgrade: `agents/context/checkpoint.sh` (T-3989 auto-handover at budget-critical, T-4004 push retry) and
`web/embeddings.py` (T-3953 reranker). Both were 3-way merged (base = upstream at a06dc1f, ours, theirs = v1.8.8):
0 conflicts; our local lines kept; `bash -n` and `py_compile` pass; `tests/checkpoint-verb-fixtures.sh`,
`tests/safe-commands-write-pattern-fixtures.sh`, `tests/bvp-auto-confirm-fixtures.sh` all pass; checkpoint.sh
carries the new `auto-handover` subcommand.

The 17 `.upstream` template copies: none changed upstream between 1.8.5 and 1.8.8 (source files located and
diffed; `default.md.upstream` byte-identical to the v1.8.8 template); T-3376 had reviewed the 1.8.5 versions;
removed.

## 4 Checks against the pre-upgrade baseline

| Check | Before | After |
|---|---|---|
| CLAUDE.md project section (above `## Core Principle`) | copy taken | unchanged; governance 844 -> 865 lines |
| `fw doctor` | 0 failures, 10 warnings (Watchtower smoke 10/53) | 0 failures, 9 warnings (Watchtower smoke 3/53; claude-fw drift warning gone) |
| Guard layer (192 members) | 8 red | 8 red; new red `check-framework-tracking-drift.sh` (the new upstream file `lib/upgrade_template_shipped.py` before commit; clean after commit), `notify-wake-supervisor-fixtures.sh` green |
| `scripts/check-vendor-divergence.sh` | 0 unregistered | 0 unregistered against the new baseline |
| Receiver probe to 010-termlink | | RECEIVED, and surfaced through the hook |

## 5 Known-hazard DETECT lines (.vendor-divergence.yaml step 8)

H2..H5: no finding. **H1 (832, framework:pickup 149/151): finding present** — `is_commit_checkpoint_command`
admits bare `git commit` and blocks `fw git commit` at budget-critical. Not caused by this upgrade:
`safe-commands.sh` is preserved locally and upstream's copy is byte-identical in 1.8.5 and 1.8.8; the same check
against the v1.8.8 tag reports the same. Reported to AEF (conversation aef-release-v1.8.7). Register step 9
asks for a re-run a day later.
