# T-3704 spec extract — P-16 / B-6: agent key card, post-time signature check, signed dead letter

Read-only research for T-3704 (arc-011 plan unit P-16, build unit B-6). Prepared 2026-11-17. Every passage under
"Spec" and "Cited parameters/invariants" is copied verbatim with `file:line`; analysis is only in the last two sections.

## Spec (verbatim, by item)

### P-16 / B-6 (plan, build table, traceability)

`interactive-agent-communication-08-plan.md:159-159`

```text
| P-16 | build | PH-2 | new | B-6: agent key card at the hub, post-time signature check, signed dead letter for retired instances | R-44, R-62, R-75 | E-69, E-70, E-169 | P-10 (security: signing key before signed dead letters) | the E-n listed pass at the level 05 states; the B-6 specs pass | fixture suite for the unit exits 0 with a PASS line per listed E-n, and its mutant list is all killed (FP 3.1.b) | check off at the hub; cards kept |
```

`interactive-agent-communication-08-plan.md:246-248`

```text
5.1 **Hard security orderings, enforced as `security` edges:** signing key before any signed state (P-10 → P-15, P-16);
encryption before any remote put, never plaintext (P-6 → P-22); audit before execute (P-49 → P-52 the account helper,
P-49 → P-56 grants); deny by default before any grant path (P-55 → P-56); grants before the startup chain and before
```

`interactive-agent-communication-08-plan.md:595-595`

```text
| 20 | P-16 | PH-2 | build | P-10 | key card, needed by forward-once |
```

`interactive-agent-communication-08-plan.md:1027-1027`

```text
| P-16 | PH-2 | T-3704 | new | next |
```

`interactive-agent-communication-04-architecture.md:1547-1547`

```text
| B-6 | P17d | Agent key card on the hub, post-time signature check, signed dead letter for retired instances (6.6.5-6) | B-1 | new |
```

`interactive-agent-communication-04-architecture.md:1561-1561`

```text
| B-20 | R-67.a1 (4) | Forward-once, VIA and FORWARD-STALE notices, H-24 (section 7) | B-6 | new |
```

`interactive-agent-communication-04a-traceability.md:21-21`

```text
| SI-3 | sidecar signatures [B]; K-15 key card, post-time check (6.6.5) | PH-1 [B], PH-2 | B-6 |
```

`interactive-agent-communication-04a-traceability.md:44-44`

```text
| SI-26 | exact instance [B]; signed dead letter (6.6.6) | PH-1 [B], PH-2 | B-6 |
```

`interactive-agent-communication-04a-traceability.md:112-112`

```text
| TH-1 | SI-3 | signatures, key card | | TH-34 | SI-24 | admin log, revoke logged (6.2.2) |
```

`interactive-agent-communication-04a-traceability.md:119-119`

```text
| TH-8 | SI-3, SI-12 | key card; K-28 | | TH-41 | SI-20 | lease [B], DR-9, K-26 |
```

### Architecture: K-15 and section 6.6 (key card 6.6.5, dead letter 6.6.6, proof 6.6.7)

`interactive-agent-communication-04-architecture.md:197-197`

```text
| K-15 | Hub changes: consumer list, admin log, sweep refusal, founding-verb scope, agent key card, dead letter (P17) | hub | TermLink | new, 6.6 |
```

`interactive-agent-communication-04-architecture.md:495-495`

```text
### 6.6 P17 — hub changes (K-15; SI-16, FX-6, FX-4, PR-1a rest, R-44 DEAD)
```

`interactive-agent-communication-04-architecture.md:514-521`

```text
6.6.5 **Agent key card (P7-2 rest).** The hub keeps a card per agent: signing key, encryption key, project, instance
ids, registered by the runme provisioning; a replacement is accepted only as a statement signed by the old key
(C-6a, built-in route from PH-3) or a runme action; the card is what the hub checks a post's signature against.
6.6.6 **Dead letter for a retired instance.** When provisioning retires an instance id (runme), the hub writes a
signed, sequenced `DEAD` statement (hub signing key, seq per project); a sender's conversation mail to it yields
DEAD and stops polling (R-44, R-62). Signed DEAD from liveness alone is PH-3 (R-60.a).
6.6.7 Proof: second consumer flagged within 60 s, refused after bind; sweep with an open message refused; inject with
an `observe` token refused and logged; DEAD unsigned refused. Build items B-2, B-3, B-4, B-6.
```

`interactive-agent-communication-04-architecture.md:403-404`

```text
6.2.5 **Effect.** A message signed by a revoked key is refused at the receiver (and at the hub on post, once K-15's
key card exists) and recorded; the refusal is visible to the sender as a signed notice. Mail to the revoked agent
```

`interactive-agent-communication-04-architecture.md:1223-1223`

```text
| BP-3 | direct post with the hub secret | SI-3 at hub and receiver, K-15 key card | closed for posting; envelope reading open (RR-6) until PH-4 encryption |
```

`interactive-agent-communication-04-architecture.md:1444-1444`

```text
| 18.2.15 | Key pinning and rotation (R-75) | **Decided [P] for PH-2** (TOFU pin of the hub signing key, rotation keeps id and key; agent key replacement only by old-key statement or runme); out-of-band pinning at PH-3 | 6.1, 6.6.5, DR-2, AR-1, 12.1 |
```

### R-44 (all sub-items)

`interactive-agent-communication-01-requirements.md:793-799`

```text
R-44 **Message states and "stuck".**
R-44.a The sender MUST see, at each poll, one of five states for every message it sent, computed from the hub record (R-45): WAITING (the next step is not yet due); WAITING FOR RECIPIENT (the recipient is known not to be running or not able to receive: keep polling, show it, deliver when it returns); STUCK (the next step is overdue AND the responsible party is reachable and alive); UNKNOWN (the sender cannot get information: destination hub unreachable or liveness unknown; keep polling, show "unknown since T", never treat as dead); DEAD (the recipient's home hub declares that instance ended: stop polling, dead letter to the sender). STUCK, UNKNOWN and WAITING FOR RECIPIENT MUST be visible to the sender and in the cockpit, and STUCK MUST name the stalled step. [R] (changed in v0.5 by CR-1, ruling 2026-10-10 A) v0.4.1 text: "UNKNOWN (the sender cannot get information: destination hub unreachable or liveness unknown ...)" and "computed from the hub record". UNKNOWN stays when neither the hub record nor circuit receipts can be read. With an established circuit and the destination hub unreachable the sender holds its receipts, so the state is computed from them and shown labelled "unreconciled", not UNKNOWN. On the hub's return the hub record decides again.
R-44.b Functional · P1 [P] · source OD-3 point 3 (log).
R-44.c Rationale: "stuck" means someone can fix something now; "unknown" and "waiting for recipient" mean nothing to fix yet. Nothing turned "waiting" into a visible "stuck" before (the 2026-10-03 failure).
R-44.d Verification: simulated-clock fixture over the hub record; live canary (R-66).
R-44.e Acceptance. The stuck deadlines are: not accepted (SENT, no STORED within 1 min while the destination hub is reachable); not handed over (STORED, no HANDED_OVER within 2 ticks of the agent becoming ready, about 1 min, while alive and ready); not answered (HANDED_OVER, no reply or no-action within 1 h while alive; a sender may set another deadline per message). Urgent: accepted within 15 s; handed over at the next tool call or turn end (R-19); answered within 5 min. Given each deadline is missed while the responsible party is reachable, then the state is STUCK naming that step. Given the destination hub is unreachable and neither the hub record nor circuit receipts can be read, then the state is UNKNOWN and polling continues. [R] (changed in v0.5 by CR-1) Given the destination hub is unreachable and an established circuit holds receipts, then the state is computed from the receipts and labelled "unreconciled", not UNKNOWN (R-44.a, R-45.a). v0.4.1 text: "Given the destination hub is unreachable, then the state is UNKNOWN and polling continues." Given the home hub declares the instance ended, then DEAD and polling stops. Given a dead or absent recipient that may return, then WAITING FOR RECIPIENT, never STUCK. Only the home hub may say DEAD (R-60).
R-44.g Status: designed only. AEF's WAITING_NO_RECIPIENT is the existing form of one state. Where escalation of STUCK lands: R-37.o. The state vocabulary is proposed to AEF (offset 314).
```

