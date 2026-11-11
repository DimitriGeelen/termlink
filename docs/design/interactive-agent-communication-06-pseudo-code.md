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

## 5 PH-2 delivery path (the operator's priority: B-27, B-16, B-25, B-20)

### 5.1 AL-1 — Adapter report intake and the READY flag (K-32 S2, K-18; ARC4 17.2..17.6; B-27, B-29)

5.1.a **SERVES:** R-21, R-47, R-20, SI-2, SI-14, SI-17; TH-7, TH-16; AFC-14, AFC-15.
5.1.b **PROBLEM:** the doorbell, the watchdog, the conversation signal and the lease all depend on what the harness
reports; a report can be malformed, reordered, lost, forged by another local user, or claim more than it may.
**APPROACH:** one intake routine on the owner-only hook socket parses strictly, keeps per session a READY record
ordered by the adapter's clock but aged on the receiver's, and checks the prompt origin against what the receiver can
see itself (its own ring log and the PTY input counters).
5.1.c **ALGORITHM:**
```text
on_report(raw, peer):                                   # one JSON object per connection, owner-only socket
 1  if peer.uid != owner_uid: close; return                           # SI-14 [B]; FCW-1
 2  if len(raw) > 65536: count("unparseable", "too-large"); return    # FCW-2
 3  obj = strict_json(raw)                       # 3.3; Err -> count("unparseable", reason); return  (FCW-2)
 4  require obj.schema == "termlink.adapter/2"   # exact, case-sensitive
    require obj.event in {ready, session_start, prompt_submit, tool_start, tool_end, turn_stop,
                          session_end, evidence, complete}
    require obj.agent_id == MY_AGENT; obj.session_id matches ^[A-Za-z0-9._:-]{1,128}$; obj.at_ms integer >= 0
    any failure -> count("unparseable", reason); keep last 16 reasons; return               # FCW-2
 5  rx = (mono(), wall(), boot_id, sidecar_epoch)
 6  sess = sessions.get(obj.session_id)
    if none: if count(live sessions of MY_AGENT) >= 64: attention("session-table-full", coalesce=hour); return  # FCW-7
             sess = new(obj.session_id)
 7  with sess.lock:
      case obj.event:
       ready:
         require obj.state in {READY, BUSY, NOT_RUNNING} else count unparseable; return
         if obj.at_ms < sess.ready.at_ms: count("ready-reordered"); goto 8                  # FCW-3
         if obj.at_ms == sess.ready.at_ms and sess.ready.state == BUSY and obj.state != BUSY: goto 8   # tie keeps BUSY
         sess.ready = {state: obj.state, at_ms: obj.at_ms, rx, seq: sess.ready.seq + 1}
         persist sess.ready (atomic_write)          # read back by the built R-44.e notice logic [B]
       session_start:
         sess.interactive = obj.interactive (bool, required)
         sess.transcript = validate_transcript_path(obj.transcript_path)  # absent, or regular file, no symlink,
                                                    # under the harness's project directory; else none + count  (FCW-8)
       prompt_submit:
         (origin, basis) = check_origin(sess, obj)                        # below
         sess.last_prompt = {origin, basis, rx, doorbell_ids: valid_ids(obj.doorbell_ids)}
         ring_log.on_submit(sess, sess.last_prompt)                       # AL-7
       tool_start, tool_end: sess.tool = {name: obj.tool (<= 128 bytes), rx}
                              if obj.event == tool_start and obj.tool in VENDOR_TOOLS: vendor_log(sess, obj.tool)  # AL-38
       turn_stop:   sess.turn_stopped = rx
       session_end: sess.ready = {state: NOT_RUNNING, at_ms: obj.at_ms, rx, seq: +1}; persist
       evidence:    deliver.evidence(obj.nonce, obj.record_ref, sess)     # AL-9; never sets READY or origin
       complete:    deliver.complete(obj.turn_boundary | obj.exited, sess) # AL-9
      sess.s2_at = rx.mono                                                # S2 (a), every valid event
 8  reply {ok: true}

check_origin(sess, obj):
 1  o = obj.origin; b = obj.basis
    valid = {human: {pty-input, inferred}, automated: {doorbell, non-interactive, harness-mark, pty-input},
             unknown: {absent}}
    if o not in valid or b not in valid[o]: count("origin-inconsistent"); return (unknown, absent)   # FCW-4
 2  if o == human and ring_log.typed_within(sess, TICK) and obj.doorbell_ids intersects ring_log.ids(sess, TICK):
       count("origin-overridden"); return (automated, doorbell)                          # FCW-5
 3  if o == human and b == pty-input:
       src = pty_last_source_before(sess, rx.mono)          # AL-2 counters
       if src in {notify, inject}: count("origin-overridden"); return (automated, pty-input)    # FCW-5
       if src == unknown: return (human, inferred)          # the receiver cannot confirm pty-input
 4  return (o, b)
```
5.1.d **INVARIANTS:** (1) a report never makes a session READY, human or handed-over unless it parses, names this
agent and arrives on the owner-only socket; (2) per session the applied READY records have non-decreasing `at_ms` and a
strictly increasing `seq`; (3) the receiver never upgrades an origin (unknown never becomes human; automated never
becomes human); it only downgrades; (4) S2 moves only on valid reports.
5.1.e **FAILS CLOSED WHEN:**
5.1.e.1 **FCW-1** [new; E-96] the reporting peer is not the socket's owner → the connection is closed, nothing is read.
5.1.e.2 **FCW-2** [AFC-14; E-16, E-84] the report is larger than 64 KiB, not strict JSON, not schema
`termlink.adapter/2` exactly, has an unknown event, a wrong type, or another agent's id → ignored and counted; it never
reads as READY, human or evidence.
5.1.e.3 **FCW-3** [new; ZR-1] a `ready` report carries an `at_ms` older than the applied one → dropped and counted; a
tie between BUSY and READY keeps BUSY.
5.1.e.4 **FCW-4** [AFC-15; E-16, ZR-1] origin and basis do not match the contract's table → the prompt is recorded as
`unknown` (not human for the lease, human for the conversation signal).
5.1.e.5 **FCW-5** [AFC-16; E-27, E-85, ZR-1] a prompt reported `human` carries ids of a line the receiver typed in the
last tick, or the session's input counters show the last input before it came from `notify` or `inject` → recorded
`automated`.
5.1.e.6 **FCW-6** [AFC-14; ZR-1] the READY record was received by another sidecar process (`sidecar_epoch`) or on
another boot → absent for the doorbell gate (AL-5) until the next report.
5.1.e.7 **FCW-7** [new; E-16 allocation cap] 64 live sessions of the agent already exist → a report for a new session
is refused with a coalesced needs-attention entry.
5.1.e.8 **FCW-8** [new; ZR-1] the transcript path is absent, a symlink, not a regular file or outside the harness's
project directory → S2 (b) is not used for that session (turn events only).
5.1.f **EDGE CASES:**
5.1.f.1 Forged: another local user cannot reach the socket (SI-14); a same-user process can (RR-2, BP-5) and can
forge READY or `human`; the doorbell's re-checks (AL-6) and the PTY-counter override are the remaining defence; the
residue is RR-7 and AR-6.
5.1.f.2 Replayed: a replayed old `ready` report is dropped by `at_ms`; a replayed `evidence` report is harmless (AL-9
writes HANDED_OVER once per attempt and checks the nonce against the transcript itself).
5.1.f.3 Clock: `at_ms` comes from the same host (adapter and receiver share it); a wall step on the host can reorder two
reports across the step; the tie and older-than rules then keep the earlier state, and the next report repairs it.
Ages use `rx.mono` only (4.3).
5.1.f.4 Concurrency: two hook processes report at once (Stop and UserPromptSubmit in quick succession); `sess.lock`
serialises them; the `at_ms` rule makes the result independent of arrival order.
5.1.f.5 Crash table:

| Crash between | State after restart | Effect |
|---|---|---|
| 5 and 7 (report accepted, nothing applied) | previous READY record | the report is lost; the hook got no reply and does not retry (it cannot block a turn); a lost BUSY leaves a stale READY: AL-6's re-checks (foreground, veto, typing) and FCW-6 (new epoch after the restart) hold the doorbell; a lost READY leaves the session not ready: the watchdog bounds the wait |
| `sess.ready` set and persist | in memory only for this epoch; the new epoch reads absent (FCW-6) | the doorbell waits for the next report; the built notice logic reads the older persisted record |
| persist and reply | applied and persisted | none |

5.1.g **EXECUTION SEMANTICS:** intake has no effect outside the receiver; a report is applied at most once per
`(session, at_ms, event)`; duplicates of `ready` are dropped by the ordering rule; duplicates of other events move S2
twice, which changes nothing observable.
5.1.h **CRYPTOGRAPHY:** none; the boundary is the owner-only socket (SI-14). Reports are not signed (same host, same
user as the harness, ARC4 17.2).
**OBS:** OBS-1 counters `adapter_reports_unparseable`, `ready_reordered`, `origin_inconsistent`, `origin_overridden`
with the last 16 reasons; OBS-2 the per-session READY record `{state, at_ms, rx_mono, seq, sidecar_epoch}` in status
`activity`; OBS-3 the `last_prompt` record.

### 5.2 AL-2 — PTY input counters per source, in the TermLink session (K-32 S1; ARC4 6.14.1, 6.14.2; B-27)

