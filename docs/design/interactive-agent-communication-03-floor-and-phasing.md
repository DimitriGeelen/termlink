# Interactive agent communication: step 3, security floor and phasing

**Task:** T-3352 · **Arc:** arc-011 · **Chain:** arc-011-design, step 3 · **Role:** security floor and phasing (MVP specialist)
**Status:** DRAFT v0.1 for the operator. Nothing in this document is approved. The floor, the phasing and every risk acceptance are the operator's to confirm (card, section 5). Everything tagged **[P]** is a proposal; the operator decides. The open questions are in section 13 and are meant to be taken one at a time.
**Review link:** none. This project's Watchtower has no inline design-review page (profile P2.1). The operator reads this file in the repository.

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-11 | First complete draft, batch mode. Security floor (31 invariants plus 8 extra items), prioritisation of the 82 requirements, the basic version, seven phases with exit criteria and rollback, 30 hypotheses, the GP-12 numbers, CAND-3 staging, the four items the operator's rulings left to this step (accounts, risk classes, policy format, interim controls) and the outage-log reconciliation rules, 8 risk-acceptance requests, 16 operator questions, drawings D-1 and D-2. | T-3352 |

## 1 Inputs of record

1.1 The two inputs the hand-over named. The hashes are the full sha256 values I computed myself at the start of this step (`sha256sum`); they match the values in the hand-over. Had either differed I would have stopped and reported.

| Short | Path | Version | sha256 |
|---|---|---|---|
| REQ | `docs/design/interactive-agent-communication-01-requirements.md` | v0.5.2, approved by the operator 2026-10-11 | `d2d839fb743eb4c542f337f2b3d244e3df4a47ac40807ebbc5f7dda5ead0a3d4` |
| TM | `docs/design/interactive-agent-communication-02-threat-model.md` | v0.3.7, residual risks accepted by the operator 2026-10-11 | `88961693384966502f4ca4eb515ed4b0590a29c3d61ac0c619e17ea4562693f3` |

1.2 The operator's rulings I treated as binding and did not reopen. A ruling is cited by its id (OQ-n, CR-n, RR-n, D4 to D7, OD-n, CAND-n, GP-n).

| Short | Where | sha256 of the file read |
|---|---|---|
| RULINGS | TM section 22.18 (the rulings register) | part of TM |
| T3351 | `.tasks/completed/T-3351-arc-011-role-chain-step-2-threat-modeler.md`, section "Decisions" | `d17ce3da1a7798a55a0867bfd3e4ac1d71cb52a7a4491479d0e23cba377a93f5` |
| T3388 | `.tasks/completed/T-3388-arc-011-step-1-reopen-requirements-v05-f.md`, section "Decisions" | `571d472c2445d8a5f1c7b25dadfa04fa0c0f94027bc2dfb202f5603b0b97140c` |
| HANDOFF | REQ 13.1 (row for step 3) and 13.5.1 | part of REQ |

1.3 Also read, in the order the hand-over gave. The hashes are of the bytes as read.

