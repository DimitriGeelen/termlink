# T-3344 — Which registered local fixes survive a re-vendor 1.6.29 → 1.8.3?

Date: 2026-10-06. Read-only analysis. Nothing in `/opt/termlink` was changed apart from this file.

**What was compared.** The register is `/opt/termlink/.vendor-divergence.yaml`. Each entry's file was
compared between our vendored tree (`.agentic-framework/`, VERSION 1.6.29) and a clean 1.8.3 clone in
the scratchpad (`aef-1.8.3`, VERSION 1.8.3). The clone is shallow (one commit), so there is no
upstream history and no CHANGELOG to search. "Carried" was judged by reading the 1.8.3 code, its
in-code task citations, and, for the security predicate, by calling the 1.8.3 function directly.

**How patch applicability was tested.** For each entry, our commit's diff was exported with
`git show <c> --relative=.agentic-framework` and checked two ways against a scratch copy of 1.8.3:

1. `git apply --check` (strict).
2. A 3-way `git merge-file`: base = our file at `<c>^`, theirs = our file at `<c>`, ours = 1.8.3.
   The figure reported is the conflict count.

The scratch repo was `git init`ed first. Without that, `git apply` silently skips every patch,
because `/` is itself a git repo on this host.

## Table

| # | Entry / task | File | Register status | Verdict | Evidence | Re-apply effort |
|---|---|---|---|---|---|---|
| 1 | T-3336 parallel embed | web/embeddings.py | local-only | **NOT-CARRIED** | 1.8.3 has no `FW_EMBED_WORKERS`/`ThreadPoolExecutor`; `_flush` (l.1199) is still serial | Trivial: `git apply --check` clean |
| 2 | T-3336 ask import path | lib/ask.py | local-only | CARRIED | 1.8.3 l.21-27 put FRAMEWORK_ROOT on sys.path, citing "T-3783, 010 T-3336" | None. Drop ours |
| 3 | T-3336 reindex PYTHONPATH + exit 2 | bin/fw | local-only | CARRIED | 1.8.3 `fw index reindex` sets `PYTHONPATH="$FRAMEWORK_ROOT…"` and `sys.exit(2)` on unimportable (T-3783). T-3860 adds signal forwarding | None |
| 4 | T-3270 focus-drift helper exception | agents/context/check-active-task.sh | filed-upstream | **NOT-CARRIED** | no `commit-pending` anywhere in 1.8.3 | Trivial: clean apply |
| 5 | T-3269 handover runs commit-pending | agents/handover/handover.sh | filed-upstream | **NOT-CARRIED** | no `commit-pending` in 1.8.3 handover.sh or the resume template | Trivial: clean apply |
| 6 | T-3198 external reviewer PASS closes low-risk task | update-task.sh (+ lib/reviewer/static_scan.py) | filed-upstream | RESTRUCTURED | Upstream solved this independently with the T-3579/T-3581 verdict ledger (`lib/verdict_ledger.py`, `apply_reviewer_verdicts`). An independent reviewer's GREEN verdict ticks REVIEWER-JUDGES criteria, backed by HMAC-signed dispatch provenance. `external_reviewer_clears_sovereignty` is absent | **Do not re-apply.** Strict apply fails on update-task.sh; static_scan.py hunk applies but is moot. Practice migrates to `fw reviewer verdict record` |
| 7 | T-3189 derive blast_radius from body | agents/termlink/bvp-estimator/estimator.py | filed-upstream | **NOT-CARRIED** | 1.8.3 `score_blast_radius` (l.2888) still has 2 tiers (inception `target_blast_radius`, then components) | Manual: strict fails, 3-way 1 conflict. Needs T-3188 first |
| 8 | T-3188 extend target_blast_radius to all types | same | filed-upstream | **NOT-CARRIED** | the fallback is still gated on `wf == "inception"` and precedes components (we put components first) | Manual: 1 conflict, small |
| 9 | T-3187 /dev/null redirect is not a write | agents/context/lib/safe-commands.sh | filed-upstream | CARRIED | 1.8.3 `has_bash_write_pattern` strips `/dev/null` sinks with a terminator boundary (T-3643, "ported from 055"). Measured: `ls 2>/dev/null` gives no; `> /dev/nullish` gives WRITE | None, but see #12 |
| 10 | T-3185 all-no-signal tasks excluded from quadrants | lib/bvp.sh | filed-upstream | **NOT-CARRIED** | no `_proposal_is_all_no_signal` or equivalent. Upstream adds adjacent controls (`lib/bvp_degenerate.py` corpus alarm, degenerate-median quadrant tie, `bvp_judge` no-signal contradiction) that do not exclude them | Manual: 3-way 2 conflicts |
| 11 | T-3184 BVP scoring needs no human; sticky override | lib/bvp.sh | filed-upstream | PARTIAL | `confirm` is ungated by default (T-3487, opt back in with `FW_REQUIRE_BVP_CONFIRM_APPROVAL=1`) and human-confirmed scores are sticky (T-3523, `lib/bvp_sticky.py`). Still §ACD-gated: `weight --set`, `driver --add/--remove`, `auto-promote --enable`. No `BVP_HUMAN_APPROVAL` key | Manual: 3-way 4 conflicts. Re-apply only the 4-verb ungating, on top of upstream's mechanism |
| 12 | T-3178 Tier-1 write-gate bypass (`2>`/`&>` file, `sed -i` boundary, pipefail) | agents/context/lib/safe-commands.sh | filed-upstream | **NOT-CARRIED** | Measured on 1.8.3 by calling the predicate: `bin/fw audit 2> out.txt` gives **no**, `&> out.txt` gives **no** (bypass live); `grep -n sed T-2958-go-in-the.md` gives WRITE (false positive live); `echo \| grep -qE` pipes remain | Manual: 3-way 1 conflict. Rebase onto upstream's T-3643 strip and drop our T-3187 strip |
| 13 | T-3176 confirm switch + confirmation telemetry ledger | lib/bvp.sh | filed-upstream | PARTIAL | confirm-ungating is superseded by T-3487. Not carried: the `bvp-confirmations.ndjson` ledger, `FW_BVP_TELEMETRY_PATH`, `no_signal_count` | Manual: 1 conflict. Re-apply the ledger only |
| 14 | T-3127 per-session `.budget-status` | agents/context/budget-gate.sh | filed-upstream | CARRIED | T-3598 stamps `claude_session_id` (stdin `session_id`, else the transcript stem) and treats a foreign-session cache as not ours; legacy caches are trusted | None. Our fixture keys on `session_key` and will likely go red (note 4) |
| 15 | T-2982 hook-telemetry race (no local patch) | lib/hook-telemetry.sh | filed-upstream | CARRIED | T-3371 serialises the read-modify-write with flock and degrades to allow (l.35-61) | None. Move to landed-upstream (note 5) |
| 16 | T-2961 checkpoint.sh status/budget safe | agents/context/lib/safe-commands.sh | local-only | CARRIED | 1.8.3 l.572-586: `checkpoint.sh)` arm with `budget\|status` safe (upstream T-3344) | None |
| 17 | T-2882 handover narrative constants + Suggested First Action | agents/handover/handover.sh | filed-upstream | PARTIAL | Carried as part 3: focus first, then `last_update` recency (T-3210). Not carried: parts 1-2. There is no `enrichment_status: pending` in the handover frontmatter, and literal `None` and `See gaps register above.` (l.1472) remain | Manual: 3-way 2 conflicts. Re-apply parts 1-2 only |
| 18 | T-2866 fw doctor claude-fw router check | bin/fw | filed-upstream | **NOT-CARRIED** | 1.8.3 doctor (l.3161-3225, rewritten by T-3358 to scan every PATH dir) still `cmp`s each hit against the wrapper `bin/claude-fw`. `bin/claude-fw-router` exists, so a correct router install still reads as drift, now once per PATH copy | Manual: 1 conflict. Re-implement the router-shape test inside the new loop |
| 19 | T-2687 pickup fail-open (const dedup hash, empty-name inception) | lib/pickup.sh | filed-upstream | **NOT-CARRIED** | 1.8.3 `pickup_dedup_hash` (l.107-120) still hashes `"||"` when all inputs are empty, with no refusal | Trivial: clean apply (use be0fcfa68, the current form) |
| 20 | T-3039 `inception decide --follow-on` (no local patch) | lib/inception.sh | filed-upstream | **NOT-CARRIED** | no `related_tasks`/`follow-on` in 1.8.3 inception.sh | None to lose: there is no local code |
| 21 | T-3316 checkpoint `budget` alias + no crash banner on usage error | agents/context/checkpoint.sh | local-only | PARTIAL | `budget` is a native verb in 1.8.3 (l.599). The `*)` arm still `exit 1`s under `fw_hook_crash_trap`, so a wrong verb still prints HOOK CRASHED and writes to the crash log | Small: re-apply only the `*)` arm (`trap - EXIT`, usage to stderr) |

