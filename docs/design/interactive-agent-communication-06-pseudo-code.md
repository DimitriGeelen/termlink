# Interactive agent communication: step 6, pseudo-code

**Task:** T-3355 · **Arc:** arc-011 · **Chain:** arc-011-design, step 6 · **Role:** pseudo-coder
**Status:** DRAFT v0.1, batch mode (no human present, `_common.md` 5.2). This step does not decide (`decides: false`);
it iterates with step 5 (evidence specialist). Nothing here is approved. Values that are not ruled numbers are tagged
**[P]**; ruled numbers carry their source; **[B]** marks logic written as built (read from the code at `24d15fba3`).
**Standing rules applied throughout (CLAUDE.md):** nothing working is switched off as a side effect (**NID**);
thresholds measure reliability, never usage (**RNU**).
**Review link:** none. This project's Watchtower has no inline design-review page (profile P2.1); the operator and
the reviewers read the files in the repository.
**Sub-documents** (split per phase, card 5.1 and the brief's ~1500-line rule), each reviewable on its own:
1. this file: conventions, the answers to step 5's questions, the PH-2 delivery path (AL-1 to AL-16, the operator's
   priority: activity meter, doorbell D+ and watchdog, lease tightening, waiting-mail signal, forward-once), the
   FAILS CLOSED WHEN register for all three files, residual risks and change requests;
2. `docs/design/interactive-agent-communication-06a-pseudo-code-ph2-hardening.md`: the rest of PH-2 (AL-17 to AL-40:
   hub id and signing key, canonical signed state with test vectors, revocation, receive admission and caps, blobs and
   keys, status and CLEAR, hub changes, reconciliation, gap wait and owed list, fault injectors, vendor channels,
   wave gates);
3. `docs/design/interactive-agent-communication-06b-pseudo-code-later-phases.md`: PH-3 to PH-6 (AL-41 to AL-53: the
   approval broker scope of the card, grants, policy records, roster and cards, circuit, membership, break-glass,
   custody, account helper, security levels).

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-11-11 | First complete draft, batch mode. 53 algorithms AL-1..AL-53 in the card's form 4.a..4.h across three files; answers to step 5's questions 14.3 (a)..(i); FAILS CLOSED WHEN register FCW-1..FCW-n with the AFC each refines and its spec or a change request; drawings D-1 (state machines) and D-2 (interaction sequences); canonicalisation test vectors; residual risks QR-n; change requests ZR-n. | T-3355 |

## 1 Inputs of record

1.1 Full sha256 of the bytes read on 2026-11-11 (`sha256sum`), at commit `24d15fba3`.

| Short | Path | Version | sha256 |
|---|---|---|---|
| ARC4 | `docs/design/interactive-agent-communication-04-architecture.md` | v0.3.1, approved 2026-11-08 | `29c6d1f7de9f18898cab105651f17977ebf97da30f32d16501819bad217dd69c` |
| TRACE | `docs/design/interactive-agent-communication-04a-traceability.md` | v0.3 (part of ARC4) | `c516a290997e0c75957c0be95af4d611e58155cd818d2fbae961c0f4ad6d4667` |
| EV5 | `docs/design/interactive-agent-communication-05-evidence.md` | v0.1; review green V-20261109-454c9bbb | `29be80ae3ad8d32b71ef53caa00f9fdc6d03426c901215eaed85c019f889526e` |
| EV5A | `docs/design/interactive-agent-communication-05a-coverage.md` | v0.1 (part of EV5) | `16a4aa77b23cad191cf1b9153e60016a266ffc763c54674cb2d6e1f6341824fc` |
| HB5 | `docs/design/interactive-agent-communication/role-handback-step-05.json` | step-5 hand-back (open questions, item 11 of T-3354) | `03fbef422d071ba1b322216cae46515b2472901eff68fbc03c967b06185315b6` |
| REQ | `docs/design/interactive-agent-communication-01-requirements.md` | v0.5.3 | `f6ce42a2984bd18e049f1f4dda05e6eaab2cc3508396fe373060c6fc527193c9` |
| TM | `docs/design/interactive-agent-communication-02-threat-model.md` | v0.3.7 (SI-1..SI-31 read in full, section 18) | `88961693384966502f4ca4eb515ed4b0590a29c3d61ac0c619e17ea4562693f3` |
| FP | `docs/design/interactive-agent-communication-03-floor-and-phasing.md` | v0.3 | `e4de2e3e102ee59f6434db584e099e5bb180c6b33dfd823ae217e6ebb4e5a8c4` |

