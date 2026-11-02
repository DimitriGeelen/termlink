# Interactive agent communication: step 4, traceability (sub-document of the architecture)

**Task:** T-3353 · **Arc:** arc-011 · **Chain:** arc-011-design, step 4 · **Role:** architect
**Status:** DRAFT v0.1, part of `docs/design/interactive-agent-communication-04-architecture.md` ("ARC4"), reviewed with
it. Card 3.6 and 4.2: every R-n, TH-n, SI-n and FX-n maps to a design element; every element serves an R-n or TH-n.
Elements are ARC4's `K-n` components (5.1) and sections; `[B]` built, `PH-n` the phase where the element lands.

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-11-02 | Four tables: floor items, requirements, threats, elements | T-3353 |

## 1 Floor items to elements (SI-1..SI-31, FX-1..FX-8)

| Item | Element (ARC4 section) | Phase | Probe owner |
|---|---|---|---|
| SI-1 | K-19 doorbell line inside the SI-1 alphabet (6.9.3, DR-6) | PH-2 | B-16 |
| SI-2 | K-18 veto [B] + K-19 re-checks, no attached client (6.9.1-2) | PH-2 | B-16 |
| SI-3 | sidecar signatures [B]; K-15 key card, post-time check (6.6.5) | PH-1 [B], PH-2 | B-6 |
| SI-4 | identity triple + digest [B] | PH-1 [B] | built |
| SI-5 | K-10 window, equivocation (V-6) | PH-2 | B-8 |
| SI-6 | stage records [B]; `up_to` (K-10) | PH-1 [B], PH-2 | B-8 |
| SI-7 | frame by the receiving adapter [B]; other rails PH-4 (RA-9) | PH-1 [B], PH-4 | built; PH-4 |
| SI-8 | policy reads only sender and priority [B] (`policy.rs`); Tier 0 routes K-27 | PH-1 [B], PH-3 | built; PH-3 |
| SI-9 | K-29 circuit credential (12.5) | PH-5 | PH-5 |
| SI-10 | K-29 minting checks (12.5) | PH-5 | PH-5 |
| SI-11 | K-9b revocation, bound one tick, stale rule (6.2) | PH-2 (RK-1), PH-4/PH-5 rest | B-5 |
| SI-12 | K-28 directory, signed cards (12.1) | PH-3 | PH-3 |
| SI-13 | K-27 digest-bound single-use approvals (12.2) | PH-3 | PH-3 |
| SI-14 | three owner-only sockets [B]; status names "same-user separation absent" (6.7.1) | PH-1 [B], PH-2 | B-14 |
| SI-15 | durable store [B]; K-14 reconciliation, host-crash test (6.5) | PH-1 [B], PH-2 | B-10 |
| SI-16 | store lock [B]; K-15 consumer list, bind (6.6.1-2) | PH-1 [B], PH-2 | B-2 |
| SI-17 | nonce in harness record type [B]; doorbell never counts as hand-over (6.9.4) | PH-1 [B] | built |
| SI-18 | receiver-side priority and trust [B] (`policy.rs`, OP-21 B in PH-1) | PH-1 [B] | built |
| SI-19 | K-11 receiver quotas and reserved shares (6.3.1) | PH-2 | B-7 |
| SI-20 | operator pin [B]; lease [B] with DR-9 interim; eligibility from signed profile K-26 (V-10) | PH-2 [B], PH-4 | PH-4 |
| SI-21 | fixed default allow-list [B]; policy store K-26 (V-2) | PH-2 [B], PH-4 | PH-4 |
| SI-22 | hash chain [B]; K-14 daily head compare (6.5.3) | PH-1 [B], PH-2 | B-10 |
| SI-23 | kill switch, fleet form [B]; read before typing (6.9.2) | PH-2 [B] | B-16 (typed path) |
| SI-24 | re-pin record [B] (T-3384); pin override record [B]; Tier 0 trail K-27; break-glass (8.4.4) | PH-0 [B], PH-4 | PH-3/PH-4 |
| SI-25 | key never printed (6.1.4); SI-25 scan on test artifacts | PH-0 | every B |
| SI-26 | exact instance [B]; signed dead letter (6.6.6) | PH-1 [B], PH-2 | B-6 |
| SI-27 | checksums in status [B] (P9) | PH-2 [B] | built |
| SI-28 | blobs under digest, 0600, no peer name (6.4.4) | PH-2 | B-12 |
| SI-29 | membership epochs (12, PH-4) | PH-4 | PH-4 |
| SI-30 | atomic claim [B]; doorbell does not bypass it (6.9.4) | PH-1 [B] | built |
| SI-31 | K-27 + provider (Track K) | PH-3 contract, PH-6 real | PH-3 |
| FX-1 | K-20 vendor-channel log (6.10); disable O-4 | PH-2, PH-4 | B-18 |
| FX-2 | hub id check [B]; V-1 minted id; V-7 declaration | PH-1 [B], PH-2, PH-3 every client | B-1 |
| FX-3 | resume only on signed DEAD + host check (PH-4) | PH-4 | PH-4 |
| FX-4 | scopes [B]; K-15 use log, owner-only (6.6.4); boundary model PH-4 | PH-2, PH-4 | B-4 |
| FX-5 | baseline [B]; ciphertext-only blobs, wrapper, writer check (6.4.5); DR-5 | PH-2 | B-11 |
| FX-6 | K-15 admin log, sweep refusal (6.6.3) | PH-2 | B-3 |
| FX-7 | "accepted/stored" wording [B]; ATTEMPTED not delivered (6.9.4) | PH-1 [B] | built |
| FX-8 | K-16 CLEAR rule (6.7.4) | PH-2 | B-14 |

