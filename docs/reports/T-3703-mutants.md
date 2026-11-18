# T-3703 mutants (FP 3.1.b)

Runners: `crates/termlink-session/tests/mutants/run-t3703a.py` (slice A), `run-t3703b.py [id ...]` (slice B). Each applies one change, runs the slice's tests, restores the file; a compile error is not a kill.

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

## Slice B (hub revocation list) — 8/8 killed, 2026-11-18

First run 7/8: b5 as first written did not compile (not a kill); rewritten to `Ok(_) => return None` and run alone: killed.

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