1.2 Built and live facts, written as built where the architecture keeps them (brief).

| Short | Path | sha256 |
|---|---|---|
| ARCH2 | `docs/design/arc-011-ph2-architecture.md` | `b8a941e5563b72707bb3abbe89677ba42ab5560dc1882721b59f59390d84d589` |
| LEASE | `docs/operations/main-lease.md` | `c95c12e76e0e3b8166badf4919e262916210c89b427ac2d4be996cda65c51e9a` |
| CODE | `crates/termlink-session/src/main_lease.rs` (`plan_claim`, `claim_inner`, `Timing::from_env`), `crates/termlink-session/src/handler.rs` (`doorbell_line`, `handle_command_notify`), `crates/termlink-sidecar/src/{lease,activity,deliver,hook,api,beat,liveness,estate}.rs`, `crates/termlink-session/src/offline_queue.rs` (`mint_client_msg_id`) at `24d15fba3` | not hashed; cited by path |

1.3 Role inputs, in the order given (README 3.3).

| Short | Path | sha256 |
|---|---|---|
| RULES | `CLAUDE.md` (project sections above `## Core Principle`) | `a9b38fd1aeed0145289c6a8c89c491c2ae077d06e1847110209827f33e172aa3` |
| COMMON | `docs/design/roles/cards/_common.md` | `cca38806358d7a6c09e64289164977a825f2cf57e914f672076ff77a2bdb5563` |
| CARD | `docs/design/roles/cards/pseudo-coder.md` | `4801aa19de42c170e347ac1d5517b0ee6106cd94eb76a2cc7d856e601ab65f76` |
| ADAPTER | `docs/design/roles/adapters/aef.md` | `d840a77649888cdbfd37d1166c15eecd6cd3c8399d497fb6b1b2c52455c6c4a4` |
| PROFILE | `docs/design/roles/profiles/termlink.md` | `b6bc160242d96ca9a8fc167c6845e292529c162fa23faa2945d4742b676e7b21` |
| README | `docs/design/roles/README.md` | `3881d5302313a0d0ca4c9ef1b4688dae101b6d693df823be962e9b70946368e0` |
| CHAIN | `docs/design/interactive-agent-communication-role-chain.yaml` | `6ed4d708b8364428a5f9f2996519e8956be02b31e5a10817b55df29be99b8752` |
| SCHEMA | `docs/design/roles/schemas/role-handback-1.schema.json` | `288a4e312f00bac978a4b2a87e57ee8a05cb14f916d77c397e601e03c703c148` |

1.4 How this step was run.
1.4.a Batch mode. Every question I could not answer from an input is a change request (section 9) or a residual
risk (section 8), never a silent assumption.
1.4.b Where the adapter names ring20 paths this project lacks (`role-chain.py`, `design-render-check.py`,
`role-handback-check.py`), the profile governs (P2.1, P4.5). Drawings were checked with `mmdc` and the local
Chromium, the method of steps 2 to 5 (section 11).
1.4.c I changed no earlier output and no code. Where a built fact must change for an algorithm to hold, it is named
as a build delta inside the algorithm (marked **Δ**) and, where it touches a ruled text, as a change request.
1.4.d The test vectors of 06a section 2.4 and 06b were computed with `.context/working/t3355/vectors.py` (Python,
PyNaCl) using the public RFC 8032 test key 1, never a key of this estate (SI-25).

## 2 How to read this document