**Counts (21 entries):** CARRIED 6, PARTIAL 4, NOT-CARRIED 10, RESTRUCTURED 1.
One NOT-CARRIED entry (#20) has no local patch, so nothing is lost for it. Of the 9 NOT-CARRIED
entries that do carry a local patch, 4 apply cleanly (#1, #4, #5, #19) and 5 need manual work (#7, #8,
#10, #12, #18).

### landed-upstream entries (confirmation)

| Entry | File | Confirmed in 1.8.3? |
|---|---|---|
| T-3337 reader never builds | web/embeddings.py | Yes. `_get_db` raises `IndexUnavailable` "rebuild with: fw index reindex" (T-3786); `build_index` is atomic (`.building`, then swap) and holds a lock (T-3860) |
| T-3337 discovery no background build | web/blueprints/discovery.py | Effectively yes, with a caveat: `_trigger_async_index_build` still exists and still calls `build_index()`, but nothing in 1.8.3 calls it, and `build_index` is now atomic. Dead code, harmless |
| T-2304 update-task.sh FRAMEWORK_ROOT | agents/task-create/update-task.sh | Yes. l.764 reads `sys.argv[3] if len(sys.argv) > 3 else os.environ…` |
| T-2721 worktree-blind audit checks | agents/audit/audit.sh | Yes (superseded): `lib/hook_paths.py` / `hook_portability.py` / `doctor-hook-exercise.py`; "Cron drift checks skipped — linked worktree" at l.2738 |
| T-2469 budget-gate wrap-up deadlock | agents/context/budget-gate.sh | Yes. l.219 allowlist regex `fw\s+(handover\|context\s+focus)` (T-2702) |

## Notes

1. **Highest risk is #12 (T-3178).** On 1.8.3, `<cmd> 2> file` and `<cmd> &> file` still evade the
   Tier-1 active-task gate, and nothing shows it. Upstream did carry our *amendment* (#9, the
   `/dev/null` strip, as T-3643) but not the fix it amends. Re-apply #12 before trusting the gate
   after a re-vendor. Then run `tests/safe-commands-write-pattern-fixtures.sh`.

2. **T-3198 should not be re-applied (#6).** Upstream's verdict ledger covers the same ruling with
   stronger provenance: signed dispatch, a producer-set exclusion, and an append-only ledger. Two
   things follow:
   - Our `tests/reviewer-sovereignty-fixtures.sh` extracts `external_reviewer_clears_sovereignty`,
     so it will exit 2 after a re-vendor. Retire it or rewrite it against `verdict_ledger.py`.
   - Existing `## Reviewer Verdict` blocks carrying `Reviewer: external-dispatch` will no longer
     close tasks.

3. **The BVP stack (#10, #11, #13) overlaps heavily with upstream's own revamp.** Upstream has
   T-3487 (confirm ungated), T-3523 (sticky), T-3489 (degenerate alarm) and `bvp_judge`. Re-apply
   only the gaps: ungating the 4 policy verbs, the confirmation telemetry ledger, and the
   all-no-signal quadrant exclusion. Re-check the operator's ruling against upstream's narrower
   scope before doing so. Our `BVP_HUMAN_APPROVAL` config key does not exist upstream; the
   equivalent is the env var `FW_REQUIRE_BVP_CONFIRM_APPROVAL`, and it covers confirm only.

4. **Fixtures that will flip after a re-vendor without losing anything.**
   - `tests/budget-status-session-key-fixtures.sh` keys on our `session_key` field; upstream uses
     `claude_session_id`.
   - `tests/hook-telemetry-race-fixtures.sh` leg 1 will go red. That is the designed "fix landed"
     signal.
   - `tests/checkpoint-verb-fixtures.sh` may partly flip, because `budget` is now native.
   - `tests/safe-commands-checkpoint-fixtures.sh` should still pass.

   All of these are guard-layer members, so expect red CI that does not mean a regression.

5. **Register updates the evidence supports.**
   - Move to landed-upstream: #2, #3, #9, #14, #15, #16.
   - #6: mark superseded.
   - Mark PARTIAL rows with the residue named above.
   - #20 stays filed-upstream.

6. **Patch application method.**
   - Clean `git apply --check` results are for each commit on its own. #19's two commits
     (ed60a64ea and be0fcfa68) are alternatives, so apply be0fcfa68 only.
   - #7 must follow #8.
   - The bvp.sh patches (#13, #11, #10) stack in commit order, so the conflict counts are lower
     bounds for applying all three in sequence.

7. **Header hazard H1 is present in 1.8.3.** This is outside the question but on the checklist. The
   register's own DETECT, run against the clone's `is_commit_checkpoint_command`, gives:

   | Command | Result |
   |---|---|
   | `fw git commit -m "T-1: x"` | **BLOCK** |
   | `.agentic-framework/bin/fw git commit …` | **BLOCK** |
   | bare `git commit` | ALLOW |

   This is the BLOCK-REGRESSED state. After upgrading, this project's documented commit command
   would be refused at the commit-checkpoint exemption. H2's dead-exec targets are fine:
   `bin/watchtower.sh`, `lib/build.sh` and `metrics.sh` are 755 in the clone.

8. **Non-registered drift was not assessed.** `lib/inception.sh` differs from the 1.6.295 baseline
   by about 73 lines under no registered entry. Run `bash scripts/check-vendor-divergence.sh`
   (checklist step 1) before the re-vendor.

Scratch artefacts (patches, 3-way merge outputs, the H1 probe):
`/tmp/claude-0/-opt-termlink/e817a600-b7cc-4402-a9cf-c959eb30f655/scratchpad/divcheck/`.