5.2.a **SERVES:** R-20, R-81, SI-2; TH-16, TH-62; AFC-9, AFC-13.
5.2.b **PROBLEM:** "nobody is typing" must be measured, and the doorbell's own keystrokes must not count as typing.
**APPROACH:** the session tags every write into the PTY's input by its source and keeps the last time per source;
the sidecar reads them through a `notify`-scope RPC. The session also reports its foreground process.
5.2.c **ALGORITHM:**
```text
session side, every write w to the PTY master input:
 1  src = client   if w came from an attached client's input stream
          notify   if w came from command.notify                           (AL-6)
          inject   if w came from command.inject, pty inject, interact or batch input
          client   otherwise                         # cannot classify -> counts as typing (FCW-10)
 2  if src == client and w is one complete terminal reply (cursor-position report ESC[<r>;<c>R, focus ESC[I / ESC[O,
       device-attribute reply ESC[?...c): src = client_auto      # the terminal answering, not a person
 3  ctr[src].bytes += len(w); ctr[src].last_mono = mono(); ctr[src].writes += 1

RPC session.input_activity (scope: notify or above):              # FCW-9: observe scope refused
 reply {boot_id, mono_now: mono(),
        sources: {client, client_auto, notify, inject}: {last_mono | null, bytes, writes},
        foreground: {pgid, leader_pid, exe_realpath | null, comm}}     # tcgetpgrp on the PTY master
```
5.2.d **INVARIANTS:** (1) a `command.notify` write is never counted as `client`; (2) every byte written to the PTY input
is counted under exactly one source; (3) the counters only grow.
5.2.e **FAILS CLOSED WHEN:**
5.2.e.1 **FCW-9** [AFC-13; E-15] the caller's token has only `observe` → `session.input_activity` is refused.
5.2.e.2 **FCW-10** [new; ZR-1] a write's source cannot be classified → it counts as `client` (someone may be typing).
5.2.e.3 **FCW-11** [AFC-13; ZR-1] the sidecar's `boot_id` differs from the session's → the sidecar treats S1 as
unknown (AL-3).
5.2.f **EDGE CASES:** (1) a person pasting: one large client write, counted as typing (correct); (2) a terminal that
sends periodic focus events counts as `client_auto`, never holds the doorbell; anything else unrecognised holds it
(the safe side); (3) concurrency: writes are serialised by the PTY writer; counter updates are atomic per write;
(4) crash of the session: the session ends, the harness is gone, the foreground check (AL-6) fails; counters are in
memory and need no recovery.
5.2.g **EXECUTION SEMANTICS:** read-only for the caller; the counters describe writes that already happened.
5.2.h **CRYPTOGRAPHY:** none; the scope check is the built token scope (`notify` below `interact`, T-3533) **[B]**.
**OBS:** OBS-4 the RPC reply per tick (sidecar ring log keeps the S1 age per decision).

### 5.3 AL-3 — Activity meter: S1, S2, S3 and last activity (K-32; ARC4 6.14.1..6.14.3, 6.14.5; B-27)

5.3.a **SERVES:** R-19, R-20, R-44, R-29; FX-8; AFC-9, AFC-10, AFC-13, AFC-27.
5.3.b **PROBLEM:** the watchdog fires on "nothing happened for 300 s"; a source that cannot be read must not look
silent. **APPROACH:** three sources, each with its own time and a known/unknown state; `last_activity` is their maximum
and the watchdog refuses to run when a source it needs is unknown.
5.3.c **ALGORITHM:**
```text
meter_tick(sess):                                   # once per tick, before AL-5 and AL-8
 1  r = session_rpc(sess, session.input_activity, timeout 2 s)
    if Err or r.boot_id != boot_id: sess.s1 = UNKNOWN(since = sess.s1.known_until or now)    # FCW-12
    else: sess.s1 = KNOWN(at = r.sources.client.last_mono); sess.fg = r.foreground; sess.s1_read = mono()
 2  if sess.transcript: st = lstat(sess.transcript)
        if st ok and (st.size, st.mtime) != sess.tr_last: sess.tr_last = (st.size, st.mtime); sess.s2_at = mono()
 3  sess.s2_known = (sess has had >= 1 valid report since boot) or (sess.transcript != none)    # FCW-13
 4  # S3 is written synchronously, not here: send API on each own send from sess, AL-9 on each HANDED_OVER to sess
 5  known = [s for s in (s1, s2, s3) if known]
    sess.last_activity = max(at of known); sess.last_source = argmax
 6  status.activity[sess] = {s1_at, s2_at, s3_at, last, source, conversation (AL-4), stamped_at: mono()}

never counted (6.14.2): arrivals (STORED), receipts, notices, marked automatic content, keep-alive traffic,
                        notify and inject keystrokes, client_auto bytes                       # FCW-14
```
5.3.d **INVARIANTS:** (1) `last_activity` moves only on S1 client input, a valid adapter report, transcript growth, an
own send from the session, or a hand-over to it; (2) a source that is unknown is never read as "no activity".
5.3.e **FAILS CLOSED WHEN:**
5.3.e.1 **FCW-12** [AFC-13; E-15, ZR-1] the input-activity RPC fails, times out (2 s) or answers from another boot →
S1 is unknown: no normal ring (AL-5) and no watchdog (AL-8) for that session.
5.3.e.2 **FCW-13** [AFC-10; E-10] the session has no valid adapter report since boot and no transcript path → S2 is
unknown: status `watchdog: unavailable`, no fire.
5.3.e.3 **FCW-14** [AFC-9; E-9, E-17] arrivals, receipts, notices, automatic content, keep-alives, `notify`, `inject`
and `client_auto` input → never activity.
5.3.e.4 **FCW-15** [AFC-27; E-19] a status `activity` field older than 2 ticks → reads `unknown`, never its last value.
5.3.f **EDGE CASES:** (1) a transcript rotated or truncated: size changes, counts as S2 activity once (harmless, it
delays a fire by at most 300 s); (2) the mtime is wall-clock but only compared for change, never subtracted; (3) a
session whose PTY ended: RPC fails, S1 unknown, no typing at all; (4) concurrency: S3 writes come from the send API and
AL-9 under `sess.lock`; the tick reads under the same lock; (5) crash: the meter is in memory; after a sidecar restart
every session starts with S1 read at the first tick and S2 unknown until its first report or a transcript path is
known, so no watchdog fires in the first moments after a restart (FCW-13); S3 restarts empty, which makes the session
look *more* silent; that is safe only because S1 and S2 must also be known and silent.
5.3.g **EXECUTION SEMANTICS:** no effect outside the sidecar; recomputed every tick.
5.3.h **CRYPTOGRAPHY:** none.
**OBS:** OBS-5 status `activity` per session with stamps; OBS-6 per ring and watchdog decision the three ages (AL-6,
AL-8).

### 5.4 AL-4 — Conversation signal (K-32; ARC4 6.14.4; B-27; answer 4.4)