2.1 Identifiers new here: `AL-n` algorithms, `FCW-n` FAILS CLOSED WHEN conditions (section 6 is the register),
`OBS-n` observation points the evidence runner reads (answer 4.9), `ZR-n` change requests to earlier steps,
`QR-n` residual risks of this step, `D-1.n` and `D-2.n` the parts of the two required drawings. Ids of earlier steps
are cited as defined there (R-n, SI-n, TH-n, RR-n, FX-n, PN-n, H-n, K-n, B-n, V-n, DR-n, AR-n, XR-n, E-n, AFC-n,
EP-n, YR-n). The PH-1 architecture's own `FC-n` are cited as **ARCH FC-n** (as step 5 does).
2.2 Each algorithm has the card's form: **a** SERVES; **b** PROBLEM and APPROACH; **c** ALGORITHM (language-neutral
steps in a `text` block, a state table where it is a state machine); **d** INVARIANTS; **e** FAILS CLOSED WHEN (each
an FCW-n); **f** EDGE CASES (forged, replayed, expired, altered input; clock and its source; canonical forms;
concurrency; a **crash table** for every state-changing step); **g** EXECUTION SEMANTICS; **h** CRYPTOGRAPHY. A line
**OBS** after **h** lists what the evidence runner reads.
2.3 RFC 2119 words are normative for the build items that implement an algorithm. The language of the build is the
build's choice (card 1); names in the pseudo-code (`store.chain_append`, `hub.stage`) are roles, not function names
in the code.
2.4 Reading order for the build, the operator's priority (brief): AL-1 to AL-9 (B-27, then B-16), AL-11 to AL-13
(B-25), AL-14 and AL-15 (B-20), then 06a in its order, then 06b.

## 3 Conventions shared by every algorithm

