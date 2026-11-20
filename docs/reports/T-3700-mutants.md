# T-3700 mutants — P-12 / B-2 (AL-30)

Runner: `python3 crates/termlink-session/tests/mutants/run-t3700.py` (scratch runtime dir, T-3822; rebuilds the clean
binary at the end). Every mutant must be killed (FP 3.1.b). m4/m6/m7 are the E-169.5, E-66.5-style and E-66.6
controls; m16–m26 cover the review-round-1 fixes, m27–m30 the round-2 fixes, m31–m33 the round-3 fixes.

## Post-deploy run, 2026-11-20 (m34 pins the live `reader` field regression fixed in e15e4c380)

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
| m9-hub-instance-ignored | V-12: the hub keys consumers without the instance (a second copy merges) | killed |
| m10-sidecar-instance-ignored | V-12: the sidecar treats its own key at any instance as itself | killed |
| m11-card-mismatch-proves | a proof signed by a key not on the named card proves | killed |
| m12-revoked-proves | a revoked key still proves | killed |
| m13-bind-without-row | a bind takes effect when its admin row could not be written | killed |
| m14-attention-every-check | a second consumer raises an entry on every check | killed |
| m15-liveness-reads-content | liveness reads the main's inbox content instead of first_at | killed |
| m16-claim-gate-removed | a claim on an inbox is neither recorded nor refused after a bind | killed |
| m17-bindings-fail-open | unreadable bindings are treated as none (every bound inbox opens) | killed |
| m18-strength-merge | r1 CC 1: a claimed reader merges into the proven entry of the same name and instance | killed |
| m19-self-by-name | r1 CC 1: the sidecar takes any entry under its name and instance for itself | killed |
| m20-no-long-poll-recheck | r1 CC 6: a long-poll that waited across a bind still delivers | killed |
| m21-liveness-fallback-any-error | r1 CC 9 / r2 N3: liveness falls back to a content read on any coded first_at error | killed |
| m22-ack-replay-proves | r1 CC 7: a replayed old receipt proves | killed |
| m23-corrupt-overwritten | r1 CC 4: a corrupt table is not moved aside (saved over) | killed |
| m24-stale-status-shown | r1 CC 2: an old consumers line is shown as current | killed |
| m25-unknown-topics-recorded | r1 CC 3: reads of inbox topics that do not exist are recorded | killed |
| m27-full-table-clean-list | r2 N1: an inbox missing from a full table is answered as a clean empty list | killed |
| m28-suppressed-count-lost | r2 N5: the suppressed count is dropped when the row carrying it fails | killed |
| m29-bindings-unreadable-unbound | r2 N2: unreadable bindings count as unbound for a missing topic | killed |
| m30-reset-marker-not-saved | r2 N4: the reset marker stays in memory until a record | killed |
| m31-gate-ignores-bind-before-create | r3 CC 4: a bound inbox whose topic does not exist is read ungated | killed |
| m32-ack-ignores-bind-before-create | r3 CC 4: an ack on a bound, not-yet-created inbox is not gated | killed |
| m33-entered-after-drop-served | r3 CC 1: an inbox entering after a drop is served as a clean list | killed |
| m34-proof-in-reader-field | live regression 2026-11-20: the proof read from `reader`, which old clients send as a name | killed |
| m26-full-table-evicts-active | r1 CC 3: a full table evicts an inbox read inside the window | killed |


**34/34 killed**