### R-62 (all sub-items)

`interactive-agent-communication-01-requirements.md:937-943`

```text
R-62 **Exact instance: fail loudly, never redirect.**
R-62.a [R] Ruled (OD-10 C): a new request goes to project plus role, and the home hub resolves it to one live copy. Mail INSIDE a conversation goes to the exact copy the conversation is bound to; if that copy has ended the sender gets a dead letter (state DEAD), never a silent redirect. A conversation moves to another copy only by an explicit hand-over with its context. A fresh copy never answers conversation mail. [R~] A copy's runtime id is never reused. [R] (changed in v0.5 by CR-15, ruling 2026-10-10 OQ-17 D) "An explicit hand-over" now says who may start one and what fences the old copy: a live holder's signed statement plus the new copy's acceptance, or the operator; a dead holder's hand-over is a Tier 0 event (R-72), policy-approved only when the home hub declared the holder DEAD and the successor holds the R-67 lease for the same role in the same project, otherwise decided on a human route; every hand-over increments the membership epoch and the old copy is fenced (R-74). [R] (changed in v0.5 by CR-20) Conversation mail bound to an exact copy is never forwarded; the loser of a duplicate-main resolution keeps its replies inside its own conversations valid (R-67.a1).
R-62.b Security invariant · P1 [P] · source OD-10 (log).
R-62.c Rationale: a wrong delivery is worse than a visible failure; the 2026-10-06 wrong-session wake-ups (2.4a).
R-62.d Verification: fixture that ends the bound copy while another copy is live.
R-62.e Acceptance. Given a conversation bound to copy X and X ended, then a message of that conversation yields a dead letter and the live copy Y is not woken for it. Given a new request to the role, then it reaches one live copy.
R-62.g Status: not built. Open: how a role is resolved (R-67).
```

### R-75 (all sub-items)

`interactive-agent-communication-01-requirements.md:1049-1055`

```text
R-75 **Every message is signed.**
R-75.a [R] Ruled (D4, 2026-10-10; CR-2): every message MUST be signed by the sending agent's own key and verified at the receiver and at the hub. [P] A message that fails verification is refused and the refusal is recorded; it is never delivered. This replaces the "not required" of R-34.e (2) (v0.4.1 text: authentication of the sender "is not required by R-34") and gives R-63 its "verified key".
R-75.b Security invariant · P1 [P] · source CR-2; D4 (T-3351 decision 2026-10-10); GP-1, PR-1, SI-3.
R-75.c Rationale: R-34 said authentication is not required, and R-63 relied on a verified key with no requirement that messages carry one, so one hostile session could sign as another (ADV-2).
R-75.d Verification: a fixture with a message signed by the wrong key, an unsigned message and a message altered after signing, at the receiver and at the hub.
R-75.e Acceptance. Given a message whose signature does not verify at the hub, then it is refused there. Given one that reaches the receiver on a circuit and does not verify, then the receiver refuses it and records the refusal. Given a session of project Q that signs with its own key and asserts it is project P, then the message does not verify as P's agent.
R-75.g Status: per-agent keys exist (T-3346); no requirement made messages carry a signature before this one. Open: key pinning and rotation details (step 4).
```

### Phasing rows for R-44 / R-62 / R-75 (03)

`interactive-agent-communication-03-floor-and-phasing.md:165-165`

```text
| R-62 | Floor | SI-26, SI-29 | |
```

`interactive-agent-communication-03-floor-and-phasing.md:173-173`

```text
| R-75 | Floor | SI-3 | |
```

`interactive-agent-communication-03-floor-and-phasing.md:326-326`

```text
| R-44 | Message states and "stuck" | Must | PH-1 four labels; PH-2 DEAD as dead letter only; PH-3 signed DEAD | INC-a was a "waiting" nobody could see |
```

`interactive-agent-communication-03-floor-and-phasing.md:344-344`

```text
| R-62 | Exact instance, never redirect | Floor | PH-1; PH-4 hand-over | SI-26, SI-29. INC-c |
```

`interactive-agent-communication-03-floor-and-phasing.md:357-357`

```text
| R-75 | Every message is signed | Floor | PH-1 | SI-3 |
```

### E-69, E-70, E-169 (evidence; all sub-items) and AFC rows

`interactive-agent-communication-05-evidence.md:721-721`

```text
### 6.10 Hub changes (K-15; ARC4 6.6; B-2, B-3, B-4, B-6)
```

`interactive-agent-communication-05-evidence.md:745-757`

```text
**E-69 — Posts are checked against the agent key card; replacement only by old key or runme.** Links: SI-3, R-75, TH-1, TH-8, TH-56. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-6).
E-69.1 EVIDENCE: the hub refuses a post whose signature does not verify against the poster's card; a forged message with the shared host key or a sibling's key id is refused at hub and receiver with an entry; a card replacement is accepted only as a statement signed by the old key or a runme action (ARC4 6.6.5; TM SI-3 probe).
E-69.2 ARRANGE: throwaway hub with cards for `fx-a`, `fx-b`; the hub HMAC secret.
E-69.3 ACT: post as `fx-a` signed with `fx-b`'s key; post with the hub secret only (BP-3); submit a card replacement signed by the new key; signed by the old key.
E-69.4 ASSERT: first two refused at the hub (and, bypassing the hub check, at the receiver) with entries; new-key replacement refused; old-key replacement accepted and logged.
E-69.5 CONTROL: the built mutant `fc-01-verify-accepts-any-signature` turns this red.

**E-70 — Signed dead letter for a retired instance.** Links: SI-26, R-44, R-62, TH-54. Category: core function. Level: L4. Tier: 2. Phase: PH-2 (B-6).
E-70.1 EVIDENCE: retiring an instance by runme makes the hub write a signed, sequenced `DEAD`; a sender's conversation mail to it ends DEAD and stops polling; an unsigned or wrongly signed DEAD is refused (ARC4 6.6.6).
E-70.2 ARRANGE: throwaway hub; a conversation between `fx-a` and instance `fx-b#1`.
E-70.3 ACT: retire `fx-b#1`; `fx-a` sends inside the conversation; inject an unsigned DEAD for `fx-a`.
E-70.4 ASSERT: sender state DEAD with the statement's seq; polling stops; unsigned DEAD refused and flagged; the message is never delivered to another instance (SI-26).
E-70.5 CONTROL: mutant accepting unsigned DEAD stops a live conversation.
```

`interactive-agent-communication-05-evidence.md:960-965`

```text
**E-169 — Hub admin and log durability: unreadable consumers, finality, admin-log and use-log rows, a poster without a card.** Links: SI-16, FX-4, FX-6, AFC-33, FCW-125, FCW-127, FCW-128, FCW-132, FCW-135. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-2, B-3, B-4, B-6)
E-169.1 EVIDENCE: an unreadable consumer list shows `consumers: unknown`, never "one"; a sweep whose finality cannot be read is refused; an admin RPC whose log row cannot be written is refused; a founding verb whose use-log row cannot be written is refused; an arc-011 message from a poster with no key card is refused.
E-169.2 ARRANGE: a hub fake failing each read or write in turn; a poster without a card.
E-169.3 ACT: tick; sweep; an admin RPC; `command.inject`; a post.
E-169.4 ASSERT: each refused or shown unknown as stated, with the reason.
E-169.5 CONTROL: mutant defaulting an unreadable consumer list to empty reports one consumer.
```

`interactive-agent-communication-05-evidence.md:1502-1503`

```text
| AFC-33 | a post whose signature does not verify against the key card: refused | 6.6.5 | E-69 |
| AFC-34 | an unsigned or wrongly signed DEAD: refused | 6.6.6 | E-70 |
```

`interactive-agent-communication-08-plan.md:928-929`

```text
| E-11 | P-7 | E-69 | P-16 | E-127 | P-15, P-33 |
| E-12 | P-7 | E-70 | P-16 | E-128 | P-18, P-33 |
```

`interactive-agent-communication-08-plan.md:970-970`

```text
| E-53 | P-17 | E-111 | P-58 | E-169 | P-12, P-13, P-14, P-16 |
```

### Pseudo-code AL-33 — key card: post-time check and replacement (verbatim, whole section)

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:630-656`

