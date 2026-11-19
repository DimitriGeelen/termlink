# T-3700 mutants — P-12 / B-2 (AL-30)

Run 2026-11-20 with `python3 crates/termlink-session/tests/mutants/run-t3700.py` (scratch runtime dir, T-3822). Every
mutant must be killed (FP 3.1.b). m4/m6/m7 are the E-169.5, E-66.5-style and E-66.6 controls.

| mutant | what | verdict |
|---|---|---|
| m1-skip-signature-check | a read proof is accepted without verifying its signature | killed |
| m2-ignore-ts-window | a proof with a skewed timestamp still proves | killed |
| m3-bind-by-name | a bind admits any reader naming the bound agent (ruling E broken) | killed |
| m4-hub-unreadable-is-empty | E-169.5 control: an unreadable consumer list answers as a list | killed |
| m5-sidecar-malformed-is-one | a malformed hub answer is shown as one consumer | killed |
| m6-acks-not-recorded | E-66.5-style control: an ack (receipt) does not record its poster | killed |
| m7-reads-not-recorded | E-66.6 control: only acks and claims record; a read-only consumer is missed | killed |
| m8-no-eviction-row | the 17th reader evicts silently | killed |
| m9-hub-instance-ignored | V-12: the hub keys consumers by agent only (a second copy merges) | killed |
| m10-sidecar-instance-ignored | V-12: the sidecar treats its own agent at any instance as itself | killed |
| m11-card-mismatch-proves | a proof signed by a key not on the named card proves | killed |
| m12-revoked-proves | a revoked key still proves | killed |
| m13-bind-without-row | a bind takes effect when its admin row could not be written | killed |
| m14-attention-every-check | a second consumer raises an entry on every check | killed |
| m15-liveness-reads-content | liveness reads the main's inbox content (a consumer of it) instead of first_at | killed |

15/15 killed

## Finding during the run

A mutant killed after `cargo build -p termlink` (here m9) left its mutated hub binary in target/debug; the L4 tests
start that binary without rebuilding it, so V-12 failed on the clean tree afterwards. Both run-t3700.py and
run-t3823.py now rebuild the clean binary at the end. Production is not affected: runme installs from its own
`target/local-fast` build.
