# T-3703 mutants (FP 3.1.b)

Runners: `crates/termlink-session/tests/mutants/run-t3703a.py` (slice A), `run-t3703b.py [id ...]` (slice B), `run-t3703c.py [id ...]` (slice C). Each applies one change, runs the slice's tests, restores the file; a compile error is not a kill.

## Slice A (AL-20, `termlink-session::signed_state`) — 6/6 killed, 2026-11-18

| mutant | what | verdict |
|---|---|---|
| m1-unsorted-members | verify over the parse order, not the canonical order (E-42.5) | killed |
| m2-accept-fraction | a fraction or exponent is not refused as a number (V-NEG-1) | killed |
| m3-keep-duplicate | a duplicate key is kept instead of refused (V-NEG-2) | killed |
| m4-bare-digest | sign and verify over the bare digest, no domain prefix (V-NEG-3) | killed |
| m5-trust-written-digest | the written digest is trusted, not recomputed (FCW-89) | killed |
| m6-range | an integer above 2^53-1 is accepted (V-NEG-4) | killed |

6/6 killed

## Slice B (hub revocation list) — 11/11 killed, 2026-11-18 (after review round 2; b9–b11 added for the r2 fixes)

In the first run b5 as first written did not compile (not a kill); it was rewritten (`Ok(_) => return None`) and is killed here.

| mutant | what | verdict |
|---|---|---|
| b1-no-row-before | hub.revoke runs without its intent row (E-167.5 / FCW-91) | killed |
| b2-unsigned-request | a self-request is not verified under the key it revokes (E-43 / FCW-90) | killed |
| b3-not-idempotent | re-revoking a listed key issues a new seq | killed |
| b4-unchained | prev_digest does not chain the versions | killed |
| b5-no-post-check | the hub post check ignores the revocation list (design 5.8) | killed |
| b6-restore-not-blocked | a list behind the committed log is served (design 5.7) | killed |
| b7-no-card-needed | a self-request without a key card is accepted with the request's names (r1 finding 8) | killed |
| b8-write-failure-ignored | a failed list write is reported as success (FCW-92) | killed |
| b9-post-check-ignores-reconcile | while reconciling, the post check misses keys the restore lost (r2 OC 1) | killed |
| b10-reissue-chains-to-file | a reissue chains to the restored file, not the committed head (r2 CC 5) | killed |
| b11-torn-row-ignored | a torn committed row does not block (r2 OC 6) | killed |

11/11 killed

## Slice C (sidecar revocation pull) — 18/18 killed, 2026-11-18 (after review round 2; c12–c18 added, c3 retargeted)

| mutant | what | verdict |
|---|---|---|
| c1-failed-pull-empty | a failed pull is treated as an empty list (E-41.5, 04 6.2.6) | killed |
| c2-any-seq | a lower seq is accepted (E-40.5) | killed |
| c3-no-receive-check | the receiver does not check the held list (E-39.5) | killed |
| c4-leave-stored | a message stored before the revocation is left STORED (E-39.6) | killed |
| c5-no-pin-check | the hub key is used without checking it against the pin | killed |
| c6-stale-reset | a failed pull resets the stale timer (E-41 59:59 / 60:01) | killed |
| c7-start-verified | a started sidecar is VERIFIED from its store (FCW-96) | killed |
| c8-accept-fork | the same seq with another digest is accepted | killed |
| c9-no-hub-id | a head naming another hub is accepted | killed |
| c10-no-load-verify | the stored list is trusted on load (r1 M1) | killed |
| c11-no-chain | a list that does not chain at held + 1 is accepted | killed |
| c12-handle-before-head | messages are handled before the head is read (design 5.4, r2 OC 10) | killed |
| c13-outage-not-counted | a pass that cannot reach the hub is not a failed pull (r2 CC 1) | killed |
| c14-no-stale-gate | the stale state does not narrow urgent (DR-8 A, r2 CC 3) | killed |
| c15-no-fallback | with no snapshot, nothing is revoked (r2 CC 9) | killed |
| c16-raise-all | the entry names every key, not the new ones (r2 CC 10) | killed |
| c17-no-heal | an unverifiable stored list keeps its floor and keys (r2 CC 7) | killed |
| c18-sweep-open-attempt | the sweep refuses a message whose attempt is open (r2 CC 9) | killed |

18/18 killed