| Short | Path | sha256 |
|---|---|---|
| RULES | `CLAUDE.md`, lines 1 to 2820, the project sections above `## Core Principle` | `54c2ceb4dc48bcf79c15d40f3668594faa06393f13626a86608f4447e57b451c` |
| COMMON | `docs/design/roles/cards/_common.md` | `cca38806358d7a6c09e64289164977a825f2cf57e914f672076ff77a2bdb5563` |
| CARD | `docs/design/roles/cards/mvp-specialist.md` | `04fa87539950fd1ce74d63ecdacf36bb4d90d53fb3b60d79d23d83d8273be66e` |
| ADAPTER | `docs/design/roles/adapters/aef.md` | `d840a77649888cdbfd37d1166c15eecd6cd3c8399d497fb6b1b2c52455c6c4a4` |
| PROFILE | `docs/design/roles/profiles/termlink.md` | `b6bc160242d96ca9a8fc167c6845e292529c162fa23faa2945d4742b676e7b21` |
| README | `docs/design/roles/README.md` | `3881d5302313a0d0ca4c9ef1b4688dae101b6d693df823be962e9b70946368e0` |
| CHAIN | `docs/design/interactive-agent-communication-role-chain.yaml` | `91c9c1ab63988e90ab906ab6103825f48a91a72c4220afaf5ee47f004690464d` |
| SCHEMA | `docs/design/roles/schemas/role-handback-1.schema.json` | `288a4e312f00bac978a4b2a87e57ee8a05cb14f916d77c397e601e03c703c148` |
| LOG | `docs/design/interactive-agent-communication/interview-step-01.md` (the GP-12 ruling, gap review item 3) | `aad8308e1924df293270d628b224dafc4814f675a4a61485dd2c1797a7114edd` |
| GP12 | `docs/reports/T-3344-gp12-review/` (`brief.md`, `comparison.md`; the five real failures) | not hashed |

1.4 How this step was run.
1.4.a Mode: batch. No human was present (`_common.md` 5.2). Every question I could not answer from a ruling is in section 13 as an operator question with options and a recommendation.
1.4.b Where the AEF adapter names ring20 paths or commands this project lacks (`role-chain.py`, `fw reviewer judge`, `design-render-check.py`, the `docs/designs/agent-authorization-broker/` paths), the profile governs (P2.1, P4.5). The drawings were checked with `mmdc` and the local Chromium, the method the step-2 document used (section 16).
1.4.c I did not change REQ, TM, the role-chain file or any task file. Where an earlier step needs a change, it is a change request in section 12 (CX-n).
1.4.d Facts about what runs today come from REQ, TM, `CLAUDE.md` and the profile, tagged **[H]**. Three read-only host facts I did read myself on 2026-10-11, used only for the price of per-agent accounts (10.1): `ps` shows 86 process lines that mention `claude` run as `root` and 16 as the operator's own login; `stat` shows `/opt/termlink`, `.git`, `.tasks` and `.context` are `root:root` mode 775 and `git config core.sharedRepository` is unset; `useradd`, `sudo` and `setfacl` are installed. The 86 lines are processes, not agents (one agent has several), so they do not replace the "19 Claude agents" of R-82.a (1).
1.4.e Nothing was sent outside the project. The policy-record format that 055 is waiting for (10.3) is written here; sending it is the orchestrator's and the operator's act, not mine (`_common.md` 4.1).
1.4.f I did not use an in-process sub-agent or a vendor peer channel (R-79).

## 2 How to read this document

