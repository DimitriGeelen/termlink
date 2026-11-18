# T-3704 confirmation round 3 — dispositions (review cap reached)

Seats: opencode (`review-3-opencode.md`) PASS with 4 MINOR; claude-code (`review-3-claude-code.md`) FAIL on one new MAJOR,
every round-2 BLOCKER/MAJOR confirmed closed except part of C-5. This is the third review round for slice A
(design r1, code r2, confirmation r3), the cap (ring20 lesson 5.16): every remaining point below is accepted with a
change, rejected with a reason, or deferred as a criterion of a named task, and is not sent round again.

| # | Finding | Disp. | Change / where it lands |
|---|---|---|---|
| CC r3 MAJOR 1 | the global 60-row cap lets made-up names suppress the admin row of a real (carded) impersonation | A | uncarded refusals coalesce to one row per signer per minute under `_uncarded` and no longer count toward the cap; the cap applies to carded agents only. Unit: 60 uncarded names then a carded refusal → the carded one is logged. Mutant m14 retargeted. |
| CC r3 MINOR 2 | an unreadable store at start leaves no baseline | A | start-up baselines a sentinel no file digest equals, so a later outside write is still refused. |
| CC r3 MINOR 4 | `set_mode` read/log outside the lock | A | read, log and write under one lock. |
| CC r3 C-5 part | receiver E-69 not on a real hub | D | criterion of P-31 (T-3713): with named rosters configured on the W1 pair, a forged post claiming a bound name is refused at the receiver (live, hub check off). Unit coverage with real keys stays. |
| CC r3 MINOR 3, OC r3 MINOR 4 | `key-card-written` entries: same-seq edit after restart not raised; silent list errors; mode changes unconsumed; first pass raises one per existing card | D | criteria of P-31 (T-3713), the unit that publishes the cards and moves the setting: key the entry on (agent, seq, signing key), raise attention on a list error and on a `key-card-mode` change, persist last-seen so a fresh sidecar does not raise for existing cards. |
| CC r3 MINOR 5, OC r3 MINOR 1 | reconcile ignores cards deleted outside the RPC; pre-change rows lack `card_digest` | D | slice B criterion (it rewrites the store for replacement statements): report logged agents missing from the store; no pre-change rows exist in the field (no card has been written on any hub yet). |
| OC r3 MINOR 2 | the setting file has no runtime integrity guard (start-up only) | D | P-31 criterion: baseline the setting like the store digest, so an outside change while running is refused or raises attention. Until P-31 the setting is `off` everywhere. |
| OC r3 MINOR 3 | 16-signer cap can push the agent's own key into "other" | D | P-31 criterion: exempt the card's own key from the cap (the hub knows it once the card exists). |
| CC r3 MINOR 6 | three test assertions weaker than their names | D | slice B / P-31 test pass: assert the -32044 and refusal count for the impostor, the "admin-log row not written" text, and that no message was delivered for the unlisted type. |

None of the deferred items lets a forged arc-011 post pass in enforce mode, and the setting stays `off` on every hub until
P-31.