3.1 **Clocks (V-5, R-57; answers 4.3).**
3.1.a `mono()` is the observer's monotonic clock in milliseconds since boot, with `boot_id` (Linux
`/proc/sys/kernel/random/boot_id`; elsewhere a per-boot value the sidecar records at start). Every deadline, window,
age and back-off below is computed on `mono()` of the process that decides. A value stamped on another boot reads
**absent**.
3.1.b `wall()` is the wall clock, used only for display, records meant for other hosts, and the lease (whose built
logic uses the claimer's wall clock, AL-11 3.f). A wall value is never subtracted from a `mono()` value.
3.1.c `TICK` = 30 000 ms **[B]** (`deadline::TICK_MS`, OQ-d A). "Within 1 tick" means `≤ TICK` on `mono()`. The
tick thread evaluates every per-tick routine once per tick; the 500 ms loop keeps pulling **[B]**.
3.1.d `sidecar_epoch` is a random 64-bit value minted at every sidecar process start; records that only this process
may trust (the READY flag for the doorbell, AL-1) carry it.

3.2 **Primitives** (each built unless marked Δ).
3.2.a `store.lock` the per-agent store lock (SI-16) **[B]**; `store.chain_append(line)` appends a hash-chained line
under the lock and fsyncs it **[B]**; `atomic_write(path, bytes)` = write temp, fsync, rename, fsync directory **[B]**.
3.2.b `attention(kind, why, coalesce_key)` writes a needs-attention entry; with a `coalesce_key` at most one entry
per key per coalescing period (stated where used) **[B]** (kinds and R-37T A′ rules as built).
3.2.c `admin_row(kind, fields)` appends to the sidecar's admin chain **[B]**.
3.2.d `hub.stage(msg, stage, extra)` writes a signed stage record to the hub record; idempotent by (message identity,
stage, attempt) **[B]**. `notice(to, kind, fields)` posts a signed notice **[B]** (`wire::NOTICES`).
3.2.e `session_rpc(session, method, params, timeout)` calls the TermLink session with the agent's `notify`-scope token
**[B]**; a timeout or transport error returns `Err`.
3.2.f `delivery_on()` reads the per-agent kill switch from the store at the moment of the call **[B]** (`admin
delivery on|off`, SI-23).
3.2.g `lease_view()` is the sidecar's last lease read with its read time (AL-12) **[B]**.

3.3 **Strict JSON** (every parser that reads peer or adapter bytes). UTF-8 only; duplicate keys refused; depth ≤ 8;
total size bounded per format (stated where used); numbers only integers in [−(2^53−1), 2^53−1] where a schema has
numbers (no fractions, no exponents, no `-0`); unknown top-level fields ignored and counted, never used. Refusal is
a value (`Err(reason)`), never a crash.

3.4 **Test seams** are honoured only through AL-16 (answer 4.8). No algorithm below reads an environment variable
directly.

## 4 Answers to step 5's questions (EV5 14.3 a..h, and HB5 item 11)

4.1 **(a) The FCW-n list.** Section 6 is the register: every FAILS CLOSED WHEN condition of the three files with its
id, the routine it sits in, the AFC-n it refines (or "new") and the E-n that covers it, or the change request to
step 5 (ZR-1) where no spec exists yet. Totals are in 6.1.

4.2 **(b) The back-off schedule as a function (AL-7).** For one session, an episode starts at the first normal ring
after the last progress (a HANDED_OVER to that session, or the pending set becoming empty). Ring `k` of the episode
(`k = 0, 1, 2, ...`) MAY happen no earlier than tick `T0 + O(k)`, where `T0` is the tick of ring 0 and
`O(0) = 0`, `O(1) = 4`, `O(2) = 12`, `O(k) = 12 + 20·(k − 2)` for `k ≥ 3`; that is, intervals `I(1) = 4`, `I(2) = 8`,
`I(k ≥ 3) = 20` ticks (20 ticks = 10 min). A ring happens at the first tick `≥ T0 + O(k)` at which AL-5's gate and
AL-6's re-checks pass; a held tick does not move the schedule. At most one ring per session per tick. While a ring has
had no submit (AL-7 state `UNSUBMITTED`) no normal ring happens at all; only the watchdog may type (AL-8). The R-44.e
"not handed over" deadline (2 ticks after READY **[B]**) produces the `OVERDUE-HANDOVER` notice and the sender's STUCK
independently of this schedule; rings continue after STUCK because the message stays owed. Expected table for E-5
(60 simulated minutes, session READY and idle, nothing ever submitted): one normal ring at tick 0, then none (the
line is unsubmitted); with a fake adapter that submits every line, lets the hook pull, and then reports the
evidence-complete point without the nonce (proof of absence, so the message is QUEUED again, AL-9): rings at ticks 0,
4, 12, 32, 52, 72, 92, 112 (8 rings in 120 ticks). Without the complete point the first pull leaves the message
UNCERTAIN for the operator (SI-30) and no further ring happens.

4.3 **(c) On which clock READY age is measured.** On the **receiver's** `mono()`, stamped when the receiver accepts the
`ready` report (`rx_mono`, ARC4 17.2: the receiver uses its own receive time for deadlines). The adapter's `at_ms`
orders reports of one session (a report older than the one applied is dropped; a tie between BUSY and READY keeps
BUSY) and is displayed; it is never subtracted from `mono()`. The words "READY re-read and its age ≤ 1 tick" of ARC4
6.9.2 are read as: the re-check re-reads the receiver's current flag record, which MUST still be READY, MUST carry the
same `ready_seq` as the snapshot the gate used (no report of any kind arrived in between), MUST carry the current
`sidecar_epoch`, and the re-read MUST be taken within 1 tick before the keystrokes. The flag's own age (time since the
READY report) is bounded only by the built 24 h (`READY_MAX_AGE_MS` **[B]**, now on `mono()`): a READY reported an
hour ago by an idle agent is still READY, which is what lets the doorbell ring an idle agent at all. The other reading
("the READY report is at most 30 s old") would never ring an agent that has been idle for more than 30 s, so it is not
taken; ZR-2 asks step 4 to word 6.9.2 this way.