```text
### 2.17 AL-33 — Agent key card: post-time check and replacement (K-15; ARC4 6.6.5; B-6; P7-2)

2.17.a **SERVES:** SI-3, R-75; TH-1, TH-8, TH-56; BP-3; AFC-33.
2.17.b **PROBLEM:** the hub accepted any post from a token holder (BP-3). **APPROACH:** a card per agent registered by
runme; every arc-011 post is verified against the poster's card; a card changes only by an old-key statement or runme.
2.17.c **ALGORITHM:**
```text
card = {schema: termlink.key-card/1, agent, project, signing_key, encryption_key, instances, seq, by: runme|old-key}
hub, on post of an arc-011 message type to inbox:*:
 1  c = card(m.sender); none -> refuse "no key card" (arc-011 types only; other channels unchanged, NID)             # FCW-135
 2  verify(c.signing_key, m) else refuse, admin_log, needs-attention                                                  # FCW-133
key.replace statement {schema: termlink.key-replace/1, agent, old, new_signing, new_encryption, at} signed by old:
 1  require verify(card.signing_key, stmt) and stmt.old == card.signing_key; else refuse, log                         # FCW-134
 2  card.seq += 1; persist atomically; admin_log; attention("key replaced")
runme provisioning: writes or replaces a card (root-only socket), logged
```
2.17.d **INVARIANTS:** after B-6 every stored arc-011 post verified against a card at the hub and again at the receiver
(AL-23). 2.17.e **FAILS CLOSED WHEN:** 2.17.e.1 **FCW-133** [AFC-33; E-69] the signature does not verify against the
card → refused. 2.17.e.2 **FCW-134** [new; E-69] a replacement not signed by the current key and not a runme act →
refused. 2.17.e.3 **FCW-135** [new; ZR-1] no card for the poster → refused (arc-011 message types only).
2.17.f **EDGE CASES:** (1) a stolen old key can replace the card (RR-2 class; revocation AL-21 is the answer); (2) crash
between persist and log: start-up compares card `seq` with the last logged `seq`; (3) concurrency: card writes
(replacement and runme provisioning) are compare-and-set on `seq` under the hub's per-agent card lock; of two
replacements signed by the same old key, the first moves the card and the second no longer verifies against the
current key (FCW-134); a post is verified against the card it reads under that lock, so it meets either the old or the
new card, never a half-written one. 2.17.g none. 2.17.h ed25519 via AL-20
(schema domain). **OBS:** OBS-59 hub refusals; card `seq`.
```

### Pseudo-code AL-34 — signed dead letter for a retired instance (verbatim, whole section)

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:658-677`

```text
### 2.18 AL-34 — Signed dead letter for a retired instance (K-15; ARC4 6.6.6; R-44, R-60, R-62; B-6)

2.18.a **SERVES:** SI-26, R-44, R-62; TH-54; AFC-34.
2.18.b **APPROACH:** provisioning retires an instance; the hub writes a signed, sequenced DEAD; a sender verifies it
before ending a conversation.
2.18.c **ALGORITHM:**
```text
runme retire <project> <instance>: hub writes {schema: termlink.dead/1, hub_id, project, instance, seq: project_seq + 1,
                                              at, reason} signed (AL-20); admin_log; attention
sender, polling a conversation to instance i:
 1  d = DEAD record for i, if any
 2  require verify(pin, d) and d.hub_id == declared home hub and d.seq > last_dead_seq(project) seen ... else refuse, flag  # FCW-136
 3  conversation state DEAD (seq shown); stop polling; never redirect to another instance (SI-26)
```
2.18.d **INVARIANTS:** only a hub-signed DEAD ends a conversation (R-60: only the home hub may say DEAD).
2.18.e **FAILS CLOSED WHEN:** 2.18.e.1 **FCW-136** [AFC-34; E-70] the DEAD is unsigned, signed by another key, from
another hub, or its sequence does not rise → refused and flagged; polling continues.
2.18.f **EDGE CASES:** DEAD from liveness alone is PH-3 (R-60.a); crash: the record is atomic; concurrency: single
writer, runme (one process), and `seq = project_seq + 1` is taken under the hub's per-project lock; a sender only reads
DEAD records. 2.18.g none. 2.18.h AL-20.
```

### Pseudo-code AL-23 — receive admission (receiver half of the check: step 4, FCW-98)

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:282-330`

```text
### 2.7 AL-23 — Receive admission: the order of checks before STORED (K-11, K-9b, K-15; ARC4 6.3.1, 6.3.2, 6.3.4, 6.2.5, 6.4.4; B-5, B-6, B-7, B-12)

2.7.a **SERVES:** SI-3, SI-5, SI-11, SI-15, SI-19, SI-1 (ids), SI-27, R-56; TH-28, TH-29, TH-30, TH-36; AFC-3,
AFC-22, AFC-23, AFC-25, AFC-26, AFC-33.
2.7.b **PROBLEM:** a message is either STORED (a promise) or refused with a reason; floods wait at the sender; nothing
is dropped. **APPROACH:** a fixed order: cheap structural checks first, then authenticity, then limits, then the blob,
then durability; only the last step reports STORED.
2.7.c **ALGORITHM:**
```text
admit(raw, topic) at the receiver:                     # the pull loop [B]; every refusal is recorded and noticed
 1  version: unknown wire version -> refuse "unsupported version" [B]                                         # FCW-97
 2  strict_json; inline payload <= 8 KiB (PN-7) else QUOTA(inline)                                            # FCW-102
 3  m.id matches ^[0-9a-f]{32}$ and the conversation id is a plain name else refuse                            # FCW-100
 4  key = key card / roster entry for m.sender (AL-33); verify(m, topic) [B SI-3] else refuse                  # FCW-98
 5  if revocation.check(m.key): refuse "revoked", signed notice to the sender (AL-22)                          # FCW-99
 6  if m.hop > 3: refuse "hop limit" with a notice (V-4)                                                       # FCW-101
 7  w = window.check(m) [B T-3362 / AL-36]: a CHECK only, no state written (CR-6.16); refuse or hold as AL-36
 7a if a copy with m's identity triple (R-51) is in the store: on_duplicate(m, copy) (06 AL-14, CR-6.11); return
                                     # CR-6.16: dedupe before quota, so a duplicate never takes a quota slot  (FCW-207)
 8  class = receipts if m is a receipt; own_project if m.sender.project == MY_PROJECT; general otherwise
    LIMITS (CR-6.17, integers that partition the aggregate of 256 messages and 64 MiB):
        receipts 26 messages, 6 MiB; own_project 51 messages, 13 MiB; general 179 messages, 45 MiB     # FCW-208
    if m is not a receipt and open(m.sender) >= 32: QUOTA(per-sender)                                          # FCW-103
    if class.count >= LIMITS[class].count or class.bytes + size(m) > LIMITS[class].bytes:
        QUOTA(aggregate or reserved)                                                                           # FCW-103, FCW-104
 9  if m.blob: fetch and check (AL-27), WITHOUT store.lock (CR-6.15, 06 3.2.a); Err -> refuse              # FCW-114..116