## Final run, 2026-11-20 (after review round 3; m31–m33 cover its fixes)

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
| m9-hub-instance-ignored | V-12: the hub keys consumers without the instance (a second copy merges) | killed |
| m10-sidecar-instance-ignored | V-12: the sidecar treats its own key at any instance as itself | killed |
| m11-card-mismatch-proves | a proof signed by a key not on the named card proves | killed |
| m12-revoked-proves | a revoked key still proves | killed |
| m13-bind-without-row | a bind takes effect when its admin row could not be written | killed |
| m14-attention-every-check | a second consumer raises an entry on every check | killed |
| m15-liveness-reads-content | liveness reads the main's inbox content instead of first_at | killed |
| m16-claim-gate-removed | a claim on an inbox is neither recorded nor refused after a bind | killed |
| m17-bindings-fail-open | unreadable bindings are treated as none (every bound inbox opens) | killed |
| m18-strength-merge | r1 CC 1: a claimed reader merges into the proven entry of the same name and instance | killed |
| m19-self-by-name | r1 CC 1: the sidecar takes any entry under its name and instance for itself | killed |
| m20-no-long-poll-recheck | r1 CC 6: a long-poll that waited across a bind still delivers | killed |
| m21-liveness-fallback-any-error | r1 CC 9 / r2 N3: liveness falls back to a content read on any coded first_at error | killed |
| m22-ack-replay-proves | r1 CC 7: a replayed old receipt proves | killed |
| m23-corrupt-overwritten | r1 CC 4: a corrupt table is not moved aside (saved over) | killed |
| m24-stale-status-shown | r1 CC 2: an old consumers line is shown as current | killed |
| m25-unknown-topics-recorded | r1 CC 3: reads of inbox topics that do not exist are recorded | killed |
| m27-full-table-clean-list | r2 N1: an inbox missing from a full table is answered as a clean empty list | killed |
| m28-suppressed-count-lost | r2 N5: the suppressed count is dropped when the row carrying it fails | killed |
| m29-bindings-unreadable-unbound | r2 N2: unreadable bindings count as unbound for a missing topic | killed |
| m30-reset-marker-not-saved | r2 N4: the reset marker stays in memory until a record | killed |
| m31-gate-ignores-bind-before-create | r3 CC 4: a bound inbox whose topic does not exist is read ungated | killed |
| m32-ack-ignores-bind-before-create | r3 CC 4: an ack on a bound, not-yet-created inbox is not gated | killed |
| m33-entered-after-drop-served | r3 CC 1: an inbox entering after a drop is served as a clean list | killed |
| m26-full-table-evicts-active | r1 CC 3: a full table evicts an inbox read inside the window | killed |


**33/33 killed**

## Round-3 run, 2026-11-20 (after review round 2; m27–m30 cover its fixes)

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
| m9-hub-instance-ignored | V-12: the hub keys consumers without the instance (a second copy merges) | killed |
| m10-sidecar-instance-ignored | V-12: the sidecar treats its own key at any instance as itself | killed |
| m11-card-mismatch-proves | a proof signed by a key not on the named card proves | killed |
| m12-revoked-proves | a revoked key still proves | killed |
| m13-bind-without-row | a bind takes effect when its admin row could not be written | killed |
| m14-attention-every-check | a second consumer raises an entry on every check | killed |
| m15-liveness-reads-content | liveness reads the main's inbox content instead of first_at | killed |
| m16-claim-gate-removed | a claim on an inbox is neither recorded nor refused after a bind | killed |
| m17-bindings-fail-open | unreadable bindings are treated as none (every bound inbox opens) | killed |
| m18-strength-merge | r1 CC 1: a claimed reader merges into the proven entry of the same name and instance | killed |
| m19-self-by-name | r1 CC 1: the sidecar takes any entry under its name and instance for itself | killed |
| m20-no-long-poll-recheck | r1 CC 6: a long-poll that waited across a bind still delivers | killed |
| m21-liveness-fallback-any-error | r1 CC 9 / r2 N3: liveness falls back to a content read on any coded first_at error | killed |
| m22-ack-replay-proves | r1 CC 7: a replayed old receipt proves | killed |
| m23-corrupt-overwritten | r1 CC 4: a corrupt table is not moved aside (saved over) | killed |
| m24-stale-status-shown | r1 CC 2: an old consumers line is shown as current | killed |
| m25-unknown-topics-recorded | r1 CC 3: reads of inbox topics that do not exist are recorded | killed |
| m27-full-table-clean-list | r2 N1: an inbox missing from a full table is answered as a clean empty list | killed |
| m28-suppressed-count-lost | r2 N5: the suppressed count is dropped when the row carrying it fails | killed |
| m29-bindings-unreadable-unbound | r2 N2: unreadable bindings count as unbound for a missing topic | killed |
| m30-reset-marker-not-saved | r2 N4: the reset marker stays in memory until a record | killed |
| m26-full-table-evicts-active | r1 CC 3: a full table evicts an inbox read inside the window | killed |