4.4 **(d) "Owed answer" for `conversation = agent` (AL-4).** The ledger is the sidecar's own stage records, the only
place that sees both directions for that session (ARC4 6.14.1 S3). A conversation `c` has an **owed answer** at time
`t` when it holds a message `m` with (i) `m` handed over to this session's agent (HANDED_OVER at this receiver) and no
REPLIED or ACKNOWLEDGED_NO_ACTION from this agent for `m`; or (ii) `m` sent by this session's agent and no REPLIED or
ACKNOWLEDGED_NO_ACTION from the peer for `m` in the sender's view of the hub record; and in both cases `m` is not ended
(not DEAD, not REFUSED, not past its R-44.e "not answered" deadline: 1 h, 5 min urgent, or the per-message deadline once
P14 builds it). Receipts, notices and marked automatic content never create an owed answer (R-56.a). The value
`agent` holds for `c` while `owed(c)` and the last S3 event of `c` is within 120 s (V-19); it ends at whichever comes
first: the answer is recorded, the message ends, or 120 s pass without an S3 event in `c`. When P14's owed list
(K-12, T-3358) exists it MUST be the same computation (one function, two readers), so that the two never disagree.

4.5 **(e) The provisional claim and `last_human_at` (AL-11).** A provisional claim is taken at SessionStart, which is
not a human prompt, so it **does not** stamp `last_human_at`: the acquire record carries `provisional: true`,
`valid_until = wall() + 5 min` (claim TTL 300 000 ms) and `last_human_at: null`. `null` reads as "no human prompt",
older than any threshold. A human-origin prompt in the provisional holder's own session **promotes** it to HELD at the
same generation and stamps `last_human_at`; one in another session takes it at once with generation + 1 (6.13.2). So
E-23 (a) and E-24 both hold: E-23 (a) by the provisional rule, E-24 because a provisional holder is never checked
against the 5-minute threshold. The provisional take itself raises the generation (it is a holder change, and LEASE
says "+1 on every holder change"); D-4 (b) labels only the PROVISIONAL→HELD edge with "generation plus 1", which is
true for a take by another session and not for a promotion in the same session (ZR-3).

4.6 **(f) The 15 s lease freshness boundary for forward-once (AL-14).** Forward-once uses its own rule, stricter than
the built withhold (which trusts a read for `STALE_READS × 15 s` = 60 s **[B]**, `lease.rs`): Y forwards only on a lease
read that **completed after the message was STORED at Y** and whose age, measured on Y's `mono()` from the moment the
read's response was received to the moment the FORWARD post is issued, is **≤ 15 000 ms inclusive** (15 000 forwards,
15 001 does not). If the periodic read does not qualify, Y makes one synchronous read (timeout 10 s, the built
`RPC_TIMEOUT`); if that fails, Y does not forward and delivers as before (ARC4 7.2.d, LEASE fail direction). The bound
is a narrowing, not the safety: the safety is X's generation check (AL-15), which turns any forward on a lease that
moved into FORWARD-STALE. YR-4 (c) lapses with this answer.

4.7 **(g) Which revocation RPC failures count as a failed pull (AL-22).** Per tick the sidecar makes one pull
attempt. It is **verified** only in two cases: (1) `revocation.head` answers with a head whose signature verifies with
the pinned hub signing key and whose `(seq, digest)` equals the list held; or (2) the head shows a higher `seq`, and
`revocation.get` returns a list whose canonical form hashes to the head's digest, whose signature verifies with the
pinned key, whose `seq` equals the head's and whose `prev_digest` chains when `seq` = held + 1; the list is then
installed atomically. **Every other outcome is a failed pull**: transport error, timeout (10 s per RPC), a hub error or
refusal response on either RPC, `get` failing after a good head, a head or list with a bad or missing signature or a
signature by an unpinned key, a digest mismatch, a lower `seq` (rollback, TH-58), a broken `prev_digest` chain, a
malformed or oversized body (> 1 MiB). Timeout and refusal are treated alike for the stale rule (PN-13); they differ
only in what status says and in one thing: a signature failure, a wrong key or a rollback also raises an immediate
needs-attention entry (`revocation-tamper`, coalesced per hour), because they are evidence of interference, not of an
outage. `unverified_since` is the `mono()` time of the last verified pull, never of the first failure.