10  with store.lock:                                               # one locked step (CR-6.16)
        re-run 7a and 8 against the current store (a concurrent pull may have stored the copy or taken the slot)
        store.write(m) with checksum, fsync file and directory; Err (disk full, I/O) -> not STORED,
            nothing below committed                                                                            # FCW-105 [B]
        commit w (seen, held_since, up_to of AL-36) and the class counters; chain_append(STORED)
11  after releasing store.lock: hub.stage(m, STORED) [B]; after_stored(m) (06 AL-14)       # CR-6.15: no RPC under the lock
QUOTA(limit): not STORED; notice(m.sender, QUOTA, {limit}); attention("quota", coalesce = (sender, tick)); the
              hub copy stays unacked, so the sender's ladder keeps it                                           # nothing dropped
```
2.7.d **INVARIANTS:** (0) (changed in v0.3 by CR-6.15, CR-6.16) `store.lock` is held only for step 10's local write
and commit, never across the blob fetch or a hub call; the sequence window, the quota counters and STORED are
committed together or not at all; (1) STORED only after step 10's fsyncs; (2) receipts never count against a sender quota and
general traffic never uses the reserved shares; (3) every refused message has a reason the sender can read; (4) no
message is deleted at the receiver because of a limit.
2.7.e **FAILS CLOSED WHEN:** 2.7.e.1 **FCW-97** [new; E-80, E-100] unknown wire version → refused with a reason.
2.7.e.2 **FCW-98** [AFC-33; E-69, E-93] signature fails or the signer has no card → refused. 2.7.e.3 **FCW-99**
[AFC-22; E-39] the key is on the verified list → refused, signed notice. 2.7.e.4 **FCW-100** [AFC-3; E-3, ZR-1] the
message id is not 32 lowercase hex → refused at the receiver, so a bad id can never reach a doorbell. 2.7.e.5
**FCW-101** [AFC-26; E-53] hop over 3 → refused with a notice. 2.7.e.6 **FCW-102** [AFC-25; E-50] inline over 8 KiB →
not STORED, QUOTA notice. 2.7.e.7 **FCW-103** [AFC-25; E-50, E-51] 32 open from one sender, or the general budget of
256 messages or 64 MiB is full → not STORED, QUOTA notice, coalesced entry. 2.7.e.8 **FCW-104** [AFC-25; E-51] a
reserved share (receipts 10%, own project 20%) is full → the same, for that class only. 2.7.e.9 **FCW-105** [new; E-122]
the store write or a fsync fails → not STORED (changed in v0.3 by CR-6.16: and neither the window nor the quota
```

## Cited parameters/invariants

### AL-20 — canonical form, digest and signature of signed hub state (used by key-replace and DEAD: "ed25519 via AL-20 (schema domain)", "signed (AL-20)")

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:140-176`

```text
### 2.4 AL-20 — Canonical form, digest and signature of signed hub state (ARC4 6.2.1; FP 10.3.2.c; test vectors)

2.4.a **SERVES:** SI-11, SI-12, SI-21, SI-3 (envelopes); TH-26, TH-58; AFC-22.
2.4.b **PROBLEM:** a signed JSON object must verify identically whatever its byte encoding, and a signature made for one
purpose must not verify for another. **APPROACH:** RFC 8785 (JCS) on a restricted value set, SHA-256 of the canonical
bytes, ed25519 over a domain-separated string `<schema>\n<hex digest>`.
2.4.c **ALGORITHM:**
```text
canonical(obj):
 1  obj passes strict JSON (06 3.3): UTF-8, no duplicate keys, depth <= 8, size <= 1 MiB, numbers only integers in
    [-(2^53-1), 2^53-1] (no fraction, exponent or -0)                                                         # FCW-88
 2  remove the fields "digest" and "signature" from the top level
 3  serialise by RFC 8785: object members sorted by UTF-16 code units of their names, no insignificant whitespace,
    strings with the minimal JSON escapes (\" \\ \b \f \n \r \t, other controls \u00xx lower-case), integers in
    shortest decimal form
digest(obj)  = lower-case hex SHA-256 of canonical(obj)
sign(k, obj) = ed25519_sign(k, utf8(obj.schema + "\n" + digest(obj)))
verify(pub, obj):
 1  require obj.digest == digest(obj)                     # recomputed, never trusted from the object
 2  require ed25519_verify(pub, utf8(obj.schema + "\n" + obj.digest), obj.signature)                            # FCW-89
```
2.4.c.1 **Test vectors** (computed by `.context/working/t3355/vectors.py`; public RFC 8032 test key 1, public key
`d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a`; never an estate key):

| Id | Input | Expected |
|---|---|---|
| V-REV-1 | revocation list, `seq` 7, one entry (fields below) | canonical 436 bytes, digest `044cfb1cf8c79cc60832cce9145895ff1bca2338d28c447823e687a3b6fe5767`, signature `02739ad55f80d54104dc59466efe9671ec5493df329023a71468926f193f76f6173b449790f2836e57af6bfc7ef8699c6f2f28f38a718846b4d4a0c8b2c7830e` |
| V-REV-2 | the same object with keys reordered, spaces added and `claude-termlink-alt` | canonical bytes and digest equal to V-REV-1 |
| V-HEAD-1 | revocation head `{schema: termlink.revocation-head/1, hub_id, seq: 7, digest: <V-REV-1 digest>}` | canonical 151 bytes, digest `cc363257e39103a2d922e6ec0000810659449f3c0098e0471ab8c9932f2b24dd`, signature `ea99514e54465969c9ff89430be6a72c3cde1a4e2bdaaa2a2c53f7db5ac44eedc70257e8c8d06b8336b0376f7f798a27daf215292901a0548fc20d98934c8202` |
| V-FWD-1 | forward envelope (06 AL-14), generation 12, hop 1 | canonical 320 bytes, digest `7a7e9bf16ae3f54de6f1007b0ec917cac6f613c9dd5ce7eed6e9a38764af421d`, signature `29495852a7b15ef34240f9471f42f49ae3c3090e76cd37eb0a0277cbbff3d0b0ffc471503e8b9839a5cd2abd9431e5ae31a9add9b6b812014b117ce834dcdc0a` |
| V-NEG-1 | V-REV-1 with `"seq":7.0`, or `"seq":7e0` | refused at step 1 (FCW-88), never hashed |
| V-NEG-2 | V-REV-1 with `"seq"` twice | refused (duplicate key) |
| V-NEG-3 | V-REV-1 signed over the bare digest (no `termlink.revocation/1\n` prefix): signature starting `3b0af0c2379828c0e02d6a1ee48ffa95` | does not verify (FCW-89) |
| V-NEG-4 | V-REV-1 with `seq` 9007199254740992 (2^53) | refused (out of range) |