## 2 Requirements to elements (R-1..R-82)

| R | Element | | R | Element |
|---|---|---|---|---|
| R-1 | chain [B]; many-to-many PH-4 (12); operator leg open (V-16) | | R-42 | FX-7 [B] |
| R-2 | hub record stages [B]; D-3 (a) | | R-43 | excluded (dropped) |
| R-3 | hooks [B]; doorbell K-19 (6.9); WAITING FOR RECIPIENT | | R-44 | deadlines [B]; DEAD dead letter (6.6.6) |
| R-4 | verbs kept; FX-4 log (6.6.4) | | R-45 | hub record first [B]; K-14 (6.5) |
| R-5 | hub path [B] | | R-46 | K-29 (12.5) |
| R-6 | sidecar [B]; status K-16 | | R-47 | adapter contract (16, 17) |
| R-7 | PH-5 circuit (12.5); GP-4 (8.2) | | R-48 | hooks + launcher [B] |
| R-8 | K-3 [B] | | R-49 | K-15 consumers (6.6.1-2), V-11 |
| R-9 | deferred PH-6 (grants first) | | R-50 | frame [B]; other rails PH-4 |
| R-10 | PH-5 (12.5) | | R-51 | identity + claim [B]; forward dedupe (7.3) |
| R-11 | send API [B]; blobs K-13 (6.4) | | R-52 | priority [B]; `--priority` B-9 |
| R-12 | PH-5 | | R-53 | reachability card (6.7.2), V-8 |
| R-13 | hub fallback [B] | | R-54 | V-7, V-1, FX-2 |
| R-14 | RECEIVED [B] | | R-55 | AEF tool; primitive T-3347 (16) |
| R-15 | STORED [B]; K-14 | | R-56 | K-11 (6.3), V-4 |
| R-16 | flag after store [B] | | R-57 | V-5 |
| R-17 | tick [B] | | R-58 | owed list B-9 |
| R-18 | priority order B-9 | | R-59 | K-10, V-6 |
| R-19 | hook route [B]; doorbell 6.9 | | R-60 | K-28 (12.1), V-12 |
| R-20 | READY + veto (6.9.2) | | R-61 | K-9a (6.1), V-1 |
| R-21 | adapter READY flag (6.9.1) | | R-62 | exact instance [B]; dead letter (6.6.6) |
| R-22 | never the screen; attached check is terminal metadata (6.9.1) | | R-63 | allow-list [B]; policy store PH-4 (V-2) |
| R-23 | SI-1 line (6.9.3) | | R-64 | grants V-2 (PH-4) |
| R-24 | nonce evidence [B] | | R-65 | PH-4 resume (FX-3) |
| R-25 | flag down when queue empty [B]; doorbell back-off (6.9.4) | | R-66 | K-21 canary, V-13 |
| R-26 | ANSWER READY [B] | | R-67 | K-22 [B], V-9, V-10, DR-9 |
| R-27 | reply path [B] | | R-67.a1 | K-23 forward-once (section 7) |
| R-28 | no-action B-9 | | R-68 | standing tests (PH-4; lease built) |
| R-29 | CLEAR (6.7.4) | | R-69 | probes and mutants per B (22.1) |
| R-30 | ladders B-9 | | R-70 | inventory; V-values recorded there (8.1) |
| R-31 | ladders default B-9 | | R-71 | PH-3 (11.2.5) |
| R-32 | circuit address [B] | | R-72 | K-27 (12.2) |
| R-33 | hub id + project id (6.1, V-7) | | R-73 | K-26 (12.3) |
| R-34 | never across projects [B] | | R-74 | PH-4 membership (12.7 re-wrap) |
| R-35 | V-3 retention | | R-75 | signatures [B]; key card (6.6.5) |
| R-36 | PH-6 digests | | R-76 | Track K, K-27 (12.2) |
| R-37 | needs-attention [B]; new entries per B | | R-77 | K-28 (12.1) |
| R-38 | deferred | | R-78 | K-26 (12.3), DR-9 |
| R-39 | packaging [B] | | R-79 | K-20 (6.10) |
| R-40 | merged into R-28 | | R-80 | K-13, K-24 (6.4), PH-4..PH-6 (12.7) |
| R-41 | deferred (planner) | | R-81 | K-18 [B]; busy bit [B] |
| | | | R-82 | K-30 (12.6), security levels (8.4) |