4.8 **(h) Test seams and their refusal in production (AL-16).** One gate decides every seam: a seam is honoured only
when the process runs in a marked test estate (`estate::Mode::Estate`, ARCH E-1..E-3 **[B]**), and a production-mode
process that finds a seam variable set **refuses to start** with a reason naming the variable (the sidecar), or
answers **unknown, exit 2** with that reason (the `termlink main` verbs). The seams:

| Seam | Where | Built gate today | Rule |
|---|---|---|---|
| `TERMLINK_SIDECAR_TEST_HEARTBEAT_MS`, `_STALE_MS`, `_FREEZE` | `beat.rs` | estate only, silently ignored in production **[B]** | AL-16: production start refused if set (Δ) |
| `TERMLINK_SIDECAR_TEST_LEASE_EVERY_MS` | `lease.rs` | estate only **[B]** | same |
| `TERMLINK_SIDECAR_TEST_LIVENESS_EVERY_MS`, `_LIVENESS_STALE_MS` | `liveness.rs` | estate only **[B]** | same |
| `TERMLINK_MAIN_TEST_LEASE_MS`, `_QUIET_MS`, `_COOLDOWN_MS` | `main_lease.rs` `Timing::from_env` | **none: honoured in production** **[B]** | AL-16: honoured only inside an estate; elsewhere unknown, exit 2 (Δ, B-25; QR-1) |
| `TERMLINK_SIDECAR_TEST_CLOCK` (simulated tick clock: a file the test advances) | new, B-16, B-27 | — | estate only |
| `TERMLINK_SIDECAR_TEST_FAKE_SESSION` (a socket path of a fake session answering `session.input_activity` and foreground) | new, B-27 | — | estate only |
| `TERMLINK_MAIN_TEST_CLOCK` (the lease's wall clock for `last_human_at`) | new, B-25 | — | estate only |
| `TERMLINK_SIDECAR_TEST_CEILINGS`, `TERMLINK_HUB_TEST_CEILINGS` (shrunk quotas and retention ceilings) | new, B-7, B-3 | — | estate only; the hub refuses to start with it outside an estate runtime directory |

4.8.a Mutants (diagnosis removed, a check removed) are build-time patches (`scripts/arc011-mutants.sh` **[B]**), never
environment variables, so no production binary carries them. A fake adapter needs no seam: it reports through the
same owner-only socket as a real one.
4.8.b The production scan (E-101 style, ZR-1 asks step 5 for a spec): every unit environment
(`/etc/termlink-sidecar/*.env`, `systemctl show -p Environment termlink-sidecar@*`), the hooks' environment in the
project settings and every running sidecar's `/proc/<pid>/environ` is searched for `TERMLINK_*TEST*`; a hit is exit 1.

4.9 **(i) Where the `termlink.evidence/1` record is emitted.** **By the spec runner, never by a production routine.**
A routine that graded its own run would make the evidence depend on the code under test. Each algorithm instead
names its **observation points** (`OBS` lines): decision records, audit rows, stage rows, status fields and counters
that the routine writes anyway. The runner (tier 1 and 2 test harness, tier 3 and 4 report generators, the L7
aggregated invariant check of YR-3) reads them after ACT, computes ASSERT and writes one `termlink.evidence/1` line per
run with the spec id, commit, artifact hash, tier, level, estate, verdict and the control's verdict (EV5 5.1). Three
rules: (1) an OBS that cannot be read makes the run `TOOLING` (exit 2), never `PASS`; (2) every OBS carries
`mono()` and `wall()` stamps and the `sidecar_epoch`, so a runner can order them and can see a restart; (3) no OBS
carries a secret, a key or a nonce in clear beyond the first 8 hex characters (SI-25).

4.10 **YR-4 lapses.** Its three ambiguities are settled by 4.2 (back-off), 4.5 (provisional and `last_human_at`) and
4.6 (15 s boundary).

