# T-3366 review round 1 — dispositions

Round 1: claude-code FAIL (`review-1-claude-code.md`); opencode did not run (usage limit, `review-1-opencode.md`).
Every finding is fixed, accepted with a reason, or put to the operator. Nothing is sent round again unanswered.

| # | Finding | Disposition |
|---|---|---|
| 1a | `adopt_legacy` re-keys a migrated hub that lost its key (m5's behaviour through the deploy path) | **Fixed.** adopt-id acts only when BOTH files are absent; id-without-key and key-without-id are refused ("does not re-key" / "does not re-mint"). Tests `adopt_refuses_a_lost_key_*`, `adopt_refuses_a_lost_id_*` (the latter after a certificate rotation); mutants m13, m14. |
| 1b | adopt re-mints a lost `hub.id` from a NEW certificate | **Fixed** by 1a (a key on disk marks a migrated hub). |
| 1c | runme hides `key_generated`, prints only the first line | **Fixed.** Every adopt line is printed. |
| 1d | local "hub.id unchanged" compares the adopted id with itself | **Fixed.** Before adopt: the RUNNING hub's served id is read (`sidecar hub-id`) and passed as `--expect`; after the restart: the NEW hub's served id must equal it. Fixtures: served-id mismatch refused with nothing written; restart that changes the served id FAILED. |
| 1e | no test for id present + key missing | **Fixed** (1a tests). |
| 2a | `observe` turns a store read error into "no pin" and re-pins | **Fixed.** A read error → `Unverified`, needs-attention `hub-signing-key-pin-unreadable`, nothing written. Test `e37_unreadable_pin_is_not_replaced`; mutant m17. |
| 2b | OBS-37 `admin_row(hub-key-pinned)` not implemented | **Accepted, stated.** The sidecar has no admin-row log; the durable record is the needs-attention entry `hub-signing-key-pinned` (store `attention.jsonl`, timed). The evidence map says so. |
| 2c | pin written from `check_pin` at runtime, not only at sidecar start (2.2.f(4)) | **Accepted with reason.** Both writers are threads of the one sidecar process that owns the store (`store.lock` refuses a second sidecar); `write_state` is temp + fsync + rename; the only value either can write on an empty pin is the same offer. The spec's concern (two writers meeting) cannot produce two different pins. |
| 2d | a pinned hub that stops offering its key → silent `NotOffered` | **Fixed.** needs-attention `hub-signing-key-withdrawn` (coalesced per day); `verified_hub_key` refuses. Test `e37_withdrawn_key_is_named_and_refused`. |
| 3a | "prior state" reading resolved by the agent alone, toward the less fail-closed side | **To the operator** (one decision, after this round): the spec contradicts itself (AL-17 step 1 vs E-36.2/36.4). |
| 3b | topic-log test asserts only `is_err()` | **Fixed.** It asserts the FCW-79 message and that no file was written. |
| 3c | mint before the pidfile lock: two concurrent first starts could serve an id that lost | **Fixed.** Publish is no-clobber (`hard_link`, never `rename` over a file); the loser reads the winner's id and key. Stale-temp cleanup removes only temps whose writer pid is dead (the race test found the earlier version deleting a live writer's temp). Tests `e166_write_never_clobbers_*`, `e166_concurrent_first_starts_converge_*` (8 threads); mutant m16. |
| 4a | window between installing the new binary and step 5: a crash/auto-restart starts it without `hub.id` | **Fixed.** Step 4 runs adopt-id with the NEWLY BUILT binary BEFORE installing it; a refusal leaves the old binary in place (FAILED "not installing"). Fixtures: adopt line precedes the install line; refusal keeps the old binary. |
| 4b | key ownership vs the hub's euid not checked | **Fixed.** Local: key uid must equal the hub process's uid (`/proc/<MainPID>`); fleet: same check remotely (`RUNME-ADOPT-UID`). Fixtures for both. |
| 4c | fleet: empty served `id0` skips the comparison; a wrong id is written before the compare | **Fixed.** No served `hub_id` → FAILED, no swap. The remote adopt runs with `--expect <served id>`, so a differing id is refused before anything is written. Fixtures for both. |
| 5a | E-126 needs status `hub {id, declared, match}` and a fake-status control | **Fixed.** `status.hub {id, declared, match}` from the id each check observed. Test `e126_*` with the control (a sidecar declared for another hub reads `match: false`); mutant m18. |
| 5b | E-166 third leg cites unquoted tests | **Fixed in bundle 2** (the test names and their result lines are quoted). |
| 5c | E-37 AR-1 LIMIT run missing; SI-25 scan only of the sidecar store | **Fixed.** `e37_ar1_limit_rogue_key_at_first_contact_is_pinned` records the LIMIT; the real-hub test now runs the hub with `RUST_LOG=info`, keeps its stdout/stderr, and scans every file of the runtime dir (logs included, key file excluded) for the seed. |
| 5d | E-38 and E-163 dropped by the agent's statement | **Clarified.** E-38 is listed under plan row P-11 (T-3363) as that unit's evidence; this unit provides P-11's prerequisite. E-163 concerns `TERMLINK_*_TEST_*` seams; T-3366 adds none (its fault injector is `#[cfg(test)]`). Both stay stated, not claimed. |
| 5e | mutants ran on one commit, the deploy on another | **Fixed in bundle 2.** Mutants, fixtures and suites rerun on the round-2 commit; the redeploy runs that same commit, and the bundle quotes both hashes. |
| 5f | live `hub.version` only for the local hub | **Fixed.** `fleet doctor --json` now reports `hub_id` and `signing_key` per hub; bundle 2 quotes all four. |
