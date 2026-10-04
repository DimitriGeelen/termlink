# T-3335 — Why "hubs never talk to hubs"? Back to principles

**Asked by the operator, 2026-10-04,** while deciding OD-1 (cross-host send path):
"Can you find back why we decided that hubs cannot talk to hubs? As we are going to multi-agent,
multi-host communications and even multiple hubs on one system, that seems to be something we might
want and actually is necessary."

## 1. The record, in order

1. **2026-05-21, G-060 / T-1791.** An operator saw 1800 vs 486 messages in `agent-chat-arc` on .107 and
   .122 and filed it as a federation bug. The RCA found that **no hub-to-hub primitive had ever been
   built** (`grep` for federat/peer_subscribe/cross_hub: 0 matches). The disparity was declared "the
   DESIGN" and the gap reclassified as documentation (`.context/project/concerns.yaml`, G-060).
   G-060 itself left the real question open: "Optional larger inception: does the fleet WANT
   auto-federation? … NOT decided in T-1791."
2. **2026-05-25, T-1793** asked exactly that (`docs/reports/T-1793-auto-federated-channel-topics.md`).
   It weighed **automatic replication of channel topics** across hubs:
   2a. benefits: simpler agent UX, one source of truth, no `--hub` to remember;
   2b. costs: state sync, a consistency model (last-write-wins, vector clocks, CRDTs), conflict
       resolution, bandwidth amplification on every post, cross-hub ordering, retention divergence.
   2c. Outcome: **parked** (DEFER in substance; the decision field says GO, a recorded discrepancy),
       because T-1166 was not blocked, client-driven cross-posting "works correctly when used", and
       the cost is significant.
   2d. **Revisit when** (a) agents keep being surprised by per-hub semantics, or (b) "a concrete
       fleet-wide coordination workflow emerges that the client-driven pattern cannot serve cleanly".
       Revisit date 2026-08-21. **No revisit happened.**
3. **2026-06/07, T-2229.** ring20 reported "cross-hub federation broken". Answered "working as
   designed"; an operator `fleet federation-status` verb was deferred (IW-3).
4. **2026-07-31, T-2470** wrote `docs/CHARTER.md`, including non-goal #1, "Not an inter-hub federation
   layer … never automatic (G-060)". The task's only human AC, "Bless the canonical purpose sentence
   (and the non-goals)", is **still unticked**; T-2470 is `started-work`, `owner: human`. Yet
   `CHARTER.md:17` states the sentence "is human-blessed". **The non-goal was never ratified.**
5. **T-2569** then turned the non-goal into a build-breaking tripwire
   (`crates/termlink-hub/tests/no_federation_tripwire.rs`): the hub may not read peer-hub config, may
   not build a hub-speaking client, and its outbound connections are enumerated.
6. **2026-09-23, SQ-1** ("sending to a peer stays channel.post via the hub") and the sidecar-API brief
   ("a second bus … put it to the charter") reasoned *from* the non-goal, not about it.

**Finding.** The rule began as an observation that a feature was missing (G-060), was parked with
explicit revisit triggers (T-1793), was written into an unratified charter by an agent (T-2470), and
was then enforced in code (T-2569). At no point did anyone decide on principle that hubs must not talk.
The only analysis on record (T-1793) is about **replicating shared topic state**, not about relaying
addressed messages.

## 2. Have T-1793's revisit triggers fired?

7. **(a) Repeated surprise: yes.** ring20 (T-2229) after the documentation shipped; 055 E2 (an agent
   started with `env -i` read an empty, different hub and saw "no mail"); AEF T-3779 (mail hub tied to
   the terminal runtime dir). The documentation did not stop it.
8. **(b) A workflow the client-driven pattern cannot serve cleanly: yes.** Interactive conversation
   between agents on different hosts (arc-011). Client-driven means every sender must know which hub
   the receiver uses, hold that hub's secret and TLS pin, and retry against it itself: N agents × M
   hubs of credentials. That is where the recurring failures sit: wrong hub, stale secret, rotation
   heal paths, the `fleet reauth` machinery.

## 3. Reflection: "hubs talking" is three different things

| Kind | What it means | Shared state? | T-1793's costs apply? |
|---|---|---|---|
| **R. Replication** | A topic exists on many hubs and stays in sync | Yes | **All of them**: consistency model, conflicts, ordering, amplification, retention divergence |
| **F. Addressed relay** (store-and-forward, the e-mail model) | A message addressed `//host/hub/project/agent` is handed by the sender's hub to the destination hub, which owns it from then on | **No**: every message has exactly one owner, its destination | Almost none: no consistency model, no conflicts; ordering is per conversation; dedupe by `client_msg_id` already exists |
| **D. Directory / presence exchange** | Hubs tell each other which agents and projects they serve | Small, soft state (rebuildable) | Mild: staleness, which a TTL handles |

9. The non-goal, as written, forbids all three, while the only reasons on record argue against **R**.
10. What arc-011 needs is **F** plus a little **D**. The five-level address (T-3325) already names the
    destination hub, so a hub can tell "this is mine" from "forward this" without any guessing.
11. **F is also the answer to both sides of OD-1.**
    11a. The operator wants push between agents on different hosts without depending on one agent's
         sidecar knowing the whole fleet.
    11b. The reviewers object to a **second path alongside the hub** (E4: two paths diverge). Hub relay
         is not a second path: it is the hub path, extended by one hop. Sidecars keep talking only to
         their local hub; only hubs hold peer-hub trust (M hubs, not N×M agent credentials).
12. **Several hubs on one host** become ordinary rather than a failure mode: today an agent on the wrong
    local hub is deaf (055 E2); with relay and a directory, it is one hop away.

## 4. What F would cost, honestly

13. The hub gains outbound connections and a durable outbound queue per peer hub. That is new hub state
    and a new failure surface; the tripwire (T-2569) has to be rewritten to allow exactly this and
    still forbid R.
14. Receipts need a hop rule: "accepted by my hub" vs "stored by the destination hub". This feeds OD-4.
15. Loops and misroutes need a hop limit and a refusal for unknown destinations (055's "unknown
    addresses are refused, not created").
16. Hub-to-hub trust: the HMAC secret + TLS pin machinery exists, but now it is hub-to-hub and must
    rotate cleanly. This is the trust-model gap every reviewer raised, moved to a smaller set (hubs).
17. A partitioned destination hub means messages wait at the origin hub, and the sender sees "queued
    at my hub, destination unreachable" rather than "delivered". That is correct behaviour, but it
    must be visible (OD-14).
18. It is a charter change, and the charter is the operator's (T-2470 is still waiting for exactly
    that ratification).

## 5. Suggested way forward (agent proposal, not a ruling)

19. Hold OD-1 until this principle is settled; OD-1's options all assumed the non-goal.
20. Settle it through T-2470: the charter ratification is still open, so the non-goal can be reworded
    there rather than amended later. Proposed wording: "Not a replication layer: hubs never sync
    shared topic state. Hubs MAY relay individually addressed messages to the destination hub
    (store-and-forward), and exchange a directory of whom they serve."
21. Open one single-question inception: "Should hubs relay addressed messages (F) and exchange a
    directory (D)?", with T-1793 as the record of R being parked, and the tripwire rewrite as its
    first build slice if GO.
22. Fix `CHARTER.md:17`: it claims a human blessing that the task record says has not happened.