## 3 Threats to floor items to elements (TH-1..TH-65)

Floor items are TM 18.1's "Covers" column read backwards; the element is table 1's.

| TH | Through | Element | | TH | Through | Element |
|---|---|---|---|---|---|---|
| TH-1 | SI-3 | signatures, key card | | TH-34 | SI-24 | admin log, revoke logged (6.2.2) |
| TH-2 | SI-14 | owner-only sockets; Track A | | TH-35 | R-7, GP-4 | 8.2; PH-5 |
| TH-3 | SI-12, FX-2 | V-1, K-28 | | TH-36 | SI-19 | inline cap, blobs (6.3.1) |
| TH-4 | SI-12 | K-28 signed moved-to | | TH-37 | SI-9, V-5 | own clock, skew flag |
| TH-5 | SI-8 | IC-1: nothing is operator-class [B] | | TH-38 | SI-7, SI-8 | frame [B] |
| TH-6 | SI-1 | doorbell line, notify scope | | TH-39 | SI-7 | frame by adapter [B] |
| TH-7 | SI-2, SI-17 | nonce evidence; ATTEMPTED ≠ delivered | | TH-40 | SI-23 | kill switch [B] |
| TH-8 | SI-3, SI-12 | key card; K-28 | | TH-41 | SI-20 | lease [B], DR-9, K-26 |
| TH-9 | SI-4, SI-29 | identity triple [B]; PH-4 epochs | | TH-42 | SI-21 | V-2 grants signed |
| TH-10 | SI-4 | conflict record [B] | | TH-43 | SI-14 | admin socket separate [B] |
| TH-11 | SI-5 | K-10 | | TH-44 | SI-9 | K-29 |
| TH-12 | SI-6 | signed contiguous `up_to` | | TH-45 | SI-12 | K-28 |
| TH-13 | SI-18 | receiver decides [B] | | TH-46 | SI-10 | K-29 |
| TH-14 | SI-15, SI-22 | K-14 | | TH-47 | SI-21 | policy store; helper K-30 |
| TH-15 | SI-6, SI-22 | head compare (6.5.3) | | TH-48 | SI-8, SI-13 | K-27 digest binding |
| TH-16 | SI-1, SI-2 | SI-1 alphabet | | TH-49 | SI-31 | PR-14 typed-back fingerprint (12.1) |
| TH-17 | SI-27 | checksums [B] | | TH-50 | SI-9 | K-29 nonce window |
| TH-18 | SI-21 | key register root-owned [B]; K-26 | | TH-51 | SI-12, SI-13 | seq on cards, single-use approvals |
| TH-19 | SI-3, SI-6, SI-22, SI-24 | signatures, chains, logs | | TH-52 | SI-30 | atomic claim [B]; revocation seq |
| TH-20 | SI-13 | Tier 0 trail | | TH-53 | SI-23, SI-24 | break-glass logged (8.4.4) |
| TH-21 | SI-24 | pin override record [B] | | TH-54 | SI-26, FX-3 | signed DEAD (6.6.6) |
| TH-22 | FX-5 | ciphertext blobs (6.4); DM encryption PH-4 | | TH-55 | SI-22, SI-3 | head compare; PH-5 |
| TH-23 | R-53 (CR-11) | card readers (6.7.2) | | TH-56 | SI-11, SI-13 | key card replacement rules (6.6.5) |
| TH-24 | SI-25 | SI-25 scan; key never printed | | TH-57 | SI-28 | digest file names (6.4.4) |
| TH-25 | RR-2 | Track A (K-30); status names it | | TH-58 | SI-11 | seq rollback refused (6.2.3) |
| TH-26 | SI-9, SI-11, SI-25 | revocation; PH-5 | | TH-59 | SI-12 | K-28, V-12 |
| TH-27 | PR-11 | PH-4 outgoing scan | | TH-60 | SI-12 | roster seq (12.1) |
| TH-28 | SI-19 | K-11 | | TH-61 | SI-29 | PH-4 membership |
| TH-29 | SI-19 | hop limit (6.3.4) | | TH-62 | SI-2 | re-checks, H-11, AR-2 |
| TH-30 | SI-15, SI-19 | quotas, ceilings (V-3) | | TH-63 | SI-30 | claim [B] |
| TH-31 | SI-5 | window 256, gap wait (V-6) | | TH-64 | SI-31 | Track K |
| TH-32 | SI-19, R-77 | registration cap PN-10 (PH-3) | | TH-65 | SI-22 | daily head compare (6.5.3) |
| TH-33 | SI-20 | lease takeover rules [B]; forward-once | | | | |