5.4.a **SERVES:** R-20, R-81, SI-2; TH-62; AFC-15.
5.4.b **PROBLEM:** a doorbell line must not land while a person is working with the agent, or in the middle of an
agent-to-agent exchange about something else. **APPROACH:** a per-session value computed from S1, S2 (a), S3, the
prompt origin and the owed-answer ledger, never from READY.
5.4.c **ALGORITHM:**
```text
conversation(sess, now):
 1  p = sess.last_prompt
    if p and p.origin in {human, unknown, absent}:                     # FCW-16: unknown fails toward "a person"
        stopped = the first turn_stop or READY report of sess with at_ms > p.at_ms, else none
        end = (stopped.rx.mono if stopped else max(p.rx.mono, last S2 event of sess after p)) + 120 s  # FCW-17
        if sess.s1 is KNOWN and sess.s1.at > p.rx.mono: end = max(end, sess.s1.at + 120 s)
        if now <= end: return operator
 2  ledger = owed_ledger(sess)            # answer 4.4; Err -> return unknown                   (FCW-18)
    C = {c : ledger.owed(c, now) and now - ledger.last_s3(c) <= 120 s}
    if C nonempty: return agent(C)
 3  return none

owed(c, now): exists m in c with
     (m handed over to this session's agent and no REPLIED / ACKNOWLEDGED_NO_ACTION by this agent for m)
  or (m sent by this session's agent and no REPLIED / ACKNOWLEDGED_NO_ACTION by the peer for m in the hub record)
  and m not ended (DEAD, REFUSED, or past its R-44.e "not answered" deadline); receipts, notices and marked
  automatic content are never m
```
5.4.d **INVARIANTS:** (1) the READY flag is not an input; the same events give the same value for every READY value;
(2) an `automated` prompt never produces `operator`; (3) `agent` never holds mail of the conversation under way.
5.4.e **FAILS CLOSED WHEN:**
5.4.e.1 **FCW-16** [AFC-15; E-18] the last prompt's origin is `unknown` or `absent` → `operator` (no ring).
5.4.e.2 **FCW-17** [new; ZR-1] no turn stop follows the prompt → `operator` lasts until 120 s after the last S2 event
after the prompt, not 120 s after the prompt.
5.4.e.3 **FCW-18** [new; ZR-1] the owed-answer ledger cannot be read → `unknown`: AL-5 treats it like `operator`
(no normal ring); the watchdog is unaffected.
5.4.f **EDGE CASES:** (1) a person reading for longer than 120 s without typing is not seen (ARC4 15 K-32 "does not
prove"); (2) many short conversations: `agent(C)` holds only other conversations' mail; with 20 parallel owed
conversations everything outside C still rings; (3) the stage memory ages out after 14 days (SI-4): an owed answer
older than that is ended by its deadline long before; (4) concurrency: computed under `sess.lock` from records that are
themselves written under it; (5) crash: recomputed from durable records (stage chain) and in-memory reports; after a
restart `last_prompt` is lost, so a person mid-conversation is protected only by the typing veto (30 s) until their
next prompt; QR-2 records this.
5.4.g **EXECUTION SEMANTICS:** pure function of recorded facts; no effect.
5.4.h **CRYPTOGRAPHY:** none.
**OBS:** OBS-7 `conversation` in status `activity` and in every ring decision record.

### 5.5 AL-5 — Doorbell ring decision, the gate (K-19; ARC4 6.9.1; B-16)

5.5.a **SERVES:** R-19, R-20, R-21, R-3; SI-2; TH-16, TH-62; AFC-1, AFC-5, AFC-13.
5.5.b **PROBLEM:** decide, once per tick, whether a session gets a doorbell line and which ids it names.
**APPROACH:** evaluate the four ruled conditions (O-2 D+) plus the schedule on a snapshot, choose at most one session per
agent, and hand the snapshot to AL-6 for the re-checks.
5.5.c **ALGORITHM:**
```text
ring_tick():                                                       # once per tick, after meter_tick for all sessions
 1  for sess in sessions of MY_AGENT: decide(sess) -> decision record (always written, OBS-8)
 2  candidates = [sess with decision == ring]
    if len(candidates) > 1: keep the one with the largest last_activity; ties: lowest session id   # FCW-26
 3  for the chosen sess: type_ring(sess, decision.ids, decision.count, decision.snapshot)        # AL-6

decide(sess):
 1  P = [m for m in store.pending(MY_AGENT)
         if m.stage == STORED and no HANDED_OVER and no open claim (AL-9) and not held (gap wait AL-36, OQ-2 held,
            drained) and not forward_intent (AL-14) and lease.would_hand_over(m, sess) (AL-12)]       # FCW-19
 2  P = [m in P if now - m.stored_mono >= TICK and now - m.last_pull_mono >= TICK]                   # FCW-20
 3  if P empty: return hold("nothing")
 4  if not sess.pty_launched or sess.harness_exe is none: return hold("not-a-termlink-pty")          # FCW-24
 5  r = sess.ready
    if r.state != READY or r.sidecar_epoch != sidecar_epoch or now - r.rx.mono > 24 h:
        return hold("not-ready")                                                                     # FCW-21
 6  if sess.s1 is UNKNOWN or (sess.s1.at and now - sess.s1.at < 30 s): return hold("typing")          # FCW-22, V-17
 7  cv = conversation(sess, now)                                                                     # AL-4
    if cv in {operator, unknown}: return hold("conversation")                                        # FCW-23
    if cv == agent(C): P = [m in P if m.conversation in C]; if P empty: return hold("conversation")
 8  if not schedule.may_ring(sess, now): return hold(schedule.reason)                                # FCW-25 (AL-7)
 9  order P by priority desc, stored_mono asc                    # the built hand-over order [B]
    return ring(ids = first 4 of P, count = len(P),
                snapshot = {ready_seq: r.seq, s1_read: sess.s1_read, conversation: cv})
```
5.5.d **INVARIANTS:** (1) a ring decision implies all of 1..8 held on one snapshot; (2) at most one session of an agent
and at most one ring per session per tick; (3) every decision, ring or hold, is recorded with its inputs and their
ages.
5.5.e **FAILS CLOSED WHEN:**
5.5.e.1 **FCW-19** [AFC-1; E-1] no message is STORED, not handed over, without an open claim, not held, not being
forwarded and not withheld from this session by the lease → no ring.
5.5.e.2 **FCW-20** [AFC-1; E-1] every pending message was stored, or touched by a hook pull, less than 1 tick ago →
no ring (the hook path goes first).
5.5.e.3 **FCW-21** [AFC-1, AFC-14; E-1] READY is not READY, is absent, was received by another sidecar process or is
older than 24 h → no ring.
5.5.e.4 **FCW-22** [AFC-1, AFC-13; E-1, E-15] client input in the last 30 s, or S1 unknown → no ring.
5.5.e.5 **FCW-23** [AFC-1; E-1, E-18] conversation `operator` or `unknown` → no ring; `agent(C)` → only mail of C.
5.5.e.6 **FCW-24** [AFC-13; E-15] the session is not a TermLink PTY session started by the reachable launcher, or its
harness executable is not registered → no ring (and no watchdog, AL-8).
5.5.e.7 **FCW-25** [AFC-5; E-5] the schedule (AL-7) does not allow a ring yet, or a typed line is unsubmitted → no ring.
5.5.e.8 **FCW-26** [new; ZR-1] more than one session of the agent could be rung → only one is.
5.5.f **EDGE CASES:** (1) a message arriving for a session that is withheld by the lease (another session of this agent
is main) is never rung there: `lease.would_hand_over` is the same predicate as the built withhold (AL-12); (2) time:
every comparison on `mono()`; `stored_mono` is the receiver's own STORED time; (3) concurrency: the hook path may pull
between `decide` and `type_ring`; AL-6 re-reads; a ring for mail just handed over is a stray ring counted by AL-9
(FCW-47); (4) crash: `decide` writes only its decision record; a crash loses at most one tick's decisions.
5.5.g **EXECUTION SEMANTICS:** decides only; typing is AL-6.
5.5.h **CRYPTOGRAPHY:** none.
**OBS:** OBS-8 the decision record per session per tick `{tick, session, outcome: ring|hold:<reason>, ids, count,
inputs: {pending, ready: {state, seq, age}, s1_age, conversation, schedule}}`.

### 5.6 AL-6 — Pre-typing re-checks, the fixed line, and the keystrokes (K-19, K-18; ARC4 6.9.2, 6.9.3; B-16)

5.6.a **SERVES:** SI-1, SI-2, SI-23, R-22, R-23, R-81; TH-6, TH-16, TH-40, TH-62; AFC-2, AFC-3, AFC-12.
5.6.b **PROBLEM:** between the gate and the keystrokes the session can change; and the typed bytes must be inert and
carry no peer byte. **APPROACH:** re-check in the ruled order, each failing closed, immediately before typing; build
the line from a fixed template, a decimal count and validated ids on both sides of the RPC; write a durable intent
before the keystrokes and the outcome after.
5.6.c **ALGORITHM:**
```text
type_ring(sess, ids, count, snap, via = doorbell):          # via = watchdog when called from AL-8 (checks differ)
 1  if not delivery_on(): return held("kill-switch")                                  # FCW-27 (AL-10)
 2  if via == doorbell:
      r = reread(sess.ready)
      if r.state != READY or r.seq != snap.ready_seq or r.sidecar_epoch != sidecar_epoch:
          return held("ready")                                                         # FCW-28
 3  fg = session_rpc(sess, session.input_activity, 2 s)
    if Err or fg.foreground.exe_realpath not in sess.harness_exes or fg.foreground.leader_pid != sess.harness_pid:
        return held("foreground")                                                      # FCW-29
 4  v = activity.measure(sess.pty)                    # built process-tree veto, 1.5 s window [B]
    if v == Active(sig): return held("veto:" + sig)                                    # FCW-30
    if v == Unmeasurable and via == watchdog: return held("veto-unmeasurable")         # FCW-41 (AL-8)
    # via == doorbell and Unmeasurable: no veto, readiness decides (OQ-16 D 3, ruled) [B]
 5  if via == doorbell:
      s1 = session_rpc(sess, session.input_activity, 2 s)       # re-read after the veto window
      if Err or (s1.client.last_mono and mono() - s1.client.last_mono < 30 s): return held("typing")   # FCW-31
      if conversation(sess, mono()) not compatible with snap.conversation and ids: return held("conversation")  # FCW-32
 6  line = compose(ids, count)                        # Err -> refuse; flag every id that failed     (FCW-33)
 7  t_last = mono()
    if Err(ring_log.append_intent({ring_id: new_id(), sess, ids, count, via, checks: [1..5 results],
                                   t_last_check: t_last})): return held("intent-unrecorded")   # FCW-35
 8  if mono() - t_last > TICK: return held("stale-checks")                             # re-read older than 1 tick (FCW-28)
 9  res = session_rpc(sess, command.notify, {ids, count}, timeout 5 s)
10  ring_log.append_outcome(ring_id, typed | failed(<error>) | unknown(timeout), check_to_keystroke_ms = mono() - t_last)
11  if res is typed or unknown:                                                         # FCW-36: unknown = maybe typed
        for m in ids: hub.stage(m, ATTEMPTED, {via, ring_id}); store.chain_append(ATTEMPTED m via ring_id)
        schedule.on_typed(sess, ring_id, ids, via)                                      # AL-7
        sess.last_typed_mono = mono()
12  return res

compose(ids, count):                                            # sidecar side; the session repeats 1..3
 1  require 1 <= len(ids) <= 4 and ids distinct and every id matches ^[0-9a-f]{32}$
 2  require count is an integer and len(ids) <= count <= 1000000
 3  return "sidecar: " + decimal(count) + " peer messages waiting, ids " + join(" ", ids)
            + ". The prompt hook shows them as untrusted data."

session side, command.notify {ids, count}:                     # Δ replaces the built handler (T-3533 line)
 1  validate as compose 1..2; failure -> INVALID_PARAMS, nothing typed                  # FCW-34
 2  write compose(ids, count) as one write, then Enter as a second write, both tagged source = notify (AL-2)
```
5.6.c.1 Test vector (SI-1): `ids = [0123456789abcdef0123456789abcdef, fedcba9876543210fedcba9876543210]`, `count = 5`
→ `sidecar: 5 peer messages waiting, ids 0123456789abcdef0123456789abcdef fedcba9876543210fedcba9876543210. The prompt
hook shows them as untrusted data.` (152 bytes, then Enter). Refused: an id of 31 or 33 hex, upper-case hex, an empty
id, five ids, two equal ids, `count = 1` with two ids, `count = "5"` (a string), `count = 5.0`.
5.6.c.2 The line matches EV5 E-3's regular expression and SI-1's alphabet (letters, digits, spaces, `.`, `,`, `:`);
its first word `sidecar:` is not an installed command (E-125).
5.6.d **INVARIANTS:** (1) no byte of a message other than its validated id reaches the line; (2) every keystroke has an
intent record written before it; (3) a typed line never produces HANDED_OVER (only AL-9 does); (4) the interval from
the last re-check to the keystrokes is recorded for every ring (H-11 (a)).
5.6.e **FAILS CLOSED WHEN:**
5.6.e.1 **FCW-27** [AFC-12; E-2, E-12] delivery is switched off → nothing typed.
5.6.e.2 **FCW-28** [AFC-2; E-2, ZR-1] the READY record re-read is not READY, has another `seq` or `sidecar_epoch`, or
the checks were made more than 1 tick before the keystrokes → nothing typed.
5.6.e.3 **FCW-29** [AFC-2; E-2, E-8] the foreground process is not the registered harness, or cannot be read → nothing
typed.
5.6.e.4 **FCW-30** [AFC-2; E-2] the process-tree veto reports activity → nothing typed.
5.6.e.5 **FCW-31** [AFC-2; E-2] the re-read shows client input in the last 30 s, or the re-read fails → nothing typed.
5.6.e.6 **FCW-32** [AFC-2; E-2] the conversation signal changed to `operator`, `unknown`, or `agent` of another
conversation → nothing typed.
5.6.e.7 **FCW-33** [AFC-3; E-3] an id is not 32 lowercase hex, more than 4 ids, duplicate ids, or a count that is not
a decimal integer in [number of ids, 10^6] → no line; the message is flagged.
5.6.e.8 **FCW-34** [AFC-3; E-3] the session's `command.notify` receives such parameters → it types nothing and answers
INVALID_PARAMS.
5.6.e.9 **FCW-35** [new; ZR-1] the ring intent cannot be written durably → nothing typed.
5.6.e.10 **FCW-36** [AFC-5; ZR-1] the notify RPC times out → the outcome is `unknown`, treated as typed for the
schedule (no re-ring until a submit or the watchdog) and never as handed over.
5.6.f **EDGE CASES:**
5.6.f.1 Forged ids: a peer chooses its message id; ids are validated at STORED (AL-23) and again here and at the
session; an id that fails is never typed. Altered: the line has no peer text to alter.
5.6.f.2 Race (RR-19, AR-2): a turn that starts after step 5 and before step 9 receives the line in a busy prompt; the
race is narrowed (each check re-read, the interval logged), not closed; E-14 is its limit probe.
5.6.f.3 Concurrency: AL-8 and AL-6 for the same session are serialised by `sess.ring_lock` (one typed line per session
at a time); the hook pull may hand the mail over during the checks; the line then names handed-over ids and the
resulting pull is a stray ring (AL-9).
5.6.f.4 Crash table:

| Crash between | State after restart | Recovery |
|---|---|---|
| 1..6 and 7 | nothing recorded | none; the next tick decides again |
| 7 and 9 (intent, no keystrokes) | intent without outcome | treated as `unknown` (typed, unsubmitted) by AL-7 (FCW-37); at worst one ring is skipped until the watchdog interval |
| 9 and 10 (keystrokes, no outcome) | intent without outcome | same; the H-11 (e) counter counts it if no submit follows |
| 10 and 11 (outcome, no ATTEMPTED stage) | outcome recorded | on start, every intent with outcome `typed` or `unknown` and no ATTEMPTED line is completed (stage writes are idempotent) |

5.6.g **EXECUTION SEMANTICS:** at most one `command.notify` per intent (no retry inside a tick); the target effect
(a line in the prompt) can happen while the record says `unknown`; the state `unknown` is reconciled not by
re-typing but by the submit or its absence (AL-7) and by the evidence check (AL-9). Target binding before dispatch:
the foreground process is checked against the registered harness pid and executable (step 3), not only afterwards.
5.6.h **CRYPTOGRAPHY:** none; the `notify` token scope is the binding **[B]**.
**OBS:** OBS-9 the ring log `{ring_id, session, ids, count, via, checks, t_last_check, outcome,
check_to_keystroke_ms}`; OBS-10 the hub stage ATTEMPTED with `via` and `ring_id`.

### 5.7 AL-7 — Ring schedule: back-off and the unsubmitted line (K-19; ARC4 6.9.4, 6.9.5; B-16; answer 4.2)

5.7.a **SERVES:** R-20, R-25, R-44; SI-2; AFC-5, AFC-6.
5.7.b **PROBLEM:** a ring that does not lead to a hand-over must not be repeated every tick, and a line sitting unsent
in the input must not be typed over. **APPROACH:** a per-session state machine with the back-off function of 4.2.
5.7.c **ALGORITHM (state table; D-1.2 draws it):**

| State | Event | Guard | Next | Action |
|---|---|---|---|---|
| IDLE | typed (AL-6) | — | AWAIT_SUBMIT | `k = 0`, `T0 = tick`, `last = tick`, `pending_ring = ring_id` |
| BACKOFF | typed | — | AWAIT_SUBMIT | `k = k + 1`, `last = tick`, `pending_ring = ring_id` |
| AWAIT_SUBMIT | prompt_submit (any origin) | within 1 tick of the ring | BACKOFF | `pending_ring = none`; if its `doorbell_ids` are all handed over or unknown: stray + 1 (H-11 (b)) |
| AWAIT_SUBMIT | tick | more than 1 tick since the ring, no submit | UNSUBMITTED | H-11 (e) + 1 |
| UNSUBMITTED | prompt_submit (any origin) | — | BACKOFF | the input buffer was submitted, whatever it held |
| UNSUBMITTED | watchdog typed (AL-8) | — | AWAIT_SUBMIT | `pending_ring = ring_id` (k unchanged) |
| any | HANDED_OVER to sess, or pending set empty | — | IDLE | progress: `k = 0` |
| any | session_start or NOT_RUNNING | — | IDLE | a new session has a new input buffer |

```text
may_ring(sess, now):                            # used by AL-5 step 8
 1  if sess.ring_state in {AWAIT_SUBMIT, UNSUBMITTED}: return no("unsubmitted")                   # FCW-25
 2  if sess.ring_state == IDLE: return yes
 3  O(k) = 0 if k = 0;  4 if k = 1;  12 if k = 2;  12 + 20 (k - 2) if k >= 3     # ticks after T0 for ring k
    if tick(now) < T0 + O(k + 1): return no("backoff")                                              # FCW-25
 4  if tick(now) == sess.last: return no("one-per-tick")
 5  return yes

on_start():                                     # after a sidecar restart
 1  for each ring intent with no outcome, or outcome typed|unknown and no submit recorded after it:
        set its session UNSUBMITTED (pending_ring = that ring)                                     # FCW-37
```
5.7.d **INVARIANTS:** (1) normal ring `k` of an episode is no earlier than `T0 + O(k)`; (2) while a typed line has no
submit, no normal ring; (3) at most one ring per session per tick; (4) progress resets the episode.
5.7.e **FAILS CLOSED WHEN:**
5.7.e.1 **FCW-25** (defined in AL-5) the schedule or an unsubmitted line forbids a ring.
5.7.e.2 **FCW-37** [AFC-5; ZR-1] after a restart a ring intent has no outcome, or a typed line has no recorded submit →
the session is UNSUBMITTED (no normal ring until a submit or the watchdog).
5.7.f **EDGE CASES:** (1) a restored stale line (T-3591): a submit whose `doorbell_ids` are all handed over pulls
nothing and counts as stray; it moves AWAIT_SUBMIT or UNSUBMITTED to BACKOFF, which is correct (the buffer was
submitted); (2) a person types their own text on top of an unsent line and submits: one submit, state BACKOFF; the
mail is pulled by that prompt's hook (no harm, it is framed as untrusted data); (3) time: ticks counted on `mono()`;
(4) concurrency: transitions under `sess.ring_lock`; (5) crash: table above (on_start), the state is otherwise
rebuilt from the ring log and the submit records.
5.7.g **EXECUTION SEMANTICS:** decides only.
5.7.h **CRYPTOGRAPHY:** none.
**OBS:** OBS-11 ring state per session in status `activity.doorbell {state, k, next_tick}`; H-11 (b) and (e) counters
with numerator and denominator (rings).

### 5.8 AL-8 — Doorbell watchdog, 300 s, independent of READY (K-33; ARC4 6.9.7, 6.14.3; B-16)

5.8.a **SERVES:** R-19, R-44, R-24; SI-2, SI-23; AFC-7, AFC-8, AFC-10, AFC-11, AFC-13.
5.8.b **PROBLEM:** a stuck or false READY flag (or a stuck conversation value) must never hold mail forever, yet the
watchdog types even when READY says busy, so it must be safe without READY. **APPROACH:** its own loop, keyed on
measured silence of all three sources, with only the checks that cannot be stuck (kill switch, foreground, veto), an
audit row for every fire and every hold, and a silence count that restarts at every typed line.
5.8.c **ALGORITHM (D-1.3 draws its states):**
```text
watchdog_tick(sess):                                # its own loop, once per tick, after meter_tick
 1  P = pending mail for sess as AL-5 step 1 (no 1-tick hook grace)
    if P empty: sess.wd = QUIET; return
 2  if not sess.pty_launched or sess.s1 is UNKNOWN or not sess.s2_known:
        sess.wd = UNAVAILABLE(reason); status "watchdog: unavailable (<reason>)"; return        # FCW-39
 3  since = max(sess.last_activity, sess.last_typed_mono, sess.wd_last_fire)                    # FCW-38
    if mono() - since < 300 s: sess.wd = COUNTING; return                                        # V-18
 4  row = {kind: "doorbell-watchdog", session, at: (mono, wall), silent_for_s, ready_flag: busy|ready|absent,
           ready_age_s, last_activity: {s1, s2, s3}, checks: [], outcome: pending, message_ids}
    res = type_ring(sess, first 4 ids of P by priority and age, len(P), snap = none, via = watchdog, intent = row)
       # AL-6 runs steps 1 kill switch, 3 foreground, 4 veto (Unmeasurable holds), then 6..11; it does not check
       # READY, typing or the conversation. Its step 7 writes `row` (checks filled) to the ADMIN CHAIN as the
       # intent, before any keystroke (FCW-42); its step 10 appends the outcome typed | failed | unknown to that row.
 5  if res is held(check):
        if (sess, check) not already rowed in this silence episode: admin_row(row with outcome = held:<check>)  # FCW-40
        return
 6  sess.wd_last_fire = mono(); sess.wd = FIRED
    if row.ready_flag == busy: attention("ready-busy-while-silent", "READY flag busy for N s while nothing happened",
                                         coalesce = (sess, day))
```
5.8.c.1 Note on order: the audit row is part of AL-6's intent (step 7 of AL-6 writes the intent; for `via =
watchdog` the intent record **is** the admin row, written to the admin chain before the keystrokes, and its outcome
is appended after). So a fire without a row is impossible, and a row with outcome `unknown` means "may have typed".
5.8.d **INVARIANTS:** (1) every fire is preceded by at least 300 s in which no S1, S2 or S3 event happened and no line
was typed into the session; (2) if mail stays pending, the session stays silent and unvetoed, all sources are known and
the switch is on, a fire happens within 300 s + 1 tick; (3) every fire and every hold kind has an admin row; (4) READY
and the conversation signal are never inputs.
5.8.e **FAILS CLOSED WHEN:**
5.8.e.1 **FCW-38** [AFC-7, AFC-11; E-7, E-11] less than 300 s since the latest of: the last activity, the last line
typed into the session (doorbell or watchdog), the last fire → no fire.
5.8.e.2 **FCW-39** [AFC-10, AFC-13; E-10] the session is not a TermLink PTY session, or S1 or S2 is unknown → no fire;
status says `watchdog: unavailable` with the reason.
5.8.e.3 **FCW-40** [AFC-8; E-8] the switch is off, the foreground is not the harness, or the veto reports activity →
held, with one audit row per hold kind per silence episode.
5.8.e.4 **FCW-41** [AFC-8; ZR-1, ZR-4] the veto cannot measure (no `/proc`, no `ss`) → held (the watchdog never types
blind; the normal ring keeps OQ-16 D 3).
5.8.e.5 **FCW-42** [new; ZR-1] the audit row cannot be written → nothing typed.
5.8.f **EDGE CASES:** (1) a long, silent tool run with CPU, network or disk in the agent's tree: the veto holds (AR-5's
case narrows to an idle wait); (2) an idle wait inside a turn (no CPU, no output): the watchdog types; the line waits in
the harness's input as the next prompt (AR-5, ruled acceptable by O-2); (3) a normal ring that was not submitted
restarts the silence count, so the watchdog never types over it within 300 s (ARC4 6.9.5 "after its full silence
interval"); (4) clock: `mono()` only; (5) concurrency: `sess.ring_lock` as AL-6; (6) crash: `wd_last_fire` and
`last_typed_mono` are rebuilt from the admin chain and the ring log at start (both durable), so a restart never shortens
the 300 s.
5.8.g **EXECUTION SEMANTICS:** as AL-6 (intent before keystrokes, outcome after, `unknown` treated as typed).
Delivery stays strict: a fire is ATTEMPTED with `via: watchdog`; HANDED_OVER only from evidence (AL-9).
5.8.h **CRYPTOGRAPHY:** none.
**OBS:** OBS-12 the admin-chain rows `kind: doorbell-watchdog`; OBS-13 hub stage `via: watchdog`; OBS-14 the
needs-attention entry `ready-busy-while-silent`; OBS-15 status `watchdog` per session.

### 5.9 AL-9 — Pull, claim, hand-over evidence and HANDOVER_UNCONFIRMED (K-18 path; ARC4 6.9.4, 17.6; as built plus Δ)

5.9.a **SERVES:** R-24, R-42, R-51, FX-7; SI-17, SI-30; TH-7, TH-63; AFC-4, AFC-6.
5.9.b **PROBLEM:** a ring is not a delivery; HANDED_OVER must come only from evidence in a harness-written record, and
one message must never be handed over twice. **APPROACH:** as built (ARCH C-9, C-11, `deliver.rs`): an atomic claim per
attempt before content leaves the store, a nonce in the frame, a finalizer that looks for the nonce in the transcript.
Δ for B-16: the UserPromptSubmit hook pulls; the 90 s label HANDOVER_UNCONFIRMED is posted to the sender; doorbell ids
in a submitted prompt are matched for the stray count.
5.9.c **ALGORITHM:**
```text
pull(sess, hook):                                            # [B] deliver.rs, from a hook (post-tool-use, Δ user-prompt-submit)
 1  if not delivery_on(): refuse "delivery-off"                                             # FCW-50 (AL-10)
 2  if lease.withhold(MY_AGENT, sess): reply the one line, once per generation; hand out nothing    # FCW-48 [B]
 3  items = []
    for m in store.offerable(MY_AGENT) ordered by priority, age:     # STORED, allow-listed, not handed over, no open claim
        rerun receiver checks (policy, revocation AL-22, switch)    # [B] the second run of ARCH 7.1
        n = new nonce (32 hex)
        if store.claim(m.id, m.next_attempt, n, now) == AlreadyClaimed: continue           # FCW-43 [B] O_CREAT|O_EXCL
        atomic_write(delivery-<n>, {msg, attempt, nonce, ts, session}); chain_append(CLAIMED m)
        items += m
 4  if items: spawn finalizer(n)                                     # [B] detached, re-execs the binary
    return frame(items, n)                                           # [B] SI-7 frame with the nonce in its header

finalize(n, final_check):                                    # [B], every few seconds up to the window
 1  rec = attempt(n); ev = adapter.find(transcript(rec.session), header_prefix(n))
    ev only counts in a harness-written hook-context or user record                         # FCW-44 [B] SI-17
 2  if ev found: chain_append(HANDED_OVER m, attempt); hub.stage(m, HANDED_OVER, {record_ref}); S3(sess) = now; done
 3  if transcript unreadable: chain_append(UNCERTAIN); attention("hand-over uncertain"); attempt stays open   # FCW-46
 4  if now - rec.ts >= 90 s and not labelled(n):                                              # Δ
        notice(sender, ATTEMPTED, {label: HANDOVER_UNCONFIRMED, attempt}); mark labelled(n)   # FCW-45; once per attempt
 5  if complete point after rec.ts reached (adapter `complete`, AL-1) and ev absent:
        chain_append(ABSENT m, attempt); m offered again as attempt + 1 with a new nonce     # [B] proof of absence
 6  elif final_check: chain_append(UNCERTAIN); attention; attempt stays open for the operator # FCW-46 [B] SI-30

on_submit doorbell ids (from AL-1 prompt_submit):            # Δ stray count, H-11 (b)
 1  for id in doorbell_ids: if id is HANDED_OVER, or not in the ring log of the last 24 h: stray += 1   # FCW-47
```
5.9.d **INVARIANTS:** (1) the only writer of HANDED_OVER is `finalize` step 2 **[B]**; (2) at most one open attempt per
message **[B]**; (3) every attempt ends HANDED_OVER, ABSENT (then a new attempt) or UNCERTAIN (operator), never by a
timeout alone **[B]**; (4) HANDOVER_UNCONFIRMED is a label on ATTEMPTED, posted at most once per attempt, never a state.
5.9.e **FAILS CLOSED WHEN:**
5.9.e.1 **FCW-43** [AFC-6; E-98, E-6] the claim for `(message, attempt)` already exists → that message is not handed out
again.
5.9.e.2 **FCW-44** [AFC-4; E-4, E-97] the nonce appears only in an assistant record, a tool result, or nowhere → never
HANDED_OVER.
5.9.e.3 **FCW-45** [AFC-4; E-4] no evidence 90 s after the claim → the sender is told HANDOVER_UNCONFIRMED; the message
stays ATTEMPTED and owed.
5.9.e.4 **FCW-46** [AFC-4; E-98] the transcript cannot be read, or the final check passes with no evidence-complete
point → UNCERTAIN, needs-attention, no redelivery until the operator decides.
5.9.e.5 **FCW-47** [AFC-6; E-6] a submitted prompt names doorbell ids that are already handed over or were never rung
→ nothing is pulled for them; a stray ring is counted.
5.9.e.6 **FCW-48** [AFC-16; ZR-1] a fresh lease names another session of this agent as main → the pull hands out
nothing and prints the one line once per generation **[B]**.
5.9.f **EDGE CASES:** (1) replay of a frame's nonce by the agent's own output: assistant record, not evidence; a
same-user process writing a well-formed user record is RR-7 (limit probe E-97); (2) two hooks pulling at once: one
claim wins **[B]**; (3) clock: the 90 s window on `mono()` of the finalizer; (4) crash table **[B]** (`KillPoint`):

| Crash between | State after restart | Recovery |
|---|---|---|
| claim file and attempt record (`AfterClaim`) | claim exists, no record | the claim blocks a second hand-over; the attempt is resolved by the evidence search on start; no evidence and no point: UNCERTAIN |
| record and CLAIMED (`AfterRecord`), CLAIMED and frame (`AfterClaimed`) | attempt open | the same search; the frame may not have reached the agent: ABSENT only with the complete point |
| evidence found and HANDED_OVER line (`BeforeHandedOver`) | attempt open, evidence in transcript | the next search finds it and writes HANDED_OVER once |
| proof of absence and ABSENT (`BeforeAbsent`) | attempt open | the search repeats and writes ABSENT |
| 90 s label and its mark (Δ) | label sent, not marked | the label may be sent twice; notices are idempotent by (message, attempt, label) at the sender |

5.9.g **EXECUTION SEMANTICS:** at-most-once hand-out per attempt (claim before content) is not exactly-once effect:
the frame may reach the agent while the record says nothing; the "outcome unknown" state is the open attempt, reconciled
by the evidence search and the complete point (SI-30); without the point the operator decides (RR-17).
5.9.h **CRYPTOGRAPHY:** the nonce is 128 random bits **[B]**; stage records to the hub are signed by the recorder (SI-6)
**[B]**.
**OBS:** OBS-16 chain lines CLAIMED, HANDED_OVER (with `record_ref`), ABSENT, UNCERTAIN; OBS-17 the
HANDOVER_UNCONFIRMED notice; OBS-18 the stray counter.

### 5.10 AL-10 — The kill switch on every typed path (K-6; SI-23; ARC4 6.9.2, 6.9.7.b; as built plus the typed paths)

5.10.a **SERVES:** SI-23, SI-24, R-63; TH-40, TH-53; AFC-12.
5.10.b **PROBLEM:** delivery off must stop the hook path, the doorbell and the watchdog within one tick, and a switch
that cannot be read must not read as on. **APPROACH:** one read function used at every decision point (`delivery_on`,
3.2.f); the built admin verb and fleet form unchanged.
5.10.c **ALGORITHM:**
```text
delivery_on():
 1  v = store.read_flag("receive.delivery")              # [B] written by `admin delivery on|off --why` (admin socket)
    if Err: return false                                 # Δ: unreadable reads as off                       (FCW-49)
 2  return v == on
read at: AL-9 pull step 1 [B]; AL-6 step 1 (doorbell and watchdog); AL-8 through AL-6.
on off: urgent mail is delivered as normal at the next tool call or turn end (downgraded, recorded), never dropped [B] (FCW-50)
admin delivery off|on --why [--all]: [B] each flip read back and logged where it happened; one admin row; needs-attention (SI-24)
```
5.10.d **INVARIANTS:** (1) after the tick in which the switch reads off, no keystroke and no mid-turn hand-over;
(2) every flip has an admin row and a needs-attention entry **[B]**.
5.10.e **FAILS CLOSED WHEN:**
5.10.e.1 **FCW-49** [AFC-12; ZR-1] the switch cannot be read → off.
5.10.e.2 **FCW-50** [AFC-12; E-12] the switch is off → no keystrokes on either typed path and no hand-over by the hook
(`delivery-off`); urgent mail downgraded, not dropped.
5.10.f **EDGE CASES:** (1) partitioned host: the switch is local (store), so it works without the hub (SI-23); (2) a
flip during AL-6 between step 1 and step 9: at most the one line already past step 1 is typed; the next tick reads off
(the "within one tick" bound); (3) crash: the flag is durable; a restart reads it.
5.10.g **EXECUTION SEMANTICS:** a flip is idempotent; read-back verifies it **[B]**.
5.10.h **CRYPTOGRAPHY:** none; the admin credential is separate from the agent's (SI-14) **[B]**.
**OBS:** OBS-19 admin rows of flips; ring log `held:kill-switch`; pull refusals `delivery-off`.

### 5.11 AL-11 — Main lease claim with prompt origin: provisional, fast takeover, `last_human_at` (K-22; ARC4 6.13.0..6.13.4; B-25)

5.11.a **SERVES:** R-67, R-68; SI-20 (interim, DR-9), SI-24; TH-33, TH-41; AFC-16, AFC-17; answers 4.5.
5.11.b **PROBLEM:** a forgotten session must not keep main through automated prompts, the session the operator uses
must get main fast, and projects whose hooks send no origin must keep working (NID). **APPROACH:** keep the built
lease (a hub claim on offset 0 of `main:<project>`, signed `main.lease.v1` records, generation, operator pin, the
claimer computing over hub primitives **[B]**) and change only the claim decision for projects that report origin.
Human activity is stamped on a separate topic so that a flood of records can never push lease records out of the
newest-1000 retention.
5.11.c **ALGORITHM:**
```text
hooks (Δ, TermLink's own hooks; AEF's adapter T-5054 does the same through `termlink main`):
  SessionStart      -> claim(kind = session-start, origin from the adapter, basis)
  UserPromptSubmit  -> claim(kind = prompt, origin, basis)            # the adapter's report, AL-1 rules
  PostToolUse       -> no lease call (Δ: was claim/renew)             # 6.13.1
  SessionEnd        -> release [B]

records: on main:<p>         Acquire{generation, holder, claim_id, at_ms, reason, from, lapsed_at_ms, obligations,
                                     origin, basis, provisional: bool, holder_human_age_s?}   [B fields + Δ fields]
                             Promote{generation, holder, claim_id, at_ms, origin, basis}               Δ
                             OriginReporting{at_ms, by}   (once per project)                            Δ
                             Release, Lapse, Pin [B]
         on main-touch:<p>   Touch{generation, session, claim_id, at_ms, basis}  (one per human-counted renewal) Δ
         on main-ignored:<p> Ignored{session, origin, basis, hook, at_ms}                                Δ

claim(p, me, origin, basis, kind):
 1  if seam_gate(TERMLINK_MAIN_TEST_*) == refuse: return unknown("test seam set outside a test estate")   # FCW-60
 2  (f, row, v) = snapshot(p); Err -> return unknown(exit 2)                                   # FCW-51 [B]
 3  rep = f.origin_reporting                     # any OriginReporting record, or any retained record with origin field
    if origin is absent and not rep: return built_claim(p, me) with every record marked origin: absent;
                                     outcome adds "this project's hooks do not report prompt origin"   # NID [B]
    if origin is absent and rep: origin = unknown                                              # FCW-53
    if origin is present and not rep: post OriginReporting (failure: continue; the next claim posts it)
 4  s = lease_state(v, f)        # VACANT | PROVISIONAL(h) | HELD(h) | LAPSED(prev) | ELIGIBLE(prev)
                                 # VACANT: no live claim and (never held, or clean release, or a provisional that ran out)
 5  pl = plan(s, me, origin, kind, v.pinned, now = wall())                    # table below
 6  execute pl with the built primitives: claim (compare-and-set on offset 0), renew, force_release, post;
    on Conflict re-snapshot and re-plan, at most 3 times, then "contended" [B]
 7  return outcome(am_i(view after), action)                                  # exit 0 main, 1 not-main, 2 unknown [B]

last_human(h, g):                                # the holder's last human-counted prompt at generation g, wall ms
 1  xs = {a.at_ms : a in Acquire|Promote of h at g with origin == human}
       U {t.at_ms : t in Touch on main-touch:<p> with t.session == h.session and t.generation == g}
 2  return max(xs) if xs else unknown            # read error of main-touch -> unknown                  (FCW-55)
```

| State | kind / origin | Guard | Plan | Generation | Record |
|---|---|---|---|---|---|
| any | prompt, origin ≠ human | — | Ignored | same | Ignored on `main-ignored` (FCW-52) |
| any | session-start | pinned == me | Take "pin" **[B]** | +1 | Acquire |
| VACANT | session-start | — | TakeProvisional (claim TTL 5 min) | +1 | Acquire `provisional: true`, `last_human` none |
| VACANT | prompt, human | pinned ∈ {none, me} | Take "first" or "after-release" | +1 | Acquire `origin: human` |
| PROVISIONAL(me) | prompt, human | — | Promote (renew TTL 15 min) | same | Promote |
| PROVISIONAL(other) | prompt, human | — | force_release, Take "over-provisional" | +1 | Acquire `from` |
| HELD(me) | prompt, human | — | Touch, then renew **[B renew]** | same | Touch |
| HELD(other) | prompt, human | pinned == me | PinOverride **[B]** | +1 | Acquire `reason: pin-override` |
| HELD(other) | prompt, human | pinned == other | Held | same | none |
| HELD(other) | prompt, human | `wall() − last_human(other) > 300 s` (+30 s if `other.host ≠ me.host`) | FastTakeover | +1 | Acquire `reason: fast-takeover, from, holder_human_age_s` |
| HELD(other) | prompt, human | otherwise, or `last_human` unknown | Held | same | none (FCW-54, FCW-55, FCW-56) |
| LAPSED / ELIGIBLE (prev = me) | prompt, human | — | Resume **[B]** | same | Acquire `reason: resume` |
| LAPSED / ELIGIBLE (prev ≠ me) | prompt, human | — | Take "takeover-after-lapse" (no quiet, no cooldown, 6.13.4) | +1 | Acquire |
| PROVISIONAL, HELD, LAPSED, ELIGIBLE | session-start | pinned ≠ me | Answer only | same | none (FCW-57) |

```text
execute Renew (HELD(me)):                       # order matters: stamp first, then extend
 1  post Touch{generation, me.session, claim_id, at_ms = wall(), basis}; Err -> return unknown              # FCW-58
 2  channel_renew(claim_id, ttl = 15 min) [B]; Expired|NotFound -> re-snapshot (3 tries) [B]
execute FastTakeover:
 1  re-read last_human(other) on a fresh snapshot; if now within threshold -> Held                         # race narrowing
 2  force_release(other.claim_id, "fast-takeover by " + me.session) [B primitive]
 3  claim offset 0 (CAS); Conflict -> re-plan [B]
 4  post Acquire; Err -> release the claim, return unknown [B "never hold an unrecorded lease"]
execute Ignored:
 1  post Ignored; Err -> outcome "ignored-unrecorded" (the lease is unchanged either way)                   # FCW-59
```
5.11.d **INVARIANTS:** (1) in an origin-reporting project, a claim with origin other than `human` never changes the
holder, the generation or the claim's expiry; (2) at most one live claim on offset 0 **[B]**; (3) the generation never
decreases and rises on every holder change, including a provisional take **[B]**; (4) a pinned holder is displaced only
by the pinned session's claim **[B]**; (5) every holder change has an Acquire record **[B]**; (6) a project that has never
sent an origin field behaves exactly as built (NID); (7) a renewal of a reporting project's lease is always preceded by a
Touch record (so `last_human` is never older than the holder's last counted renewal).
5.11.e **FAILS CLOSED WHEN:**
5.11.e.1 **FCW-51** [AFC-16; E-20] the lease cannot be read, or a record cannot be posted after a claim → unknown
(exit 2), never main **[B]**.
5.11.e.2 **FCW-52** [AFC-16; E-20, E-21, E-27] origin `automated` or `unknown` in a reporting project → no renewal, no
acquire, no takeover; recorded `ignored: origin`.
5.11.e.3 **FCW-53** [AFC-17; E-22] a claim without an origin field in a project that has reported origin → treated as
`unknown` (ignored).
5.11.e.4 **FCW-54** [AFC-16; E-24] the holder is the pinned session → never fast-taken.
5.11.e.5 **FCW-55** [new; ZR-1] the holder's `last_human` cannot be determined (records unreadable or gone) → no fast
takeover; the claim waits for lapse.
5.11.e.6 **FCW-56** [AFC-16; E-24, ZR-1 for the cross-host margin] the holder's last human prompt is at most 5 min old
(same host), or at most 5 min 30 s old (holder on another host, skew margin V-5) → no fast takeover.
5.11.e.7 **FCW-57** [AFC-16; E-21, E-23] SessionStart on a lease that is not VACANT → no provisional take.
5.11.e.8 **FCW-58** [new; ZR-1] the Touch record cannot be posted → the renewal is not made; unknown (exit 2).
5.11.e.9 **FCW-59** [new; ZR-1] the Ignored record cannot be posted → the lease is unchanged; the outcome says
`ignored-unrecorded`.
5.11.e.10 **FCW-60** [new; ZR-1] a `TERMLINK_MAIN_TEST_*` seam is set outside a test estate → unknown (exit 2) (AL-16).
5.11.f **EDGE CASES:**
5.11.f.1 Forged origin: a same-user process can call `termlink main claim --origin human` (RR-2, AR-6); the receiver
side of AL-1 downgrades a doorbell line (FCW-5), but a hook-independent call is outside its view; recorded in QR-3.
5.11.f.2 Replayed or old records: the fold reads records by offset **[B]**; a Touch of an earlier generation is never
counted for the current one (`last_human` filters by generation).
5.11.f.3 Clock: timestamps are the claimer's `wall()` **[B]**; on one host exact; across hosts the 30 s margin makes a
skew under 30 s unable to cause an early takeover; a larger skew can, and V-5 flags skew over 2 s in status.
5.11.f.4 Concurrency: (a) two sessions claim a vacant lease at once: the claim CAS lets one win, the other re-plans and
reads HELD **[B]**; (b) the holder's human prompt and another session's fast takeover race: if the taker's fresh
re-read (FastTakeover step 1) misses the holder's Touch, the taker wins; the holder's renew then fails, it re-plans and
reads HELD(taker) with a fresh `last_human`, and is told it lost main; the record names the takeover. Bounded by one
RPC round trip; QR-4. (c) a provisional holder and a human prompt elsewhere: the human claim force-releases and takes;
the provisional holder's own later prompt reads HELD(other).
5.11.f.5 Crash table:

| Crash between | State after restart | Recovery |
|---|---|---|
| Touch and renew | Touch recorded, claim not extended | the claim may lapse at its expiry; the session's next human prompt resumes it (same generation) |
| force_release and claim (fast takeover, pin override) | no live claim, current = old holder | a lapse-like vacancy **[B]**: the next human prompt anywhere takes (or the old holder resumes); the hub's claim log shows the force-release |
| claim and Acquire post | live claim, no record | built self-heal: the holder's next renew posts `record-repair` **[B]**; until then `consistent: false` in status |
| OriginReporting post fails | project still non-reporting | the next claim with an origin posts it; absent claims count until then (NID direction) |

5.11.g **EXECUTION SEMANTICS:** each plan is a hub claim operation followed by a record; "never hold an unrecorded
lease" **[B]** is the target-side rule; the claim CAS is the idempotency for concurrent takers. A fast takeover's
effect (S1 loses main) is visible to S1 at its next claim and in `termlink main status`.
5.11.h **CRYPTOGRAPHY:** `main.lease.v1` records are signed **[B]** (the claimer's identity key as built); Touch,
Ignored, Promote and OriginReporting use the same signing as the built records (Δ: same message type family,
`main.lease.v1` with a `kind` field). No new algorithm.
**OBS:** OBS-20 `termlink main status --json` (holder, generation, provisional, `last_human_at`, `origin_reporting`,
recent records including Ignored with basis); OBS-21 the outcome JSON of every claim (action, verdict, reason).

### 5.12 AL-12 — The sidecar's lease read and the non-main withhold (K-22; LEASE; as built plus Δ)

5.12.a **SERVES:** R-67.a1 (minimum), R-62; SI-20 interim; AFC-16, AFC-19.
5.12.b **PROBLEM:** the sidecar must know who main is without writing the lease. **APPROACH:** as built: read every
15 s on its own thread, trust a read for 4 periods, withhold mail from a non-main session of the same agent, fail open
on anything unknown **[B]**; Δ: a provisional holder is a holder, and the view exposes `last_human_at` and the state
for AL-13 and AL-14.
5.12.c **ALGORITHM:**
```text
reader thread [B]: every 15 s: r = LeaseClient.status(project); shared.set(mono(), r)       # Ok(view) | Err(why)
view(now): last = shared.last
   if last is none or Err or now - last.at > 4 x 15 s: return UNKNOWN(why)                  # FCW-61 [B]
   return last.view      # Δ: with state, provisional, last_human_at of the holder (AL-11 last_human)
withhold(agent, session, now) [B]:
   v = view(now); if UNKNOWN or no holder: return none                  # fail open: deliver as before
   if v.holder.session == session or v.holder.agent != agent: return none
   return line("this session is not main of P; peer mail is left unread for main S (generation N)"), once per (session, generation)
would_hand_over(m, sess) = (withhold(MY_AGENT, sess, now) is none)
```
5.12.d **INVARIANTS:** (1) the sidecar never writes the lease **[B]**; (2) an unknown lease never stops delivery **[B]**.
5.12.e **FAILS CLOSED WHEN:** 5.12.e.1 **FCW-61** [AFC-16, AFC-19; E-34 (f)] the last read failed or is older than
4 periods (60 s) → unknown: no withhold (delivery as before), no forward (AL-14), status `main.lease.state: unknown`
**[B]**. (This is "fail open" for delivery and "fail closed" for every lease-based decision.)
5.12.f **EDGE CASES:** (1) the withhold trusts a 60 s read while forward-once needs 15 s (4.6): the two directions
differ on purpose (withholding keeps mail where it is; forwarding moves it); (2) crash: the view is in memory; a
restart reads within 15 s; until then unknown.
5.12.g **EXECUTION SEMANTICS:** read-only. 5.12.h **CRYPTOGRAPHY:** the records' signatures are checked by the fold
**[B]**.
**OBS:** OBS-22 status `main.lease` with `checked_at_ms`.

### 5.13 AL-13 — "Mail waiting, main not reading" (K-34; ARC4 6.13.5, 17.7; B-25, B-28)

5.13.a **SERVES:** R-67, R-53, R-29, R-37; FX-8; AFC-18.
5.13.b **PROBLEM:** role mail can sit unread because nobody is a reading main; the operator must learn it, and an
unavailable channel must never read as "sent". **APPROACH:** one definition, computed per tick by whichever receiver
holds the role mail (TermLink sidecar here, AEF's receiver for its inboxes, the same function by contract 17.7);
landing places escalate with age; the out-of-band notice is rate-limited per project through a hub claim.
5.13.c **ALGORITHM:**
```text
mail_waiting_tick():                                       # once per tick
 1  W = [m in store: m.to is a role (main or the project), not an exact instance; STORED; not HANDED_OVER;
         not FORWARDED; not REFUSED]                                                              # FCW-64
 2  old10 = [m in W : mono() - m.stored_mono >= 10 min]                                            # V-21
 3  L = lease view (AL-12)
    reading = L.state == HELD and not L.provisional and L.last_human_at known
              and wall() - L.last_human_at <= 10 min
    # vacant, lapsed, provisional, unknown, or a holder silent for 10 min -> not reading       # FCW-65
 4  if old10 nonempty and not reading and not active:
        active = true; since = mono(); attention("main-mail-waiting", coalesce = episode)
 5  if active and old10 empty: active = false                     # clears only when the old mail is handed over
 6  status.main.mail_waiting = active ? {count: len(W), oldest_age_s, holder, holder_human_age_s, since, oob} : null
 7  oob step: if active and oldest_age >= 30 min:
        if no out-of-band channel is configured (GP-11 not live): oob = "not available"                      # FCW-62
        elif hub.claim("oob:" + project, offset 0, ttl = 60 min) == Conflict: oob = "sent-by-other within 1 h"  # FCW-63
        else: r = channel.send(project, "MAIL WAITING ...");
              oob = (r has the channel's own delivery receipt) ? "sent at T" : "not confirmed"
session-start line: scripts/session-start-alerts.sh reads status.main.mail_waiting and prints one line [B-script, Δ field]
non-main line: AL-9 withhold path, when active and the prompt's origin is human: one line per (session, generation)
```
5.13.d **INVARIANTS:** (1) the status never says "sent" without the channel's receipt; (2) at most one out-of-band
notice per project per hour, across every receiver of the project; (3) the weekly count of episodes is reported, never
capped (RNU).
5.13.e **FAILS CLOSED WHEN:**
5.13.e.1 **FCW-62** [AFC-18; E-28] no out-of-band channel exists, or it gives no delivery receipt → status `oob: not
available` or `not confirmed`; never "sent".
5.13.e.2 **FCW-63** [AFC-18; E-28, ZR-1] another receiver of the project holds the per-project out-of-band claim (a
notice within 60 min) → no second notice.
5.13.e.3 **FCW-64** [AFC-18; E-28 control] mail to an exact instance → never counted.
5.13.e.4 **FCW-65** [AFC-18; E-28] the lease is unknown, vacant, lapsed or provisional, or the holder's last human
prompt is unknown or older than 10 min → "no reading main" (the signal can rise).
5.13.f **EDGE CASES:** (1) a reading main that does not take the mail is not this signal's case; R-44.e's
OVERDUE-HANDOVER and V-8 cover it; (2) clock: mail ages on `mono()`, the holder's age on the lease's `wall()` (AL-11);
(3) concurrency: two receivers of one project raise the signal independently; the OOB claim makes the notice single;
(4) crash: `active` and `since` are recomputed from durable mail ages at the next tick; a restart may re-raise the
needs-attention entry once (coalesced per episode by key `(project, oldest message id)`).
5.13.g **EXECUTION SEMANTICS:** the OOB notice is at-most-once per hour per project by the claim; the claim taken and
the send failing gives `not confirmed` and no retry within the hour (the status shows it, the next hour retries).
5.13.h **CRYPTOGRAPHY:** none beyond the hub claim's authentication **[B]**.
**OBS:** OBS-23 status `main.mail_waiting`; OBS-24 needs-attention `main-mail-waiting`; OBS-25 the `oob:<project>`
claim history and the channel receipt.

### 5.14 AL-14 — Forward-once of role mail at the receiving copy Y (K-23; ARC4 7.1, 7.2, 7.4; B-20; DR-10)

5.14.a **SERVES:** R-67.a1 (4), R-51, R-62, R-56; SI-3, SI-4, SI-30; TH-33; AFC-19, AFC-26.
5.14.b **PROBLEM:** role mail that reaches agent Y while main is a session of agent X of the same project must reach X
once, keep its original signature and tell the sender. **APPROACH:** Y stores it (its promise), records a durable
forward intent, posts a FORWARD envelope signed by Y that carries the original bytes, marks its copy final, and sends a
VIA notice; every step is idempotent by the envelope identity `(Y, "forward", m.id)`.
5.14.c **ALGORITHM:**
```text
after_stored(m) at Y:                                      # called after AL-23 makes m STORED
 1  if m.to is an exact instance, or m is a FORWARD envelope, or m.to is not a role of Y's project: return   # FCW-66
 2  if m.hop >= 3: return                                   # hop limit (V-4); deliver as before             (FCW-69)
 3  L = lease view; if not (L.ok and L.read_rx_mono >= m.stored_mono and mono() - L.read_rx_mono <= 15000):
        L = LeaseClient.status(project) synchronously, timeout 10 s; Err -> status_note(m, "no forward: lease unknown"); return  # FCW-67
 4  if L.holder is none (vacant, lapsed): return            # deliver as before, status says why                 (FCW-68)
    X = L.holder.agent
    if X == MY_AGENT: return                                # same agent: the withhold of AL-12 applies
    if X not in roster(project): attention("main outside roster"); return                                     (FCW-68)
 5  with store.lock: if forward_intent(m) exists: return
                    chain_append(FORWARD_INTENT m.id, generation = L.generation, to = X)                       # FCW-70
 6  env = {schema: "termlink.forward/1", project, generation: L.generation, hop: m.hop + 1,
           via: Y.instance, inner_topic: m.posted_topic, inner_sha256: sha256(m.raw), inner_b64: base64(m.raw)}
    sig = sign(Y.key, "termlink.forward/1\n" + sha256_hex(jcs(env)))
 7  post(inbox:X on the home hub, {env, sig}, idempotency id = (Y, "forward", m.id)); Err -> retry next tick
    if not posted within the R-44.e "not accepted" bound (1 min): chain_append(FORWARD_ABANDONED m.id);
        attention("forward failed"); return                # m is offered at Y again, as before            (FCW-71)
 8  chain_append(FORWARDED m.id, to X, generation); hub.stage(m, FORWARDED, {via: Y, main: X, generation})
 9  notice(m.sender, VIA, {msg: m.id, via: Y, main: X, generation})     # "delivered via Y; main is X (generation N)"
while FORWARD_INTENT(m) without FORWARD_ABANDONED: m is not offerable at Y (pull, ring, watchdog)          # FCW-71
```
5.14.d **INVARIANTS:** (1) the inner bytes posted equal the bytes Y received; (2) a message is forwarded at most once by
Y (one intent per message); (3) a forwarded message is never handed over at Y; (4) never forwarded: exact-instance mail,
a FORWARD, mail with hop ≥ 3, mail on an unknown, stale or vacant lease.
5.14.e **FAILS CLOSED WHEN:**
5.14.e.1 **FCW-66** [AFC-19; E-34 (d), (e)] the mail is to an exact instance, is itself a FORWARD, or is not role mail
of Y's project → no forward.
5.14.e.2 **FCW-67** [AFC-19; E-34 (f)] no lease read completed after STORED within 15 s, and the synchronous read fails
→ no forward; delivered at Y as before; status says why.
5.14.e.3 **FCW-68** [AFC-19; E-34 (f)] the lease is vacant or lapsed, names this agent, or names an agent outside the
project's roster → no forward.
5.14.e.4 **FCW-69** [AFC-26; E-53, E-34] the message's hop is already 3 → no forward.
5.14.e.5 **FCW-70** [new; ZR-1] the forward intent cannot be written durably → no forward; delivered at Y as before.
5.14.e.6 **FCW-71** [new; E-35, ZR-1] while a forward intent is open, the message is never offered at Y; if the
envelope cannot be posted within 1 min the forward is abandoned (recorded, needs-attention) and the message is offered
at Y as before.
5.14.f **EDGE CASES:**
5.14.f.1 Forged or altered: X verifies both signatures (AL-15); Y cannot change the inner bytes without breaking the
inner signature.
5.14.f.2 Replayed: a re-posted envelope (retry, crash) has the same idempotency id; X dedupes the inner message by its
identity triple (R-51) **[B]**.
5.14.f.3 Clock: the 15 s bound on Y's `mono()` (4.6).
5.14.f.4 Canonical form: the envelope is canonicalised by RFC 8785 under 3.3's integer rule; test vector V-FWD-1 in
06a 2.4.
5.14.f.5 Concurrency: the lease moves between Y's read and X's pull: X answers FORWARD-STALE (AL-15); two forwards of
one message cannot happen (one intent under the store lock).
5.14.f.6 Crash table:

| Crash between | State after restart | Recovery |
|---|---|---|
| STORED and intent | STORED, no intent | `after_stored` runs again for stored role mail without intent or hand-over (start-up scan) |
| intent and post | intent, nothing posted | re-post (same idempotency id) at the next tick |
| post and FORWARDED | posted, intent open | re-post; X dedupes by the inner identity triple; then FORWARDED |
| FORWARDED and VIA | final at Y, no VIA | start-up scan: FORWARDED without a VIA line sends VIA (notices idempotent by (message, kind)) |

5.14.g **EXECUTION SEMANTICS:** at-least-once posting with receiver-side idempotency (the identity triple) gives one
hand-over (H-24 forward half). The target can commit (X stores) while Y's record says intent only: the start-up scan
re-posts and X dedupes. Target binding before dispatch: X is the holder of a lease read within 15 s, in the roster.
5.14.h **CRYPTOGRAPHY:** the forwarder signs with its ed25519 agent key **[B key]** over the domain-separated digest
`termlink.forward/1\n<sha256(jcs(envelope))>` (ARC4 6.2.1's form, applied to the envelope; DR-10). No new key or
algorithm.
**OBS:** OBS-26 chain lines FORWARD_INTENT, FORWARDED, FORWARD_ABANDONED; OBS-27 hub stage FORWARDED; OBS-28 the VIA
notice in the sender's record.

### 5.15 AL-15 — Accepting a FORWARD at X (K-23; ARC4 7.3; B-20)

5.15.a **SERVES:** R-67.a1 (4), R-67.e, R-51; SI-3, SI-4, SI-11, SI-19; TH-33; AFC-19, AFC-22, AFC-25, AFC-26.
5.15.b **PROBLEM:** X must accept only a genuine forward of genuine role mail of its own project, and only while X is
still main at the forward's generation. **APPROACH:** verify outer, then inner, then the generation against X's own
lease read; on mismatch store, do not forward, tell the sender to re-resolve.
5.15.c **ALGORITHM:**
```text
on_forward(env, sig, poster) at X:
 1  strict_json(env); require env.schema == "termlink.forward/1"; require the decoded inner message within the
    inline cap (PN-7, 8 KiB; a blob travels as its reference, AL-26) and the envelope within 16 KiB -> else refuse  # FCW-72
 2  Ykey = key card / roster entry of poster; require poster in roster(X.project) and env.project == X.project
    require verify(Ykey, "termlink.forward/1\n" + sha256_hex(jcs(env)), sig)                               # FCW-72
 3  inner = base64_decode(env.inner_b64); require sha256(inner) == env.inner_sha256
    m = parse(inner); require verify_message(m, topic = env.inner_topic)            # the original signature, SI-3
    require env.inner_topic is the inbox of an agent of X.project, and m.to is a role of X.project          # FCW-73
 4  require m is not a FORWARD and env.hop == m.hop + 1 and env.hop <= 3                                   # FCW-74
 5  run AL-23 checks on m as if received directly: revocation of m's key, sequence window, quota of m's sender  # FCW-76
 6  L = lease view at X with the 15 s rule of AL-14 step 3 (sync read on miss)
    if L unknown: store env (STALE); notice(m.sender, FORWARD-STALE, {msg: m.id, reason: lease-unknown}); return   # FCW-75
    if L.holder.agent != MY_AGENT or L.generation != env.generation:
        store env (STALE); notice(m.sender, FORWARD-STALE, {msg: m.id, generation_seen: L.generation}); return     # FCW-75
 7  process m as received (identity triple dedupe R-51 [B]); mark env consumed
```
5.15.d **INVARIANTS:** (1) X hands over only messages whose inner signature verifies against the original topic;
(2) a FORWARD whose generation is not X's current one is never handed over at X; (3) X never forwards a FORWARD.
5.15.e **FAILS CLOSED WHEN:**
5.15.e.1 **FCW-72** [AFC-19; E-34 (a)] the envelope is malformed, too large, signed by a key that is not a roster member
of X's project, or its outer signature fails → refused, recorded, signed refusal notice to the poster.
5.15.e.2 **FCW-73** [AFC-19; E-34 (b), (c)] the inner bytes do not match their digest, the inner signature fails
against the inner topic, or the inner topic or address is not a role of X's project → refused.
5.15.e.3 **FCW-74** [AFC-19, AFC-26; E-34 (d)] the inner message is itself a FORWARD, or the hop does not add up, or
exceeds 3 → refused.
5.15.e.4 **FCW-75** [AFC-19; E-33] X's lease is unknown, names another agent, or has another generation → stored as
stale, not handed over, FORWARD-STALE to the original sender.
5.15.e.5 **FCW-76** [AFC-22, AFC-25; E-39, E-50] the original sender's key is revoked, its sequence number is refused, or
its quota at X is full → refused or held exactly as a direct message would be.
5.15.f **EDGE CASES:** (1) replay of an old envelope at a later generation: generation mismatch, FORWARD-STALE; the
sender's ladder re-resolves and re-sends the same id, which the new main dedupes; (2) a FORWARD-STALE loop is bounded:
Y forwards only on a fresh lease and the sender re-resolves through the hub; each loop step is a visible notice;
(3) clock: X's 15 s rule on its own `mono()`; (4) concurrency: two copies of the envelope (Y retried): the second dedupes
by identity triple; (5) crash: between store and the STALE notice: start-up scan of STALE envelopes without a notice sends
it.
5.15.g **EXECUTION SEMANTICS:** acceptance is idempotent by the identity triple; FORWARD-STALE is idempotent by (message,
generation).
5.15.h **CRYPTOGRAPHY:** two ed25519 verifications (outer, inner) with keys from the roster / key card (AL-33); the
domain prefix `termlink.forward/1` separates envelope signatures from message signatures.
**OBS:** OBS-29 refusal records with the failed check; OBS-30 FORWARD-STALE notices; OBS-31 hand-over of the inner
message (AL-9).

### 5.16 AL-16 — The test-seam gate (EV5 14.3.g; answer 4.8)

5.16.a **SERVES:** R-6, R-39; SI-27, SI-25; FP 3.1.b (floor probes stay honest); EV5 3.2.b.
5.16.b **PROBLEM:** test seams (shortened timers, frozen components, simulated clocks, shrunk ceilings) must never be
on in an installed system. **APPROACH:** one gate function; production refuses loudly instead of ignoring silently, so
a seam left in a unit file is found at start, not by a later surprise.
5.16.c **ALGORITHM:**
```text
SEAMS = the table of 4.8 (names and prefixes TERMLINK_SIDECAR_TEST_, TERMLINK_MAIN_TEST_, TERMLINK_HUB_TEST_)
seam(name, mode):                         # the only way any routine reads a seam
 1  if mode is Estate(root): return env(name)           # honoured
 2  return none                                         # production: never honoured
startup_check(mode, process):             # sidecar start, hub start, every `termlink main` verb
 1  if mode is Estate: return ok
 2  set = [v in environ if v starts with any SEAMS prefix]
 3  if set nonempty:
       sidecar, hub: refuse to start, exit 2, reason "test seam <v> set outside a test estate"          # FCW-77
       termlink main: answer unknown (exit 2) with that reason                                        # FCW-60
mode for `termlink main` (Δ): Estate when HOME and the runtime directory lie in a marked estate (estate::guard_any) [B guard]
```
5.16.d **INVARIANTS:** (1) no seam value reaches any decision in production; (2) a production process with a seam
variable set does not run its function.
5.16.e **FAILS CLOSED WHEN:** 5.16.e.1 **FCW-77** [new; ZR-1] a seam variable is set in a production-mode sidecar or
hub → it refuses to start. (`termlink main`: FCW-60.) 5.16.e.2 **FCW-78** [new; ZR-1] a hub started with
`TERMLINK_HUB_TEST_CEILINGS` outside an estate runtime directory → refuses to start.
5.16.f **EDGE CASES:** (1) NID: refusing to start is a change for a host that has a seam set today; the production
scan (4.8.b) is run once before B-25 ships, and any hit is fixed first, so the change switches off nothing that works;
(2) crash: none (stateless).
5.16.g **EXECUTION SEMANTICS:** decided at start. 5.16.h **CRYPTOGRAPHY:** none.
**OBS:** OBS-32 the refusal reason in the unit journal and status `mode`; OBS-33 the production scan's result.

