# T-3185 AC1 — close-readiness review (rung-3, reviewed revision 4484e2f55)

## Verification section (run under `bash -c 'set -eo pipefail; ...'`)
- `bash tests/bvp-no-signal-ranking-fixtures.sh` -> Pass: 10, Fail: 0, rc 0. Case 6 recovered pre-change
  code from git (e106b99f6^), its control leg ran, and that copy DID give the all-no-signal stub quadrant 'hv-lc'.
- `bash scripts/check-vendor-divergence.sh` -> "0 commit(s) touch vendored code since 034701160, all registered", rc 0.

## Agent ACs, each checked against code/history
1. Divergence registered before change: `.vendor-divergence.yaml` T-3185 entry ("Registered BEFORE the change",
   commit pending-same-commit, landed in e106b99f6 together with the code); re-application after the 1.8.3
   re-vendor is recorded (4685184d2). The fix is now carried upstream as T-4486 (vendored lib/bvp.sh markers).
2. No quadrant + excluded from medians: `.agentic-framework/lib/bvp.sh` `_ranked` filter feeds both bvp_vals and
   cost_vals; rows with all_no_signal get quadrant '-'. Live check: `fw bvp --include-proposed` shows T-3423
   (every driver "(no-signal)") with QUAD '-'.
3. No score changes: the exclusion only sets quadrant; bvp_raw/bvp_norm untouched (fixture Case 4: identical
   totals 42 across three fixtures after one is excluded). Live T-3423 still shows its BVP 140 / 0.80.
4. Counted and reported before the table: live run prints "NOTE: 40/248 task(s) (16%) scored every driver
   no-signal ... UNASSESSED, not low-value" with the full ID list, before the table; threshold note reports
   the actual population (103).
5. Unreadable rationale fails toward inclusion: `_proposal_is_all_no_signal` returns False on absent proposal,
   non-dict entry, empty scores, absent/empty rationale (fixture Case 3 passes).
6. Before/after measured on real corpus: recorded in commit e106b99f6 message (before: hv-lc topped by six
   identical stubs T-2422.. at 70/0.40/3.6; after: T-2938/T-2939/T-2940; 35/247 excluded).
7. Fixtures fail against pre-change code with control leg: Case 6 (above); lookup hardened in 7ed17ee73.
8. Filed upstream: register `filed_at` = framework:pickup offset 204, read-back byte-identical (e902988b7);
   upstream subsequently landed it as T-4486, which is stronger than read-back.

## Observations (non-blocking)
- In the unfiltered `fw bvp --include-proposed` table, excluded tasks still sort by BVP and so appear at the
  top with QUAD '-'; the AC scope (no quadrant, disclosure) is met, and the quadrant-based work-this-first
  view no longer contains them. Worth noting if the operator reads the raw table as the recommendation.
- The task file body no longer carries the original Context/Updates history (only AC ticks); evidence lives in
  git commits e106b99f6, e902988b7, 7ed17ee73 and the divergence register.

Verdict: green.