## 4 Elements to what they serve (no unexplained element)

| Element | Serves |
|---|---|
| K-9a hub id + signing key | R-61, R-54, R-33; TH-3, TH-58 (enables SI-11) |
| K-9b revocation list | R-46 (RK-1), SI-11; TH-26, TH-56, TH-58 |
| K-10 window | R-59; SI-5, SI-6; TH-11, TH-12, TH-31 |
| K-11 caps | R-56; SI-19; TH-28, TH-29, TH-30, TH-36 |
| K-12 ladders | R-18, R-25, R-28, R-30, R-31, R-52, R-58 |
| K-13 blobs | R-11, R-80; SI-28, FX-5; TH-22, TH-57 |
| K-14 reconciliation | R-15, R-45; SI-15, SI-22; TH-14, TH-15, TH-65 |
| K-15 hub changes | R-49, R-44, R-75, R-4; SI-16, FX-4, FX-6; TH-1, TH-34, TH-54 |
| K-16 status | R-6, R-29, R-53, R-57, R-70; FX-8, FP 3.1.e |
| K-17 injectors | R-6.e, R-69, R-70 |
| K-18 veto | R-81; SI-2; TH-62 |
| K-19 doorbell | R-3, R-19..R-23; SI-1, SI-2; TH-6, TH-16 |
| K-20 vendor log | R-79; FX-1 |
| K-21 canary | R-66 |
| K-22 lease | R-67, R-68; SI-20; TH-33, TH-41 |
| K-23 forward-once | R-67.a1 (4), R-51, R-62; TH-33 |
| K-24 encryption keys | R-80.a (3); FX-5 (FX-5A C′) |
| K-25 declaration | R-54; FX-2; TH-3 |
| K-26 policy store | R-63, R-64, R-73, R-78; SI-20, SI-21; TH-18, TH-42 |
| K-27 Tier 0 registry | R-72, R-76; SI-8, SI-13, SI-24, SI-31; TH-20, TH-48 |
| K-28 directory | R-60, R-77; SI-12; TH-4, TH-8, TH-45, TH-59, TH-60 |
| K-29 circuit | R-7, R-10, R-12, R-46; SI-9, SI-10; TH-44, TH-46, TH-50 |
| K-30 account helper | R-82; TH-2, TH-25 |
| K-31 custody | R-80.a (5), (7) |
| Security levels (8.4) | R-82 and R-76 strength selection (FP 10.1.8.g operator direction) |
| V-3 retention ceilings | R-35; TH-30 |
| GP-4 behaviour (8.2) | R-7, R-44, R-45; TH-35 |

4.1 No element without a source was found. Two elements reveal a requirement gap and are raised as change requests
rather than gold-plating: the per-agent encryption key in PH-2 (XR-5) and the security-level classes (XR-6).