**30/30 killed**

## Round-2 run, 2026-11-20

| mutant | what | verdict |
|---|---|---|
| m1-skip-signature-check | a read proof is accepted without verifying its signature | killed |
| m2-ignore-ts-window | a proof with a skewed timestamp still proves | killed |
| m3-bind-by-name | a bind admits any reader naming the bound agent (ruling E broken) | killed |
| m4-hub-unreadable-is-empty | E-169.5 control: an unreadable consumer list answers as a list | SURVIVED, then killed (see below) |
| m5-sidecar-malformed-is-one | a malformed hub answer is shown as one consumer | killed |
| m6-acks-not-recorded | E-66.5-style control: an ack (receipt) does not record its poster | killed |
| m7-reads-not-recorded | E-66.6 control: only acks and claims record; a read-only consumer is missed | killed |
| m8-no-eviction-row | the 17th reader evicts silently | killed |
| m9-hub-instance-ignored | V-12: the hub keys consumers without the instance (a second copy merges) | killed |
| m10-sidecar-instance-ignored | V-12: the sidecar treats its own key at any instance as itself | killed |
| m11-card-mismatch-proves | a proof signed by a key not on the named card proves | killed |
| m12-revoked-proves | a revoked key still proves | killed |
| m13-bind-without-row | a bind takes effect when its admin row could not be written | killed |
| m14-attention-every-check | a second consumer raises an entry on every check | killed |
| m15-liveness-reads-content | liveness reads the main's inbox content instead of first_at | killed |
| m16-claim-gate-removed | a claim on an inbox is neither recorded nor refused after a bind | killed |
| m17-bindings-fail-open | unreadable bindings are treated as none (every bound inbox opens) | killed |
| m18-strength-merge | r1 CC 1: a claimed reader merges into the proven entry of the same name and instance | killed |
| m19-self-by-name | r1 CC 1: the sidecar takes any entry under its name and instance for itself | killed |
| m20-no-long-poll-recheck | r1 CC 6: a long-poll that waited across a bind still delivers | killed |
| m21-liveness-fallback-any-error | r1 CC 9: liveness falls back to a content read on any first_at error | killed |
| m22-ack-replay-proves | r1 CC 7: a replayed old receipt proves | killed |
| m23-corrupt-overwritten | r1 CC 4: a corrupt table is not moved aside (saved over) | killed |
| m24-stale-status-shown | r1 CC 2: an old consumers line is shown as current | killed |
| m25-unknown-topics-recorded | r1 CC 3: reads of inbox topics that do not exist are recorded | killed |
| m26-full-table-evicts-active | r1 CC 3: a full table evicts an inbox read inside the window | killed |

m4 survived the full run: after round 1 an unreadable table is moved aside (the "history incomplete" path), so no
test made a SAVE fail any more. Added `a_failed_save_makes_the_list_an_error_never_a_list`; re-run of m4 alone: killed.

**26/26 killed**

## Round-1 run, 2026-11-19: 15/15 killed (m1–m15)

## Finding during the round-1 run

A mutant killed after `cargo build -p termlink` (m9) left its mutated hub binary in target/debug; L4 tests start
that binary without rebuilding, so V-12 then failed on the clean tree. run-t3700.py and run-t3823.py now rebuild the
clean binary at the end. Production is unaffected: runme installs its own `target/local-fast` build.