2.1 Identifiers. Each kind is numbered once in this document. Ids of earlier steps are unchanged and are cited, never redefined.
2.1.a New here: `FX-n` an extra floor item (3.4). `PH-n` a phase (section 6). `BV` the basic version (section 5). `FP-n` a proposal of this step (for example the numbers of section 7). `RA-n` a risk-acceptance request (section 11). `OP-n` an operator question (section 13). `CX-n` a change request to an earlier step (section 12). `RC-n` a reconciliation rule (10.5). `IC-n` an interim control (10.4). `C-n` in 10.2 a class of Tier 0 event. `H-14` and up are new hypotheses; `H-1` to `H-13` are TM's and keep their numbers. `D-1` and `D-2` are this document's drawings (each document numbers its own).
2.1.b From earlier steps and used as they are defined there: `R-n`, `OD-n`, `CAND-n`, `GP-n`, `ADV-n` (REQ); `SI-n`, `TH-n`, `PR-n`, `PN-n`, `RR-n`, `CR-n`, `OQ-n`, `BP-n`, `RK-n`, `TB-n`, `F-1` to `F-7` (TM).
2.2 Authority tags, as in REQ 6.1.h. **[R]** operator ruling. **[R~]** a ruled principle with the orchestrator's detail, which may be changed with evidence. **[H]** historical fact from the record. **[A]** my assumption. **[P]** my proposal. A statement without a tag restates a source and cites it.
2.3 Plain meaning of the words this step adds.
2.3.a **Security floor.** The list of controls that must hold wherever their subject exists. It is not a priority class and it is not traded against other work. Section 3.
2.3.b **Subject.** The thing a floor item protects: the doorbell, a circuit, a grant. A floor item "enters" in the first phase where its subject exists and "completes" in the phase where every part of it holds.
2.3.c **Ship gate (SG).** A feature may be switched on for a real target only when every floor item that belongs to it passes its probe, and the probe's negative control has been seen to fail when the control is removed (R-69, kill-checked). This is what stops the phasing from deferring the floor.
2.3.d **No knob before its lock (NK).** A phase ships no way to configure an authority (a grant, an allow-list, an eligibility, a membership change) before the control that protects that configuration exists. Before then the only authority in the system is a fixed built-in default that nothing can widen. This keeps the first releases small and honest.
2.3.e **Strength flags A and K.** Some floor items cannot fully hold against a process on the same host until two things exist: **A**, one operating-system account per agent (R-82); **K**, the operator key with its confirmation provider (R-76). A floor item that needs a flag holds in a weaker form until the flag exists, and the status call says which flags are absent (3.1.e). The weaker form is what TM already accepted as RR-2 (host root, always), RR-16 and RR-18; this step only says for how long (RA-1, RA-4).
2.4 Reading rule. Purpose and intent over form (the operator's rule of 2026-10-08, REQ 6.17.2): where a wording and its purpose disagree, the purpose decides and the disagreement is reported to the operator, not settled silently.
2.5 Style. Everything is numbered. Choices are lettered A to D with a recommendation. RFC 2119 words mark what is normative.

## 3 The security floor (card 3.1)

3.1 What the floor is and the rules that bind it.
3.1.a **Definition.** The floor is every security invariant SI-1 to SI-31 of TM section 18, plus every requirement that prevents an unauthorised irreversible action and is not already an invariant (the extras FX-1 to FX-8 of 3.4). It sits outside Must, Should and Could (card 3.1).
3.1.b **Ship gate (SG, 2.3.c).** Binding on every phase. A phase's exit criteria always include the green probes of the floor items that enter or complete in it.
3.1.c **No knob before its lock (NK, 2.3.d).** Binding on every phase.
3.1.d **Rollback keeps the floor.** A rollback removes a subject together with its floor items. It never leaves a subject running without its floor item, and it never turns a control off to make a rollback easier. Where a rollback would have to remove a protection (for example decrypting content that was encrypted), the phase says so and the rollback stops short of it (6.5).
3.1.e **Status honesty.** The status call of R-6.e MUST name each strength flag that is absent ("no per-agent accounts", "no operator key", "no confirmation provider") and MUST NOT claim a defence against an adversary the present strength does not stop. This restates R-76.e and R-82.e as one floor rule; it is cheap and it is the control against ADV-3 applied to the floor itself.
3.1.f **Deferral rule.** If a floor item cannot hold in full in a phase where its subject exists, the gap is a risk-acceptance request (section 11). Nothing in the floor is listed as Must, Should or Could to reduce it.
3.2 What counts as an unauthorised irreversible action. I used six classes to decide the extras. A control belongs to the floor if its failure allows one of them.
3.2.a **IR-a** An action on a host or in an agent that cannot be undone by rolling the system back: a keystroke typed into a session, a command run, a file written, a push, an agent started or resumed.
3.2.b **IR-b** Loss of an accepted message, or a false record that hides a loss. After STORED the sender is released, so the loss cannot be recovered by the sender.
3.2.c **IR-c** Disclosure of content or of a secret. It cannot be un-read.
3.2.d **IR-d** A grant of authority that outlives the mistake: admission of a hub, a key replacement, a re-home, a re-pin, a grant, a role takeover, an account. It can be revoked, but what happened before the revocation stays.
3.2.e **IR-e** Destroying the evidence that proves what happened (a sweep or trim of the record before delivery is proven).
3.2.f **IR-f** One instruction acted on twice (a double hand-over, a replay).
3.3 The 31 invariants as floor items. Statements are shortened; the full wording and the probe of each are in TM 18.1 and 18.2 and are binding. Columns: **Parents** the requirements the invariant restates or depends on. **Enters** the first phase in which the subject exists and the item must hold. **Complete** the phase in which all of it holds. **Needs** a strength flag (2.3.e) for the part that stops a process on the same host or the operator's impersonator. **Standing** E existing in a ruled requirement; R proposal now ruled by the named accepted change request; P proposal that no change request carried, so the operator confirms it by confirming the floor (OP-6).

| Id | Statement (short) | Parents | Enters | Complete | Needs | Standing |
|---|---|---|---|---|---|---|
| SI-1 | The typed line is fixed text, a decimal count and 32-hex ids; no peer byte reaches it; inert in a shell | R-23 | PH-2 | PH-2 | none | R CR-5 |
| SI-2 | Nothing typed unless READY and the foreground process is the harness, both re-checked just before; activity veto; never from the screen | R-19.r, R-20, R-21, R-22, R-81 | PH-2 | PH-2 | none | E, R CR-5 |
| SI-3 | Every message signed by the sending agent's own key, verified at receiver and hub | R-75, R-34.e | PH-1 | PH-1 | A for same-host attackers | R CR-2 |
| SI-4 | Identity is (sender, conversation, message id) bound to a digest; same identity with another digest is a conflict | R-51 | PH-1 | PH-1 | none | E, R CR-3 |
| SI-5 | Same sender, conversation and number with another digest is equivocation; numbers beyond the window refused | R-59 | PH-2 | PH-2 (circuit part PH-5) | none | E, R CR-4 |
| SI-6 | Receipts and stage records signed by the recorder, forward-only, `up_to` only the highest contiguous stored number | R-59, R-45 | PH-1 (stage records) | PH-2 (`up_to`) | none | E, R CR-4 |
| SI-7 | Peer content only inside a frame built by the receiving side with a random boundary | R-50 | PH-1 | PH-4 (every other rail) | none | E, R CR-6 |
| SI-8 | No peer message grants, approves or raises trust; a CR-18 route decides, never message content | R-50, R-72 | PH-1 | PH-3 | none | E, R CR-6, CR-18 |
| SI-9 | Circuit credential sender-constrained, instance-bound, one conversation, counted from establishment, never reset | R-46 | PH-5 | PH-5 | none | R CR-7, OQ-3 |
| SI-10 | The hub mints a credential only after checking key, instances, liveness and cap; it does not apply the interrupt allow-list | R-46, R-62 | PH-5 | PH-5 | none | E, R CR-7 |
| SI-11 | Each revocation scope takes effect within its stated bound; reachable and partitioned bounds never conflated; failed pull is not an empty list | R-46, R-57 | PH-2 (RK-1, RK-6) | PH-5 (RK-2, RK-3, RK-5) | none | R CR-7, OQ-3 |
| SI-12 | A card is accepted only if signed by a roster hub for a project it homes, within its time-to-live, with a rising sequence number | R-60, R-77 | PH-3 | PH-3 | K for enrolment codes | R CR-8, OQ-8 |
| SI-13 | An approval is bound to the digest of the full action including executor and target, single-use by atomic consumption, short-lived | R-72, R-76 | PH-3 | PH-3 | K for the full form | R CR-14, CR-18 |
| SI-14 | The send and status API is an owner-only local socket with a caller check; admin is a separate credential | PR-8 | PH-1 | PH-2 (three scopes) | A for "owner-only" to mean anything | P |
| SI-15 | STORED only after record and directory entry are on stable storage with a checksum; reconciled with the hub record on start | R-15, R-16.a | PH-1 | PH-2 (reconciliation, host-crash test) | A for F-7 prevention | E, R OQ-5 |
| SI-16 | Exactly one sidecar holds an agent's store lock; exactly one identity consumes an inbox | R-49 | PH-1 | PH-2 (consumer visibility on the hub) | none | E |
| SI-17 | HANDED_OVER evidence is the delivery nonce inside a harness-written record type; never an assistant record or a tool result | R-24 | PH-1 | PH-1 | A narrows RR-7 | R CR-17 |
| SI-18 | Priority, urgency and trust class are decided at the receiver from the verified sender, signed fields and allow-list; a downgrade is recorded, never a drop | R-52, R-63 | PH-2 | PH-2 | none | E, R CR-2 |
| SI-19 | Open-message cap, size limits, per-sender quota and aggregate ceiling enforced at the receiver; reserved shares for receipts and own project | R-56 | PH-2 | PH-2 | none | E (sender cap), P (receiver side, PR-28) |
| SI-20 | Role eligibility, priority and pin come from operator-signed configuration at the home hub | R-67, R-78 | PH-2 (pin only) | PH-4 | A and K | R CR-13, CR-14 |
| SI-21 | A grant, allow-list or roster entry takes effect only with a valid operator signature or a record the hub or an operator-owned store holds and the sidecar verifies; epoch checked | R-63, R-64, R-78 | PH-2 (fixed default only) | PH-4 | A and K | R CR-13, CR-14 |
| SI-22 | Stage records and the local log hash-chained per conversation; receipts carry the chain head; truncation inside the anchored prefix detected | R-45 | PH-1 (chain) | PH-2 (heads, daily compare) | A against same-user rewrite | R CR-1 |
| SI-23 | The operator can switch off mid-turn delivery for one agent and for the fleet, within one tick, audited | none ruled | PH-2 | PH-2 | none | P |
| SI-24 | Every break-glass action is operator-initiated, logged with who, when and why, and shown in needs-attention | R-67.a | PH-0 (re-pin record) | PH-4 | none | E (override of main), P (the rest) |
| SI-25 | No secret appears in logs, status output, the doorbell, a frame, a stage record or the hub record | `_common.md` 4.2 | PH-0 | PH-0 and every later phase | none | E in part |
| SI-26 | A message to an exact instance reaches only that instance and fails loudly if it has ended | R-34, R-62 | PH-1 | PH-4 (hand-over fence) | none | E |
| SI-27 | The installed sidecar and adapters are verified against release checksums at install and in every status call; an incompatible protocol is refused with a reason | R-57, R-69 | PH-2 | PH-2 | none | E in part, P (checksums) |
| SI-28 | A blob is stored only under its own digest in a private directory; no peer name or path is used as a file name | R-11 | PH-2 | PH-2 | none | E in part, P (path rule) |
| SI-29 | Membership changes only by a signed, countersigned, recorded event; a membership epoch on every turn; fenced copies refused | R-74, R-62 | PH-4 | PH-4 | none | R CR-15, OQ-12, OQ-17 |
| SI-30 | At most one delivery attempt open per message: one atomic claim before content leaves the store; redeliver only on proof of absence | R-51.a, R-24.e | PH-1 | PH-1 | none | R CR-17 |
| SI-31 | A high-impact approval takes effect only after an active out-of-band confirmation bound to its digest; never on silence; operator key revocable | R-76 | PH-3 | PH-3 | K | R CR-14, OQ-14 |

3.3.1 Reading the table. 3.3.1.a Two items enter before any message path exists: SI-25 (a scan for secrets in the artifacts of every test run, from the first line of new code) and SI-24's earliest form (the record of every re-pin and `tofu clear`, T-3384, ruled in OQ-8 D″). 3.3.1.b SI-20 and SI-21 enter in PH-2 only in the form NK allows: the built-in default "own project only" and an operator-pinned main, with no way to configure either. They complete in PH-4 when the configuration surface and its lock arrive together. 3.3.1.c SI-8 enters in PH-1 as "message content approves nothing" and completes in PH-3, when the first Tier 0 route exists and the sentence "a route decides" has something to point at.
3.4 The extra floor items. Each prevents an unauthorised irreversible action (3.2) and has no invariant of its own. FX-7 and FX-8 go beyond the strict criterion and are marked.

| Id | Statement | Source | Class | Enters | Complete | Needs | Probe |
|---|---|---|---|---|---|---|---|
| FX-1 | All delegation goes through TermLink. Vendor peer channels and vendor in-process sub-agents are disabled by the harness adapter where the harness allows, and every use is logged and shown in needs-attention where it cannot be disabled | R-79, R-47.a, BP-14 | IR-a | PH-2 (log and show) | PH-4 (disable, after T-3386, see RA-5) | none | Use the vendor channel on a harness that cannot disable it and require a log row and a needs-attention entry; on a harness that can, require the channel refused |
| FX-2 | The project's mail hub is declared by canonical id and address; the client verifies `hub_id` and refuses on mismatch; no local fallback | R-54, BP-8 | IR-b, IR-c | PH-2 | PH-3 (every client) | none | Answer the declared address with another hub's id and require refusal and a status line; rotate the certificate and require no change |
| FX-3 | A resume of an exact instance happens only when the home hub's DEAD is signed and sequenced and the host-local supervisor has verified that no process holds the session | R-65.a, TH-54 | IR-a | PH-4 | PH-4 | none | Hub says DEAD while a process holds the session: require no resume and an operator flag; unsigned DEAD: refused |
| FX-4 | The founding verbs (inject, pty inject, exec, remote exec) have their own scope; every use is logged with the caller; the owner session may refuse; cross-project use is a Tier 0 event | R-4.o, PR-29, BP-1, BP-2 | IR-a | PH-2 (scope and log) | PH-4 (boundary model, see RA-2) | none | Call each verb with an Observe-scope token and require refusal; every allowed call leaves a log row; a cross-project call without a route decision is refused |
| FX-5 | Disclosure does not grow. No feature writes content to a hub in a form a token holder or the hub operator can read beyond what today's rails already expose; a circuit turn copied to the hub is encrypted per message to its authorised readers | R-80, TH-22 | IR-c | PH-2 (no growth) | PH-5 (circuit copies), PH-6 (project mail, custody) | A for custody; K for break-glass | Hold a topic read token and read an encrypted turn: envelope only. Count plaintext content fields on the hub per phase: never above the PH-0 baseline |
| FX-6 | Evidence is kept. Every hub admin RPC is logged; a sweep or trim of a conversation topic below its open messages is refused | BP-7, R-35.o (3) | IR-e | PH-2 | PH-2 | none | Sweep a topic with an open message and require refusal and a log row |
| FX-7 **[P]** | Nothing reports a message "delivered" without a recorded HANDED_OVER; until then every tool says "accepted" or "stored" | R-42, R-24 | beyond IR (false assurance leads to action on a false belief; ADV-3, 2026-10-03) | PH-1 | PH-1 | none | A post with no HANDED_OVER is reported as accepted or stored by the hub and by every sender tool |
| FX-8 **[P]** | A deaf agent is never CLEAR: CLEAR needs recent surfacing progress, never a fresh heartbeat alone | R-29.e, R-53 | beyond IR (the 32,205-refusal sidecar; ADV-3) | PH-2 | PH-2 | none | A sidecar with a seconds-old heartbeat that refuses every confirmation reads NOT CLEAR |

3.4.1 Why FX-7 and FX-8 are in. They do not meet the strict test of the card. I put them in because they close the one failure class the record proves, the false green: three real failures fired nothing (R-66.c), and both cost nothing in time because they ship in PH-1 and PH-2 anyway. The operator can move either out (RA-8).

<!-- CONTINUE -->