2.4.c.2 V-REV-1's canonical bytes, exactly: `{"entries":[{"agent":"claude-termlink-alt","by":"operator","event":
"admin-revoke","key":"ed25519:3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29","project":
```

### FCW register rows (06 section 6) and AFC map

`interactive-agent-communication-06-pseudo-code.md:1569-1569`

```text
| FCW-98 | 06a | AL-23 | AFC-33 | E-69, E-93 | signature fails or the signer has no card → refused. |
```

`interactive-agent-communication-06-pseudo-code.md:1596-1607`

```text
| FCW-125 | 06a | AL-30 | new | ZR-1 | the consumer list cannot be read → status `consumers: unknown` (never "one"). |
| FCW-126 | 06a | AL-31 | AFC-32 | E-67 | a sweep, retention change or trim would remove an open message → refused and logged. |
| FCW-127 | 06a | AL-31 | new | ZR-1 | finality cannot be read → refused. |
| FCW-128 | 06a | AL-31 | new | ZR-1 | the admin-log row cannot be written → the RPC is refused. |
| FCW-129 | 06a | AL-31 | new | E-67 | past a ceiling → final first; open only as a last resort, loudly. |
| FCW-130 | 06a | AL-32 | new | E-68 | the token lacks `control` (inject) or `execute` (exec) → refused **[B]**. |
| FCW-131 | 06a | AL-32 | new | E-68 | an owner-only session is called by another identity → refused. |
| FCW-132 | 06a | AL-32 | new | ZR-1 | the use-log row cannot be written → the verb is refused. |
| FCW-133 | 06a | AL-33 | AFC-33 | E-69 | the signature does not verify against the card → refused. |
| FCW-134 | 06a | AL-33 | new | E-69 | a replacement not signed by the current key and not a runme act → refused. |
| FCW-135 | 06a | AL-33 | new | ZR-1 | no card for the poster → refused (arc-011 message types only). |
| FCW-136 | 06a | AL-34 | AFC-34 | E-70 | the DEAD is unsigned, signed by another key, from another hub, or its sequence does not rise → refused and flagged; polling continues. |
```

`interactive-agent-communication-06-pseudo-code.md:1720-1721`

```text
| AFC-33 | FCW-98, FCW-133 |
| AFC-34 | FCW-136 |
```

### SI-3, SI-16, SI-26 (03 floor table and 02 invariant table)

`interactive-agent-communication-03-floor-and-phasing.md:96-96`

```text
| SI-3 | Every message signed by the sending agent's own key, verified at receiver and hub | R-75, R-34.e | PH-1 | PH-1 | A for same-host attackers | R CR-2 |
```

`interactive-agent-communication-03-floor-and-phasing.md:109-109`

```text
| SI-16 | Exactly one sidecar holds an agent's store lock; exactly one identity consumes an inbox | R-49 | PH-1 | PH-2 (consumer visibility on the hub) | none | E |
```

`interactive-agent-communication-03-floor-and-phasing.md:119-119`

```text
| SI-26 | A message to an exact instance reaches only that instance and fails loudly if it has ended | R-34, R-62 | PH-1 | PH-4 (hand-over fence) | none | E |
```

`interactive-agent-communication-03-floor-and-phasing.md:677-677`

```text
| SI-3 signatures | A process of any agent reads a sibling's key and signs as it | A key is readable only by its own account |
```

`interactive-agent-communication-02-threat-model.md:1337-1337`

```text
| SI-3 | Every message carries a signature by the sending agent's own key over its canonical form, verified at the receiver sidecar and at the hub; an unsigned or wrongly signed message is never stored, flagged or handed over | Forge a message with the shared host key and with a sibling agent's key id; require refusal and an entry | TH-1, TH-8, TH-19, PR-1 |
```

`interactive-agent-communication-02-threat-model.md:1350-1350`

```text
| SI-16 | Exactly one sidecar holds the store lock of an agent; a second is refused and shown; exactly one identity consumes an inbox | Start a second sidecar and a second consumer; require refusal | R-49, BP-15 |
```

`interactive-agent-communication-02-threat-model.md:1360-1360`

```text
| SI-26 | A message addressed to an exact instance reaches only that instance and fails loudly if it has ended | The existing standing test of R-62 | TH-54, R-34, R-62 |
```

`interactive-agent-communication-02-threat-model.md:1373-1373`

```text
| SI-3 | Proposal | CR-2 | A same-user process that read the key (RR-2) | Can run: forge with the host key and a sibling key |
```

`interactive-agent-communication-02-threat-model.md:1396-1396`

```text
| SI-26 | Existing (R-62) | none | none beyond RR-4 | Can run (the standing test of R-62) |
```

### Threats TH-1, TH-8, TH-54, TH-56; BP-3; PR-1

`interactive-agent-communication-02-threat-model.md:750-753`

```text
TH-1 **Sender impersonation by the shared host key.** TB-3, TB-5, TB-6, TB-7 · S · ADV-2a, ADV-6
15.2.1 Scenario: every agent on the host signs with one host key (`RQ` 11 item 5), so a session of project Q sends as project P, or an agent as its manager; the receiver cannot tell. R-34.e (2) deliberately does not cover this.
15.2.2 L high, it is the documented state and the reviewers named it (`RV1` point 17). I high, every allow-list and urgent decision rests on "verified sender".
15.2.3 Countermeasure: PR-1 and SI-3 (per-agent key, every message signed and verified). Residual against a same-user attacker who reads the key: RR-2.
```

`interactive-agent-communication-02-threat-model.md:785-788`

```text
TH-8 **Forged registration or presence card.** TB-5, TB-12 · S · ADV-2a, ADV-2b, ADV-11
15.2.22 Scenario: a session registers as project P role "main", or posts a presence heartbeat for an agent that is not running, to receive P's requests or to make a dead agent look alive.
15.2.23 L medium (presence is a topic; registration is by session claim today). I high (receive another's mail).
15.2.24 Countermeasure: PR-1, PR-1a, PR-5 (registrations signed by a known agent key, cards signed by the hub with sequence and time-to-live), SI-12, SI-20.
```

`interactive-agent-communication-02-threat-model.md:795-798`

```text
TH-56 **Key registration or rotation takeover.** TB-5, TB-12 · S, E · ADV-2a, ADV-11
15.2.28 Scenario: an attacker registers a new key for an existing agent (claiming a lost key), and from then on messages signed by the new key are accepted as that agent.
15.2.29 L low if rotation needs the old key, medium if a session claim suffices. I high (full identity takeover).
15.2.30 Countermeasure: PR-1a: replacing an existing agent's key needs a signature by the old key or operator approval bound to a digest (SI-13).
```

`interactive-agent-communication-02-threat-model.md:1081-1084`

```text
TH-54 **A false DEAD from a compromised or mistaken home hub.** TB-5, TB-12 · E, T · ADV-11, ADV-6
15.11.4 Scenario: only the home hub may say DEAD (R-60). A compromised or confused home hub declares a live copy DEAD; senders get dead letters, and a **resume** of that instance becomes allowed (R-65), so two processes now hold the same transcript.
15.11.5 L low to medium (a hub is a single component and the 2026-10-04 stray hub shows hubs go wrong). I high (a duplicate session of one conversation answers twice and splits its context; this is the 2026-10-06 wrong-session class).
15.11.6 Countermeasure: DEAD is signed and sequenced; PR-15: before a **resume** the host-local supervisor verifies that no process holds that session (a hub's word alone never starts a copy). **PR-15 does not apply to role succession.** The takeover of "main" follows R-67 and R-68 unchanged: lease lapse, quiet period, cooldown, fencing generation, old holder told. R-68 test (2) requires a visible takeover from a stuck holder whose process is alive, so a "no process holds it" check on a takeover would forbid the ruled behaviour. Protection for succession is the fencing generation, the old holder being told, and the operator-visible entry (R-67), plus SI-20 (who may become eligible). Residual: RR-4.
```

`interactive-agent-communication-02-threat-model.md:1241-1241`

```text
| BP-3 | Direct `channel post` and `subscribe` to inbox topics with the hub secret | A-1: post as the host identity, read content | The T-1427 host signature only | open | posting is closed in effect by SI-3 (unsigned or wrongly signed is refused); reading of envelope metadata stays open (RR-6); content is encrypted per message to its authorised readers (PR-33, B′, ruled, 22.18) |
```

`interactive-agent-communication-02-threat-model.md:1433-1433`

```text
| PR-1 | Every message signed by the sending agent's own key | 6.3.a | CR-2 |
```

### R-60.a (cited by AL-34: "only the home hub may say DEAD"; signed + sequenced DEAD per CR-12)

`interactive-agent-communication-01-requirements.md:922-922`

```text
R-60.a [R] Ruled (OD-11 A with location rules 2a-2f): each home hub keeps a card per project, filled only from authenticated registrations it observed (project id, roles, live copies, liveness, version). Hubs exchange cards, never messages, and never re-announce another hub's cards. Only the home hub MAY say dead; anything it cannot confirm is unknown. Identity is the project id minted once by the project's framework and written in `.framework.yaml`; folder path and host are observed attributes (cf. T-2815). (2b) A folder renamed or moved on the same host keeps its project id; the card updates. (2c) A move to another host: the same id registers at the new host's hub, which becomes the home hub; the old hub marks the project "moved to hub X" and stops answering for it; senders follow the new card; no silent redirect. (2d) A copy with both running is by default a SECOND INSTANCE of the same project; a copy meant as a new project (a fork) MUST re-mint its own id. (2e) The first time a hub sees an id in a new place while the old one is alive, it does not guess: it flags "same project id seen at X and Y" to the operator and the cockpit, and treats the copy as a second instance until declared a fork. (2f) Runtime ids are never reused. [R] (changed in v0.5 by CR-8, ruling 2026-10-09 OQ-8 D″) Admission and authenticated advertisements are added by R-77: a fleet roster, cards signed by the originating hub's key, a project root key, signed hub-key rotation and "moved to" statements, a no-guess rule where no prior binding exists. Home-hub binding stays as (2e) rules it: the bound hub stays believed and a competing claim is a flagged second instance. v0.4.1 text, which said nothing about who may join or how a card is authenticated, is unchanged. [R] (changed in v0.5 by CR-12, ruling 2026-10-10 D5) "Only the home hub MAY say dead" now means a DEAD statement that is signed by the home hub's key and carries a sequence number; a receiver refuses an unsigned or unsequenced DEAD and one whose sequence is lower than the last seen (R-65.a, R-65.e). [R] (changed in v0.5 by CR-18, ruling 2026-10-09 OQ-8 D″) A request to join the fleet, to home a project, to re-home, to replace a key or to re-pin is a Tier 0 event decided by an approval route (R-72); it is not decided by the card exchange.
```

### P7-2 (the built key-register ruling the card extends)

`arc-011-ph2-architecture.md:144-144`

```text
| P7-2 | How the pair's signing keys are registered (PR-1a) | A operator-held register on this host now, hub card in P17; B hub-side agent card now; C defer | **Ruled 2026-10-13: A** — `runme.sh` writes `/etc/termlink-sidecar/keys/<agent>.pub` (root-owned) and builds the rosters from it; in production a missing key file or a key that differs from the register is a refused start (no silent re-mint); a key changes only through a runme action naming the change. Hub card + old-key-signed replacement added to P17 |
```

`arc-011-ph2-architecture.md:270-273`

```text
1. **Key register (P7-2, PR-1a part).** A production sidecar refuses to start when its key file is missing (it
   never mints one; PH-1's `load_or_create` stays for test estates), when its config names no `key_register`, or
   when the key differs from the register entry `/etc/termlink-sidecar/keys/<agent>.pub`. The hub-side card and
   old-key-signed replacement are row P17.
```

PN-n: none of the B-6 passages cite a PN-n parameter (AL-23 cites PN-7, the 8 KiB inline cap, which is B-7's). K-n: K-15 only (quoted above). No V-n value is cited by AL-33/AL-34.

## Current code map

### 1. Hub: where `channel.post` is handled and what it checks today

1a. Dispatch: `crates/termlink-hub/src/router.rs:93-96` routes `CHANNEL_POST` to `crate::channel::handle_channel_post(id, &req.params, peer_addr)`. Before that, `crates/termlink-hub/src/server.rs:2025-2080` does the scope check, the founding-verb gate (B-4, T-3702) and the admin-row path (`is_admin_method` -> `dispatch_admin`, `server.rs:2066`, `server.rs:1865`). `channel.post` is not an admin method (`rpc_audit.rs:613-623`).
1b. Handler: `crates/termlink-hub/src/channel.rs:838` `handle_channel_post` -> `channel.rs:863` `handle_channel_post_with_peer`. Required params: `topic`, `msg_type`, `payload_b64` (≤ `MAX_PLAINTEXT_FIELD_BYTES`), `ts`, `sender_id`, `sender_pubkey_hex`, `signature_hex`. Optional: `artifact_ref`, `metadata` (string map), `client_msg_id`, `stamp_rx`.
1c. **The signature check already exists but uses a self-asserted key.** `channel.rs:906-952`: the hub parses `sender_pubkey_hex` and verifies `signature_hex` over `canonical_sign_bytes(topic, msg_type, payload, artifact_ref, ts)` (`termlink-protocol/src/control.rs:367`, a length-prefixed binary layout, not JCS). Failure -> `CHANNEL_SIGNATURE_INVALID` (-32012, `control.rs:434`). `channel.rs:955-968` (T-1427): `sender_id` must equal `fingerprint_of(pubkey)` (first 16 hex of sha256, `termlink-session/src/agent_identity.rs:193`), else `CHANNEL_IDENTITY_MISMATCH` (-32014, `control.rs:444`). So any freshly generated key passes: there is no binding between a key and an agent name or project. That missing binding is exactly what B-6's card adds.
1d. The verified signature is stored on the envelope (T-3486, `channel.rs:1101-1116`, `EnvelopeSignature {sender_pubkey_hex, signature_hex}`) and returned on read when `with_signature` is set (`channel.rs:2834-2841`).
1e. `metadata` is outside the signed bytes ("routing only", `channel.rs:970-972`). The hub stamps `observed_addr` and `hub_rx` itself (`channel.rs:984-990`).
1f. Hub-originated envelopes (`hub:event.broadcast`, `hub:inbox.deposit`, `hub:topic_lint`) carry `signature: None` (`channel.rs:209-365`). A card check must exempt or never see them.
1g. `server.rs:1750-1755` and `server.rs:1948-1956`: `params.sender_id` feeds the governor's sender key (`governor.rs:86-105`). Dedupe is namespaced by `sender_id` (`dedupe.rs`).

### 2. Sender identity carried by posts today

2a. Agent key: `termlink_session::agent_identity::Identity` (`agent_identity.rs`). Key file resolution: `TERMLINK_IDENTITY_FILE` -> `TERMLINK_IDENTITY_DIR/identity.key` -> ... -> shared host `identity.key` (`agent_identity.rs:269-274`); per-agent `<base>/identities/<id>.key` (`agent_identity.rs:254-262`). `Identity::sign`, `verify`, `public_key_hex`, `fingerprint` (`agent_identity.rs:161-187`), `check_detached_hex` (`:213`).
2b. Sidecar message body: `crates/termlink-sidecar/src/wire.rs:39-53` `Msg {from (fingerprint), from_agent, from_instance, to_agent, to_instance, conv, msg_id, seq, priority, ts_ms, body}` plus `v`, `kind`; canonical = sorted-key JSON (`wire.rs:166-201`; note: sorted by `BTreeMap<&String>` = byte order, not RFC 8785 UTF-16 order, and no strict-JSON number rules). `MSG_TYPE = "sidecar.msg.v1"`, `STAGE_TYPE = "sidecar.stage.v1"` (`wire.rs:18-21`).
2c. Signing: `wire::sign` (`wire.rs:213`) signs `canonical_sign_bytes(topic, MSG_TYPE, payload, None, ts)` with the agent's own key. That is the same byte string the hub verifies in 1c, so the sidecar's transport signature IS the hub post signature (`signature_hex == metadata.sidecar_sig`). Stages: `wire::sign_stage` (`wire.rs:456`); `Stage {from, from_agent, to_agent, conv, msg_id, stage, ts_ms, chain_head}` (`wire.rs:384-396`).
2d. The posted params (`crates/termlink-sidecar/src/send.rs:397-456`, `post_params` `send.rs:221-240`): `topic = inbox:<declared_hub_id>/<to_agent>` (`send.rs:207`), `sender_id = fingerprint`, `sender_pubkey_hex` = agent key, `signature_hex`, `client_msg_id = msg_id`, metadata `{conversation_id, sidecar_msg_id, sidecar_digest, sidecar_pk, sidecar_sig}`. `post_stage` (`send.rs:511-537`) is the same shape for `sidecar.stage.v1`. `post_note` (`send.rs:480-509`) posts `sidecar.liveness.v1` to `liveness:<hub_id>/<agent>` (`liveness.rs:44`, `:71`).
2e. **The claimed agent name (`from_agent`) and instance are only inside the signed payload**, not in any hub-level param. The hub's own `sender_id` is the key fingerprint, so `card(m.sender)` keyed by `sender_id` would be circular. The hub has to parse the `sidecar.msg.v1` / `sidecar.stage.v1` payload (it already does so for stages in `mail_guard.rs`) to get the agent name it looks the card up by.
2f. Receiver check (built PH-1, SI-3 [B]): `wire::check_signer` (`wire.rs:292-313`) requires that the signer key hashes to `from`, that `from` is in the `Roster`, that the key equals the roster key, and that the signature verifies; `wire::verify` (`wire.rs:316`) also enforces `to_instance == my_instance` -> `Refusal::WrongInstance` (SI-26). The roster is a fingerprint->key map built from config `roster: Vec<hex>` (`daemon.rs:110-121`, `roster_of`); it is used in `pull.rs:383`. There is no agent-name->key binding at the receiver either: the roster says "this key may send" and does not say "this key is agent X of project P".
2g. Key register (P7-2, built T-3438): `daemon.rs:46-50` `key_register`; `daemon.rs:124-160` `load_identity`: production refuses a missing key file, a missing `key_register` or a key that differs from `/etc/termlink-sidecar/keys/<agent>.pub`. The B-11 X25519 register is the same pattern (`enckey.rs`, `/etc/termlink-sidecar/keys/<agent>.x25519.pub`, root-owned check). These two register files are the natural source for the card's `signing_key` and `encryption_key` (P-31 publishes the cards).

### 3. Registration and existing "card" concepts

3a. `session.register_remote` (`router.rs:81`, `:771-...`) registers TCP sessions (display_name, host, port, roles, tags, capabilities) in an in-memory remote store; with a registration claim (T-3760) via `route_session_registration` (`router.rs:1262`). It carries no signing key and no agent/project binding. It is unrelated to agent key cards.
3b. "card" today means the `session.whoami` identity card (`router.rs:268`) and the R-53 reachability card (04 6.7.2, B-14). **No agent key card, no `termlink.key-card/1`, `termlink.key-replace/1` or `termlink.dead/1` schema exists anywhere in `crates/`** (grep: zero hits). There is also no AL-20 JCS canonicaliser in `crates/` yet (P-15/B-5 has not landed); the B-6 statements need one.
3c. Hub signing key (T-3366, B-1): `crates/termlink-hub/src/hub_identity.rs` (`ID_FILE = "hub.id"`, `KEY_FILE = "hub.signing.key"` at `:26-27`; `HubIdentity::signing_key()` `:43`; `signing_key_fingerprint()` `:38` = `ed25519:<sha256 hex of pubkey>`; `current()` `:74`; `load_or_mint` `:329`). `hub.version` (`router.rs:893-912`) publishes `hub_id` and `signing_key` **as that fingerprint only, never the public key**.
3d. Sidecar pin (T-3366 S3): `crates/termlink-sidecar/src/hub_key_pin.rs`. It pins the `hub.version` fingerprint on first local-socket contact; `verified_hub_key` (`hub_key_pin.rs:195-199`) is "the gate those units call", and the module doc (`:5-6`) names P-16 dead letters as a consumer. It returns the fingerprint string, not a key.

### 4. Dead letters, retired instances, instance ids today

4a. Sidecar chain event `DEAD_LETTER` (`send.rs:17`, `:52`, written at `:703`): "the hub rejected it `POISON_THRESHOLD` times". This is a transport poison-drop, **not** the R-44 DEAD state. `api.rs:1167` maps it to a label. `PRE_HANDOVER_VOCAB` (`api.rs:44-57`) has no `DEAD` word; the five R-44 states are partly there (WAITING, WAITING FOR RECIPIENT, STUCK, UNKNOWN).
4b. Session offline queue `dead_letters` table (`termlink-session/src/offline_queue.rs:136-152`, `dead_letter()` `:362`) is the T-2243 poison store, also unrelated.
4c. Instance ids: the sidecar's own `instance` (`daemon.rs:28`, `pull.rs:62`); the operator-written address book `Book {peers: agent -> instance, main}` (`book.rs:17-23`, `resolve` `book.rs:73-100`, "a different id is refused, never redirected"). Instances live only in sidecar config; **the hub has no registry of instances and no notion of retired**. Nothing named `retire`/`retired` exists in `crates/`.
4d. Hub finality: `crates/termlink-hub/src/mail_guard.rs:1-11` (B-3, T-3701) already lists `DEAD` among the final stages it honours for sweep refusal, so a DEAD outcome has to be visible to it (as a stage record or a separate check).
4e. Admin log: `rpc_audit.rs:663` `append_admin_row_at`, rows with `admin: true` (B-3). AL-33/AL-34 `admin_log` can reuse this. The hub has no needs-attention store of its own (only `rpc_audit.rs` and `mail_guard.rs` mention attention), so AL-33's "needs-attention" at the hub has no existing sink.
4f. Hub socket access: unix-socket peers are accepted when same-uid (`server.rs` `decide_unix_peer`, around `:964-975`); scope comes from the token. There is no separate "root-only socket".

### 5. AEF side (affects the scope of the check)

5a. The AEF Python sidecar (`.agentic-framework/lib/sidecar/`) also posts to `inbox:*` topics through `termlink channel post` with other msg types (e.g. `sidecar.receipt`, `receipts.py:231-232`; chat posts), signed by the host-shared identity. A card check over all of `inbox:*` would refuse that working rail.

## Gaps and contradictions

1. **Card lookup key is undefined and the obvious one is circular.** AL-33 says `c = card(m.sender)`. At the hub, `sender_id` is by definition `fingerprint_of(sender_pubkey_hex)` (T-1427), so a card keyed by it proves nothing. The card is "per agent", so the lookup has to use the claimed agent name `from_agent` inside the `sidecar.msg.v1`/`sidecar.stage.v1` payload, and the hub has to parse that payload. The spec never says this. Recommendation: key cards by `(project, agent)`, read `from_agent` from the payload, and require `card.signing_key == sender_pubkey_hex`. The existing signature verify (1c) is then reused unchanged.
2. **"verify(c.signing_key, m)" names no bytes.** AL-20 (JCS, `<schema>\n<digest>`) governs signed hub *state* (key-replace, DEAD). Posts are signed over `canonical_sign_bytes` (binary, `control.rs:367`). Read literally, AL-33 step 2 could mean re-signing posts under AL-20. The coherent reading is that the post check keeps the existing transport signature and adds key==card. AL-20 then applies only to `termlink.key-card/1`, `termlink.key-replace/1` and `termlink.dead/1`.
3. **Project binding is not on the wire.** R-75.e: "a session of project Q that signs with its own key and asserts it is project P, then the message does not verify as P's agent". `Msg` has no `project` field, and the inbox topic is `inbox:<hub>/<agent>` with no project. So the card's `project` is checked only through the agent name. That holds only if agent names are unique per hub. Open question: are agent names hub-unique, or must the project be added to the payload (a wire v2 change)?
4. **Which "arc-011 message types".** FCW-135 limits the refusal to "arc-011 message types only", but no list exists. The code has `sidecar.msg.v1`, `sidecar.stage.v1` (inbox) and `sidecar.liveness.v1` (liveness topic). AEF's `sidecar.receipt` and chat posts also travel on `inbox:*` (5a). If the gate is keyed on topic rather than on msg_type, it switches off a working rail, which breaks the operator's never-decommission (NID) rule. The type list must be pinned explicitly.
5. **Roll-out ordering.** Cards are published by P-31 (a deployment after P-16). If the hub check is on by default, every sidecar post on .107 gets FCW-135 until P-31 runs. The plan's rollback ("check off at the hub; cards kept") implies a config switch, and it has to default to off, or to log-only, until cards exist.
6. **DEAD cannot be verified with what the sidecar pins.** AL-34 says `verify(pin, d)`, but the pin and `hub.version` hold only `ed25519:<sha256(pubkey)>` (3c, 3d), and verifying needs the 32-byte public key. Either `termlink.dead/1` has to carry the hub public key (the sender then checks sha256(pk) == pin), or `hub.version` has to publish the key. Neither is specified.
7. **Where DEAD lives and how a sender finds it is unspecified.** 04 6.6.6 says "a sender's conversation mail to it yields DEAD"; AL-34 has the sender "polling" for a "DEAD record for i, if any". None of the following is stated: which topic or RPC holds DEAD records; whether the hub refuses or answers a post addressed to a retired instance (the instance is only in the payload `to_instance`, and the topic is per agent); and how `mail_guard` (4d) learns the message is final. The hub also has no instance registry (4c). The `instances` field on the card is presumably what `runme retire` edits, but the spec does not tie the two together.
8. **`seq > last_dead_seq(project)` is strict per project.** A sender that meets the same DEAD record again, in a second conversation or after a restart, would see an equal seq and refuse it (FCW-136). It looks intended to mean "not lower than the highest seen, and an equal seq only for byte-identical content". R-60.a says "refuses ... one whose sequence is lower than the last seen", which is not strict, so AL-34 (`>`) and R-60.a (refuse only `<`) contradict each other.
9. **"root-only socket" for runme provisioning has no counterpart in code.** The hub socket admits any same-uid peer (4f), and 04 6.7.1 records "agents run as root". A root agent could therefore write or replace its own card through the same path runme uses (the RR-2 class). Which credential makes a call "a runme act" (FCW-134) is not defined. Options: an admin-scope token held only by runme, or a dedicated socket path with mode 0600 owned by a separate uid (not available while agents run as root).
10. **Name clash on dead letters.** The existing sidecar `DEAD_LETTER` chain event (poison-drop after `POISON_THRESHOLD` hub rejections, 4a) and the new R-44 `DEAD` state (signed retirement) are different things. The new state needs its own event name and label, plus a `DEAD` entry in the sender vocabulary (`api.rs:44-57`). Otherwise status will conflate them.
11. **The receiver half does not use cards.** AL-23 step 4 reads "key card / roster entry"; AL-33 2.17.d says every post is verified "at the hub and again at the receiver". Today the receiver uses only the config roster, which binds keys and not names (2f). Open question: does the receiver fetch cards from the hub (that needs a hub-signed card, so B-6 depends on the P-15 style canonicaliser and the pin), or does it keep the roster plus book? E-69.4's "bypassing the hub check, at the receiver" is already met by the roster for a key that is not rostered. It is not met for "post as `fx-a` signed with `fx-b`'s key" when both keys are rostered and the receiver ignores `from_agent`. `check_signer` checks only `fingerprint == from`, and `from_agent` is never checked against the key.
12. **Needs-attention at the hub.** AL-33 step 2 says "refuse, admin_log, needs-attention", and AL-34 runme retire says "attention". The hub has no attention store (4e). Either define one (for example a hub status field) or read this as the sidecar-side attention.
13. **Canonical JSON differs between built and spec.** The sidecar's `canonical_json` sorts keys by byte order and does not enforce the strict-JSON rules. AL-20 requires RFC 8785 (UTF-16 order, integer range, no duplicate keys). For ASCII keys the order is the same, but the B-6 statements must use an AL-20 implementation, not `wire::canonical_json`, or the V-REV vectors will not apply.
14. **Level/tier mismatch inside one unit.** E-69 and E-70 are L4 Tier 2 (live adversarial); E-169 is L2 Tier 1 (a hub fake). P-16's done-criterion "fixture suite for the unit exits 0 with a PASS line per listed E-n" therefore covers both a live-hub level and a fake level. Plan for two harnesses.
15. **Key format strings are inconsistent across docs.** The card has `signing_key` (format not given). The revocation vector uses `"key":"ed25519:<64 hex>"`, which is ambiguous between a public key and a hash. `hub.version` uses `ed25519:<sha256 of the public key>`. Agent `sender_id` is 16 hex of sha256. The register files hold raw 64-hex public keys. The card schema must fix one form (recommended: `ed25519:<64-hex raw public key>`, matching the revocation entry).
