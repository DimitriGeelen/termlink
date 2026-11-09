# Interactive agent communication: step 5, evidence specification

**Task:** T-3354 · **Arc:** arc-011 · **Chain:** arc-011-design, step 5 · **Role:** evidence specialist
**Status:** DRAFT v0.1, batch mode (no human present, `_common.md` 5.2). This step does not decide (`decides: false`);
it iterates with step 6 (pseudo-coder). Nothing here is approved. Every threshold that is not a ruled number is
tagged **[P]**; ruled numbers carry their source.
**Standing rules applied throughout (CLAUDE.md):** thresholds measure reliability, never usage (**RNU**: reach per
attempt, every failure visible, zero false "delivered"; usage is reported as a cost); nothing working is switched off
as a side effect (**NID**); "nothing is called working without a live test with two real agents and a negative
control" (profile P1.2.e) as refined by the four CAND-3 tiers (R-69.a).
**Review link:** none. This project's Watchtower has no inline design-review page (profile P2.1); the operator and
the reviewers read this file in the repository.
**Sub-documents** (each reviewable on its own):
1. this file, the evidence specification;
2. `docs/design/interactive-agent-communication-05a-coverage.md`, the coverage tables (card 4.2): every R-n, SI-n,
   FX-n, "impossible" requirement, architecture fail-closed condition and measured property to the specs E-n, gaps
   marked. It is generated from this file's spec headers by the script named in its section 1, so it cannot drift
   from the specs silently.

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-11-09 | First complete draft, batch mode. Test estate and agent pairs per CAND-3 tier; evidence levels; result format; specs E-1 to E-148 (PH-2 first, then baseline re-assertion, later phases, production invariant checks); architecture fail-closed conditions AFC-1 to AFC-24 with specs; pseudo-code FAILS CLOSED WHEN coverage open for step 6; drawings D-1 and D-2; residual risks; change requests. | T-3354 |

## 1 Inputs of record

1.1 Full sha256 of the bytes read on 2026-11-09 (`sha256sum`), at commit `1c71223fd`.

| Short | Path | Version | sha256 |
|---|---|---|---|
| ARC4 | `docs/design/interactive-agent-communication-04-architecture.md` | v0.3.1, approved by the operator 2026-11-08 (CHAIN step 4 `approval:`) | `29c6d1f7de9f18898cab105651f17977ebf97da30f32d16501819bad217dd69c` |
| TRACE | `docs/design/interactive-agent-communication-04a-traceability.md` | v0.3 (part of ARC4) | `c516a290997e0c75957c0be95af4d611e58155cd818d2fbae961c0f4ad6d4667` |
| REQ | `docs/design/interactive-agent-communication-01-requirements.md` | v0.5.3 (XR-2, XR-9 folded, T-3676) | `f6ce42a2984bd18e049f1f4dda05e6eaab2cc3508396fe373060c6fc527193c9` |
| TM | `docs/design/interactive-agent-communication-02-threat-model.md` | v0.3.7, residual risks accepted 2026-10-11 | `88961693384966502f4ca4eb515ed4b0590a29c3d61ac0c619e17ea4562693f3` |
| FP | `docs/design/interactive-agent-communication-03-floor-and-phasing.md` | v0.3 (XR-1, XR-3..XR-8 folded, T-3676) | `e4de2e3e102ee59f6434db584e099e5bb180c6b33dfd823ae217e6ebb4e5a8c4` |

1.2 Built and live facts, read as baseline (evidence is specified for them; they are not redesigned).

| Short | Path | sha256 |
|---|---|---|
| ARCH2 | `docs/design/arc-011-ph2-architecture.md` | `b8a941e5563b72707bb3abbe89677ba42ab5560dc1882721b59f59390d84d589` |
| LEASE | `docs/operations/main-lease.md` | `c95c12e76e0e3b8166badf4919e262916210c89b427ac2d4be996cda65c51e9a` |
| CODE | `crates/termlink-sidecar/src/` (notably `testkit.rs`, `beat.rs`, `lease.rs`, `activity.rs`), `crates/termlink-sidecar/tests/` (incl. `mutants/fc-01..fc-19`), `scripts/arc011-mutants.sh`, `scripts/arc011-ph1-demo.sh`, `scripts/session-message-selftest.sh`, `tests/runme-fixtures.sh` at `1c71223fd` | not hashed; cited by path |

1.2.a LEASE changed since step 4 read it (`f4d79119…`): commit `07da39022` (T-3668, `termlink main` reads
`CLAUDE_CODE_SESSION_ID` and never the TermLink session id inside Claude Code). It changes no lease rule; E-28 and
E-31 use the corrected session identity.

1.3 Role inputs, in the order given (README 3.3).

| Short | Path | sha256 |
|---|---|---|
| RULES | `CLAUDE.md` (project sections above `## Core Principle`) | `a9b38fd1aeed0145289c6a8c89c491c2ae077d06e1847110209827f33e172aa3` |
| COMMON | `docs/design/roles/cards/_common.md` | `cca38806358d7a6c09e64289164977a825f2cf57e914f672076ff77a2bdb5563` |
| CARD | `docs/design/roles/cards/evidence-specialist.md` | `b829e633f8119e5870b335e440b899784708878b4306523c6a69a60fde9a9c4e` |
| ADAPTER | `docs/design/roles/adapters/aef.md` | `d840a77649888cdbfd37d1166c15eecd6cd3c8399d497fb6b1b2c52455c6c4a4` |
| PROFILE | `docs/design/roles/profiles/termlink.md` | `b6bc160242d96ca9a8fc167c6845e292529c162fa23faa2945d4742b676e7b21` |
| README | `docs/design/roles/README.md` | `3881d5302313a0d0ca4c9ef1b4688dae101b6d693df823be962e9b70946368e0` |
| CHAIN | `docs/design/interactive-agent-communication-role-chain.yaml` | `6ed4d708b8364428a5f9f2996519e8956be02b31e5a10817b55df29be99b8752` |
| SCHEMA | `docs/design/roles/schemas/role-handback-1.schema.json` | `288a4e312f00bac978a4b2a87e57ee8a05cb14f916d77c397e601e03c703c148` |

1.4 How this step was run.
1.4.a Batch mode. Every question I could not answer from an input is an open item (section 15) or a change request
(section 17), never a silent assumption.
1.4.b Where the adapter names ring20 paths this project lacks (`role-chain.py`, `design-render-check.py`,
`role-handback-check.py`), the profile governs (P2.1, P4.5). Drawings were checked with `mmdc` and the local
Chromium, the method of steps 2 to 4 (section 18).
1.4.c I changed no earlier output and no code. The hand-back is
`docs/design/interactive-agent-communication/role-handback-step-05.json`, next to the step-1 and step-4 records.
1.4.d The pseudo-code (step 6) does not exist yet. Its FAILS CLOSED WHEN conditions are therefore open; section 13
lists the fail-closed conditions the architecture itself states (AFC-n), each with a spec, and leaves the
pseudo-code's own list for the step-6 iteration.

## 2 How to read this document

2.1 Identifiers new here: `E-n` evidence specs, `AFC-n` the architecture's own fail-closed conditions (section 13),
`FCW-n` the pseudo-code's FAILS CLOSED WHEN conditions (step 6, open), `L1`..`L7` evidence levels (section 4),
`EP-n` the named estate pairs (section 3), `ER-n` residual risks of this step, `YR-n` change requests. Ids of earlier
steps are cited as defined there: R-n (REQ), TH-n, SI-n, RR-n, BP-n, TB-n (TM), FX-n, H-n, PH-n, T-n dimensions, PN-n
(FP), K-n, B-n, V-n, DR-n, AR-n (ARC4), W-n waves and OQ-x (ARCH2).
2.2 **Name clash, resolved by prefix.** The PH-1 architecture (`arc-011-ph1-architecture.md`) and `testkit.rs` use
`E-1`..`E-5` for estate rules and `FC-1`..`FC-19` for its fail-closed rules (the mutants `fc-01`..`fc-19`). Here those
are always cited as **ARCH E-n** and **ARCH FC-n**; a bare `E-n` is this document's.
2.3 Each spec has one header line and five numbered parts:
2.3.a header: `**E-n — title.** Links: … Category: … Level: … Tier: … Phase: … (build item)`;
2.3.b `E-n.1 EVIDENCE` the claim the spec proves ("suppose it worked perfectly, how would I tell", card 3.1);
2.3.c `E-n.2 ARRANGE`, `E-n.3 ACT`, `E-n.4 ASSERT` (the observable, mechanically checkable result);
2.3.d `E-n.5 CONTROL` the negative control or mutant that MUST turn the spec red (R-69, kill-checked), or, for a
limit probe, the statement that its expected result is `LIMIT`.
2.4 Categories (card 3.3): **user workflow**, **core function**, **edge case**, **error handling**, **integration**,
**adversarial**. One behaviour per spec, except where the behaviour is a sequence (a state machine run, a race).
2.5 RFC 2119 words are normative for the build items that implement a spec. Which runner, language or tool implements
a spec is decided in the build (card 5); where a spec names an existing harness it names the estate, not a mandate.
2.6 Reading order for the build: section 6 (PH-2 next items) first, in the order of the operator's priority (brief):
doorbell D+ and watchdog, lease tightening, forward-once, revocation, blobs and keys, caps, status, fault injectors,
hub changes, W3 acceptance; then section 7 (PH-2 rows with tasks), 8 (baseline re-assertion), 9 (later phases), 10
(production invariant checks).

## 3 The test estate and agent pairs per CAND-3 tier (R-69.a; T-3354 agent criterion)

3.1 **The rule (R-69.a, FP 8.1).** Verification is selected per failure mode in four tiers; negative controls are
kill-checked; each dimension is required once its subject exists, never earlier by a stand-in. A tier proves only
what its estate contains: hermetic tests are evidence for logic, live runs for wiring, and neither alone proves the
other (card 3.4).

3.2 **Tier 1, fixture.**
3.2.a Estate: `termlink_sidecar::testkit::Estate::create_without_hub()` (a marked throwaway tree with its own HOME,
runtime directory, keys and an exported canary secret; the estate guard refuses unmarked paths, ARCH E-1..E-3);
the bash fixture suites `tests/*fixtures*.sh` with their `RUNME_*` and `TERMLINK_*_TEST_*` seams; `tests/test-*.py`.
3.2.b Clock: simulated. Existing seams: `TERMLINK_SIDECAR_TEST_HEARTBEAT_MS`, `_STALE_MS`, `_FREEZE` (`beat.rs`),
`TERMLINK_SIDECAR_TEST_LEASE_EVERY_MS` (`lease.rs`), `TERMLINK_SIDECAR_TEST_LIVENESS_EVERY_MS` (`liveness.rs`),
`TERMLINK_MAIN_TEST_LEASE_MS`, `_QUIET_MS`, `_COOLDOWN_MS` (LEASE). New seams the PH-2 items need **[P]**: a
simulated tick clock for the ring decision and the watchdog (B-16, B-27), a fake session that reports per-source PTY
input counters and a foreground process name (B-27), a fake adapter emitting `termlink.adapter/2` JSON (B-29), and a
clock for the lease's `last_human_at` (B-25). Every seam is honoured only in a marked test estate and refused in
production mode (as `beat.rs` today; ARCH2 6.8.1 (4)).
3.2.c Agents: none. A "peer" is a second sidecar identity in the same estate (EP-1: `fx-a`, `fx-b`; EP-2 adds
`fx-c` for three-party cases), signing with estate keys.
3.2.d Proves: decision logic, state machines, parsers, canonical forms, refusal paths, the order of checks. Does not
prove: anything about a real hub, a real PTY, a real harness or a real disk.

3.3 **Tier 2, live hub.**
3.3.a Estate: `Estate::create()` (the same tree plus a throwaway `termlink hub` started from the built binary, with
only the estate's environment); for two-hub cases two estates whose hubs know each other only through the test
(EP-3: hubs `fx-hub-1`, `fx-hub-2`).
3.3.b Agents: none. Scripted sidecars (EP-1, EP-2) against the real hub; real TermLink PTY sessions spawned in the
estate where a spec needs a PTY (a plain shell or a scripted fake harness as the foreground process, EP-4).
3.3.c Proves: hub RPCs, signatures and pins over TLS and sockets, artifact store contents, consumer lists, admin
log rows, PTY input counters by source. Does not prove: harness behaviour or the installed scheduler.

3.4 **Tier 3, one real agent plus a scripted peer, through the installed scheduler and hooks, per harness.**
3.4.a Estate: the `testdev` account on .107 (OQ-i A ruled 2026-10-13): its own login, the **installed release
artifact**, production-mode sidecars under a testdev-owned root, the real hub on .107, systemd units and the cron
backstop as installed. Faults are injected only through the tier-3 forms of 6.8.1 (SIGSTOP of the whole sidecar,
config pointing at a throwaway hub, hooks removed), because production mode refuses seams.
3.4.b Pair **EP-5**: real agent `testdev-real` (Claude Code, interactive, under a TermLink PTY started by the
reachable launcher, TermLink's W1 hooks) and scripted peer `testdev-peer` (a production-mode sidecar driven by a
script, no harness). Per further harness (T-19): **EP-6** the same with 055's opencode adapter, from the release that
first includes it.
3.4.c Proves: hook wiring, the doorbell reaching a real prompt, transcript evidence written by the real harness,
the installed scheduler, the prompt-origin report of a real adapter. Does not prove: two real agents conversing.
3.4.d Status: `testdev` canary pair is T-3361 (B-19), not built. Until EP-5 exists, tier-3 specs are **not run**
and are listed as not run in each phase report; no stand-in counts (FP 8.1).

3.5 **Tier 4, real agents.**
3.5.a **EP-7, the W1/W2 pair** (ARC4 4.5): `claude-termlink` and `claude-termlink-alt`, this project's agents on
.107, hook delivery since 2026-10-14; the doorbell (W2) and the H-11 harness run here (B-17). The main session
joined W1 on 2026-10-18 and is the third real party for lease cases (EP-7b).
3.5.b **EP-8, the canary pair**: `testdev-canary-a` and `testdev-canary-b` (OQ-i A; B-19), daily on the installed
artifact, rotated through real pairs once the doorbell exists (OQ-i open item).
3.5.c **EP-9, W3 population**: every agent of 010 on .107 through the reachable launcher (B-21); P24's two-agent
acceptance with a negative control and the H-14 soak run on real traffic here (B-22).
3.5.d **EP-10, three-agent many to many (T-4c, PH-4)**: EP-7 plus a third real agent of 010. **EP-11, cross-host
(T-5, PH-3)**: one 010 agent on .107 and one on a second host with its own hub (.141 is a candidate as it serves no
other project's receiver). Agents of other projects (ring20, 055, 832, Penelope) are used only with their owner's
consent through `framework:pickup`; none is a test fixture by default.
3.5.e Proves: round trips with transcript evidence, the five stage rows with times, "no action", the operator's
real terminal conditions (typing, attended idle sessions). Does not prove: anything outside the population and the
days observed.

3.6 **Dimensions that are not tiers** (FP 8, T-6..T-21) run on the tier their subject needs: process crash and kill
points (tier 1 and 2), host crash (a virtual machine on a real disk image, EP-12: a disposable VM on .107 with the
installed artifact), reboot (EP-5 host), clean host (`tests/arc011-clean-host.sh`), second OS (the macOS CI job,
T-3271), mixed version (EP-3 with two binaries), restore from backup (tier 2 with a snapshot of the estate), account
isolation (Track A pilot, PH-3).

3.7 **Who runs what.** Tier 1 and 2 run per push in the guard layer (`scripts/run-guard-layer.sh`) and per release.
Tier 3 runs per release and after a rotation, a re-vendor and a reboot (R-66.a). Tier 4 runs as operator runme
actions (waves, drills) and as the daily canary. The agent never starts a wave or a drill itself (ARCH2 3).

## 4 Evidence levels (card 3.4)

| Level | Name | Typical tier | What it can show | What it cannot show |
|---|---|---|---|---|
| L1 | property-based or formal (model check of a state machine, property over generated traces) | 1 | an invariant over every generated case of a model | that the code is the model; cases outside the generator |
| L2 | hermetic test | 1 | logic and refusal paths, deterministic | wiring |
| L3 | fuzzing (canonicalisation and parsing) | 1 | parsers and canonical forms survive hostile bytes in the cases tried | semantic errors behind a valid parse |
| L4 | integration | 2, 3 | wiring across real components | real-agent behaviour (tier 2); two real agents (tier 3) |
| L5 | live drill (incl. break-glass) | 4, operator runme | the estate and the operator's procedure work together once | frequency, drift |
| L6 | chaos and recovery | 1 to 4, EP-12 | survival of the faults injected | faults not injected |
| L7 | production invariant check (in operation: status, canary, guard layer, phase reports) | 4, live | the property holds in operation over the period observed | that it held before or will after |

4.1 L7 is added to the card's hierarchy because the card requires "every measured production property from the
architecture has an invariant check" (3.5) and none of the six levels names checks that run in operation.
4.2 A spec states the lowest level that can prove its claim and, where the claim has a wiring half, a second spec at
the higher level. A higher level never replaces a lower one: a passing live run does not excuse a missing fixture.

## 5 Result format (card 4.3)

5.1 **One record per run of a spec [P]**, one JSON object per line, schema `termlink.evidence/1`:
`{schema, spec: "E-n", run_id, at, commit, artifact_sha256, tier, level, estate: "EP-n", verdict, assertions:
[{name, expected, observed, ok}], control: {name, verdict_with_control_removed}, limit: null | "RR-n", seed}`.
5.2 `verdict` is one of `PASS`, `FAIL`, `TOOLING`, `LIMIT`. Exit codes follow the project rule (CLAUDE.md rule 6):
0 PASS, 1 FAIL, 2 TOOLING. A tooling error is never reported as PASS.
5.3 `LIMIT` is the expected result of a limit probe (RR-7, RR-15, RR-19, RR-20 and the like): the run shows the known
limit as stated. It exits 0 only if the observed behaviour equals the stated limit and nothing worse; any other
outcome is FAIL. A LIMIT is never counted as a pass of a control (TM 18.2, SI-17 probe (2)).
5.4 A spec is **kill-checked** when its record carries `control.verdict_with_control_removed == "FAIL"` from a run on
the same commit with the control removed (the mutant applied). A spec without a kill-checked control is not counted
as evidence for a floor item (FP 3.1.b, R-69.a).
5.5 Where results are recorded **[P]**: tier 1 and 2 in the guard-layer log and, at a phase exit, in the PH-n FC
matrix report (ARCH2 4 exit 1); tier 3 and 4 in `docs/reports/<task>-<spec>-<date>.md` with the JSONL attached; the
canary in its log with staleness checked (V-13); L7 checks as status fields, needs-attention entries and the guard
layer's 0/1/2 exit. The phase exit report (B-22 for PH-2) lists every spec of the phase with its last verdict and
the commit, and lists every spec not run with its reason.
5.6 Secrets: every record and log is part of the SI-25 scan (E-101); a record never contains a key, secret or token.

## 6 PH-2 specs: the next build items (operator priority)

### 6.1 Doorbell D+ and the 300 s watchdog (K-19, K-33, K-18; ARC4 6.9; B-16)

**E-1 — Ring gate: the doorbell rings only when all four D+ conditions hold.** Links: R-20, R-19, R-3, SI-2, TH-62, AFC-1. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-16).
E-1.1 EVIDENCE: for one session and one tick, a ring happens if and only if (a) mail is STORED, not handed over and no hook fired for it within 1 tick, (b) READY, (c) no client input for 30 s (V-17), (d) conversation `none` (ARC4 6.9.1).
E-1.2 ARRANGE: simulated tick clock; fake session under a TermLink PTY with settable READY flag, per-source input `last_at`, conversation signal, foreground `claude`; one STORED message.
E-1.3 ACT: run one tick for each of the 16 combinations of (a)..(d).
E-1.4 ASSERT: `rings` equals 1 in the all-true row and 0 in the other 15; at most one ring per session per tick; every decision record lists the four inputs with their ages. Result: a 16-row table compared with the expected table, exit 0 on equality.
E-1.5 CONTROL: mutant that drops condition (d) rings in the `conversation: operator` row; mutant that reads a missing READY flag as ready (ARCH FC-5 pattern) rings in the `READY absent` row.

**E-2 — Pre-typing re-checks run in order, each failing closed.** Links: SI-2, SI-23, R-20, R-22, R-81, TH-16, AFC-2. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-16).
E-2.1 EVIDENCE: after the gate passes, the six re-checks of ARC4 6.9.2 (kill switch; READY re-read with age ≤ 1 tick; foreground process is the registered harness; process-tree veto; no client input in 30 s re-read; conversation still `none`) each stop the keystrokes when they fail, and the interval from the last check to the keystrokes is logged (H-11 (a)).
E-2.2 ARRANGE: as E-1, gate true; a hook in the fake session that flips exactly one re-check input between the gate and the keystrokes.
E-2.3 ACT: six runs, one per re-check; a seventh with none flipped.
E-2.4 ASSERT: runs 1..6: zero bytes written to the PTY by source `notify`, decision record `held:<check>` naming the flipped check; run 7: one `command.notify` call and a logged `check_to_keystroke_ms`. No run reads the PTY output buffer (R-22: the screen is never read; asserted by the fake session counting output reads, expected 0).
E-2.5 CONTROL: removing any one re-check makes its run write keystrokes (six mutants, one per check).

**E-3 — The typed line is inert and carries no peer byte (SI-1).** Links: SI-1, R-23, TH-6, TH-16, AFC-3. Category: adversarial. Level: L3 + L2. Tier: 1 and 2. Phase: PH-2 (B-16).
E-3.1 EVIDENCE: every line the doorbell can produce matches `^sidecar: [0-9]+ peer messages waiting, ids( [0-9a-f]{32}){1,4}\. The prompt hook shows them as untrusted data\.$`; no byte chosen by a peer reaches it; typed into a shell it starts nothing.
E-3.2 ARRANGE: tier 1: a generator of message ids and counts, including ids with a newline, ESC sequences, `$(id)`, backticks, `;`, Unicode look-alikes, 31 and 33 hex characters, upper-case hex, an empty id, and more than four pending messages. Tier 2 (EP-4): a TermLink PTY whose foreground process is `bash`, then `sh`, then `zsh`, run as root and as an unprivileged user.
E-3.3 ACT: tier 1: build the line for each generated input (at least 10^5 cases, seeded). Tier 2: write the fixed line produced for a valid id set into each shell and press Enter.
E-3.4 ASSERT: tier 1: every produced line matches the regular expression; every input with an invalid id is refused before a line exists and the refusal is recorded; at most 4 ids appear. Tier 2: the only new process is the shell's "command not found" path (exit 127; process list before and after differs by nothing that outlives it); `command -v sidecar` is empty on the host (also an L7 check on every fleet host, E-121).
E-3.5 CONTROL: mutant that interpolates the message's `name_hint` or subject into the line fails the regular expression; mutant that skips id validation lets a newline through.

**E-4 — A ring is ATTEMPTED, never HANDED_OVER; no false "delivered".** Links: R-24, R-42, FX-7, SI-17, SI-30, TH-7, AFC-4. Category: adversarial. Level: L2 + L4. Tier: 1 and 3. Phase: PH-2 (B-16).
E-4.1 EVIDENCE: the only writer of HANDED_OVER is the evidence check (nonce in a harness-written hook-context or user record); a ring, a pull, a nonce echoed in an assistant record or a tool result never produce it; after 90 s without evidence the message carries the HANDOVER_UNCONFIRMED label and stays owed (ARC4 6.9.4, 17.6).
E-4.2 ARRANGE: tier 1: estate with a fake transcript writer that can place the nonce in (i) a hook-context record, (ii) an assistant record, (iii) a tool result, (iv) nowhere. Tier 3 (EP-5): `testdev-real` with the doorbell on, `testdev-peer` sending.
E-4.3 ACT: ring, let the hook pull, write the transcript case; advance 90 s.
E-4.4 ASSERT: (i) HANDED_OVER with `record_ref`; (ii), (iii), (iv) no HANDED_OVER, stage ATTEMPTED with label HANDOVER_UNCONFIRMED at 90 s, sender state not "delivered" (FX-7 wording "accepted"/"stored" only). Tier 3: for 20 rings, HANDED_OVER count equals the count of nonces found in user-type records of the real transcript, exactly.
E-4.5 CONTROL: the built mutants `fc-06-ready-flag-taken-as-evidence`, `fc-07-storage-counted-as-delivery`, `fc-11-any-record-is-evidence` (ARCH FC) re-run against the doorbell path; a new mutant "ring counted as delivered" MUST turn E-4 red.

**E-5 — Back-off, one ring per tick, and no re-type of an unsubmitted line.** Links: R-25, R-44, R-20, AFC-5. Category: edge case. Level: L2. Tier: 1. Phase: PH-2 (B-16).
E-5.1 EVIDENCE: without hand-over the session is rung at 0, then after 4 and 8 ticks, then at most every 10 minutes; at the R-44.e deadline the message becomes STUCK with a notice; while a typed line has had no UserPromptSubmit carrying its ids, the doorbell does not ring that session again (ARC4 6.9.4, 6.9.5 (b)).
E-5.2 ARRANGE: simulated clock; a session that stays READY and idle; a fake adapter that never submits.
E-5.3 ACT: run 60 simulated minutes.
E-5.4 ASSERT: ring times equal the expected schedule; no two rings in one tick; the H-11 (e) counter equals the number of rings not followed by a submit within 1 tick; after the first unsubmitted ring no further normal ring occurs (only the watchdog may, E-7); STUCK appears at the R-44.e deadline with a notice to the sender.
E-5.5 CONTROL: mutant that re-rings every tick fails the schedule; mutant that re-types on top of an unsubmitted line rings twice.

**E-6 — A restored stale doorbell line pulls nothing and is counted (T-3591).** Links: SI-30, R-51, R-24, AFC-6. Category: edge case. Level: L4. Tier: 3. Phase: PH-2 (B-16).
E-6.1 EVIDENCE: a doorbell line replayed from the harness's prompt history for mail already handed over delivers nothing twice and is recorded as a stray ring (H-11 (b)).
E-6.2 ARRANGE: EP-5; one message handed over with evidence.
E-6.3 ACT: submit the same doorbell line again in `testdev-real` (as the history restore does).
E-6.4 ASSERT: the hook pulls zero messages; the transcript holds exactly one delivery frame for that id; the stray counter rises by 1.
E-6.5 CONTROL: with the atomic claim removed (ARCH `fc-04-no-claim-file`) the replay delivers a second frame.

**E-7 — The watchdog fires after 300 s of silence even when READY says busy.** Links: R-19, R-44, R-24, AFC-7. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-16).
E-7.1 EVIDENCE: for a session with mail STORED and not handed over, when S1, S2 and S3 show no activity for 300 s (V-18), the watchdog types the SI-1 line once, independent of READY and of the conversation signal, writes its audit row and marks the stage `via: watchdog` (ARC4 6.9.7).
E-7.2 ARRANGE: simulated clock; READY forced `busy`; conversation forced `agent` for another conversation; S1, S2, S3 frozen; veto clear; foreground `claude`; one STORED message.
E-7.3 ACT: advance to 299 s, then to 300 s + 1 tick.
E-7.4 ASSERT: at 299 s no keystroke; at 300 s + 1 tick exactly one `command.notify`; one admin-chain row with every field of ARC4 6.9.7.c (`kind`, `session`, `at`, `silent_for_s`, `ready_flag`, `ready_age_s`, `last_activity {s1,s2,s3}`, `checks`, `outcome: typed`, `message_ids`); hub stage ATTEMPTED with `via: watchdog`; one needs-attention entry "READY flag busy for N s while nothing happened" (coalesced: a second fire the same day adds none).
E-7.5 CONTROL: mutant that makes the watchdog check READY never fires; mutant without the audit row fails the row assertion.

**E-8 — The watchdog's own checks hold it, and it never types into a shell.** Links: SI-2, SI-23, R-22, AFC-8. Category: error handling. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-16).
E-8.1 EVIDENCE: the watchdog types only after the kill switch, the foreground-is-harness check and the process-tree veto pass; each hold writes `outcome: held:<check>`.
E-8.2 ARRANGE: as E-7 at the firing moment; tier 2 (EP-4) with `bash` as foreground.
E-8.3 ACT: three runs: kill switch on; foreground a shell; veto active (CPU in the agent's tree).
E-8.4 ASSERT: no `notify` bytes in all three; one audit row each with the named hold; tier 2: the shell received zero bytes.
E-8.5 CONTROL: mutant that skips the foreground check types into `bash` in tier 2.

**E-9 — Arriving mail, receipts and doorbell keystrokes are not activity.** Links: R-44, R-19, AFC-9. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-16, B-27).
E-9.1 EVIDENCE: a steady inflow cannot keep the watchdog from firing: STORED arrivals, receipts, marked automatic content, keep-alive traffic and `notify` keystrokes do not move S1, S2 or S3 (ARC4 6.14.2).
E-9.2 ARRANGE: as E-7, plus a peer that posts a message every 10 s, receipts every 5 s and keep-alives.
E-9.3 ACT: advance 300 s + 1 tick.
E-9.4 ASSERT: the watchdog fires once at 300 s + 1 tick; `last_activity` is unchanged by the inflow.
E-9.5 CONTROL: mutant that counts STORED arrivals as S3 never fires within 30 simulated minutes.

**E-10 — No blind watchdog: S2 unknown means "watchdog unavailable".** Links: R-29, FX-8, AFC-10. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-16).
E-10.1 EVIDENCE: a session whose S2 source is `unknown` (no adapter report, no transcript path) has no watchdog and says so (ARC4 6.14.3).
E-10.2 ARRANGE: session without adapter and without transcript path; mail STORED; silence 600 s.
E-10.3 ACT: run 20 ticks.
E-10.4 ASSERT: zero watchdog fires; status shows `watchdog: unavailable` for the session; the message reaches its R-44.e deadline as STUCK with a notice (visible failure, 15.2.b).
E-10.5 CONTROL: mutant that treats `unknown` as "no activity" fires.

**E-11 — Watchdog bound and liveness over arbitrary activity traces.** Links: R-44, R-19, AFC-7, AFC-11. Category: core function. Level: L1. Tier: 1. Phase: PH-2 (B-16).
E-11.1 EVIDENCE: for every activity trace, (safety) every fire is preceded by at least 300 s with no S1, S2 or S3 event and no earlier fire in that silence; (liveness) if mail stays pending and the session stays silent and unvetoed, a fire happens within 300 s + 1 tick (the K-33 measured property, ARC4 15).
E-11.2 ARRANGE: property-based generator of traces over 2 simulated hours: S1/S2/S3 events, READY flips, veto windows, kill-switch flips, hand-overs.
E-11.3 ACT: run the watchdog model and the implementation's decision function on at least 10^4 traces (seeded, shrinking on failure).
E-11.4 ASSERT: safety and liveness hold on every trace; implementation decisions equal the model's.
E-11.5 CONTROL: mutant with a 299 s interval violates safety; mutant that resets silence on its own fire's keystrokes violates liveness.

**E-12 — The kill switch stops the typed path within one tick (live).** Links: SI-23, R-63, TH-40, AFC-12. Category: core function. Level: L5 + L2. Tier: 1 and 4. Phase: PH-2 (B-16; ARCH2 exit 4).
E-12.1 EVIDENCE: delivery off for one agent and for the fleet stops both the hook path and the doorbell and watchdog paths within one tick, and on a partitioned host at once through the local admin scope; every switch is audited.
E-12.2 ARRANGE: tier 1 simulated; tier 4 EP-7 during W2, a runme drill action (as `scripts/arc011-w1-killswitch-drill.sh`, extended to the typed path).
E-12.3 ACT: switch off; send an urgent message from an allowed sender; let the watchdog interval elapse; switch on.
E-12.4 ASSERT: no `notify` bytes and no mid-turn hand-over after the switch's tick; the urgent message is downgraded, not dropped; the CLI exits 4 on the typed path as ruled; one audit row per switch; after switch-on the message is delivered by the normal rules.
E-12.5 CONTROL: drill with the switch read removed from the watchdog path types within the drill (run in tier 1 only).

**E-13 — H-11 measured over 7 days in W2.** Links: R-20, R-81, SI-2, TH-62, H-11, H-20. Category: integration. Level: L7. Tier: 4. Phase: PH-2 (B-16, B-17).
E-13.1 EVIDENCE: on EP-7 for 7 days, (a) check-to-keystroke p99 ≤ 100 ms [P]; (b) stray lines ≤ 1 per 1000 rings; (c) vetoes while idle and ready ≤ 10% of ticks; (d) rings within 2 s of a turn start ≤ 1 per 1000; (e) typed lines not followed by a submit within 1 tick, counted; (f) watchdog fires per 1000 rings and the share that found READY false-busy, reported.
E-13.2 ARRANGE: W2 runme action run by the operator; ring log and audit rows on.
E-13.3 ACT: normal work for 7 days.
E-13.4 ASSERT: the H-11 report computes (a)..(f) from the ring log and the transcripts; (a)..(d) against their bounds; (e) and (f) reported with numerator and denominator per attempt (RNU: failures per ring, never rings per day). Result: `docs/reports/<B-17 task>-h11.md` with the JSONL.
E-13.5 CONTROL: the report generator is checked on a seeded log with known counts (tier 1) and goes red when a stray line is dropped from its input.

**E-14 — Limit probe: the check-to-keystroke race (RR-19, AR-2).** Links: SI-2, TH-62, RR-19, AR-2. Category: adversarial. Level: L2 (limit). Tier: 1. Phase: PH-2 (B-16).
E-14.1 EVIDENCE: a turn that starts between the last re-check and the keystrokes still receives the line; the race is narrowed, not closed.
E-14.2 ARRANGE: controllable fake harness that starts a turn at a chosen offset after the last re-check.
E-14.3 ACT: offsets 0, 1, 10, 50 ms.
E-14.4 ASSERT: the observed outcome equals the stated limit (a line arrives in a busy prompt and is counted as stray by H-11 (d)); verdict `LIMIT`, never PASS.
E-14.5 CONTROL: expected result `LIMIT`; any outcome other than the stated limit (for example a lost line not counted) is FAIL.

### 6.2 Activity meter and conversation signal (K-32; ARC4 6.14; B-27)

**E-15 — S1 counts client keystrokes only.** Links: R-20, R-81, SI-2, AFC-13. Category: core function. Level: L4. Tier: 2. Phase: PH-2 (B-27).
E-15.1 EVIDENCE: the TermLink session counts PTY input per source (`client`, `notify`, `inject`); S1 moves only on `client` (ARC4 6.14.1, 6.14.2).
E-15.2 ARRANGE: EP-4, a real TermLink PTY session with an attached scripted client; the sidecar reading `last_at` per source each tick through the `notify`-scope RPC.
E-15.3 ACT: write keystrokes as the client; call `command.notify`; call `inject`.
E-15.4 ASSERT: `s1_at` moves after the client write only; `notify` and `inject` move their own counters and not S1; the RPC is refused with an `observe` token and served with `notify`.
E-15.5 CONTROL: mutant that sums all sources into S1 lets a doorbell keystroke reset the watchdog (E-9 goes red too).

**E-16 — S2 moves on turn events and transcript growth; unparseable reports count for nothing.** Links: R-47, R-21, AFC-14. Category: adversarial. Level: L3 + L2. Tier: 1. Phase: PH-2 (B-27).
E-16.1 EVIDENCE: `prompt_submit`, `tool_start`, `tool_end`, `turn_stop`, `session_start`, `session_end` and transcript growth move S2; a report the receiver cannot parse is ignored and counted and never reads as READY, human or evidence (ARC4 17.2).
E-16.2 ARRANGE: fake adapter; a fuzzer over `termlink.adapter/2` objects (missing fields, wrong types, unknown `event`, `origin: "Human"`, extra fields, 1 MiB strings, invalid UTF-8, duplicated keys).
E-16.3 ACT: send each valid event once; send at least 10^5 fuzzed objects.
E-16.4 ASSERT: valid events move `s2_at`; every unparseable object increments `adapter_reports_unparseable` and changes neither READY, nor origin, nor any stage; no crash, no unbounded memory (the parser's allocation is capped, `scripts/check-alloc-sink-clamps.sh` green).
E-16.5 CONTROL: mutant that defaults a missing `state` to READY fails; mutant that accepts `origin` case-insensitively fails the `"Human"` case.

**E-17 — S3 moves on sent and handed-over messages, never on arrivals.** Links: R-20, AFC-9. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-27).
E-17.1 EVIDENCE: S3 is a message the session's agent sent, or one handed over to it; STORED arrivals, receipts and marked automatic content do not count (ARC4 6.14.1, 6.14.2).
E-17.2 ARRANGE: estate EP-1, session of `fx-a`.
E-17.3 ACT: `fx-a` sends; `fx-b` sends to `fx-a` (STORED only); hand it over; post a receipt and an automatic notice.
E-17.4 ASSERT: `s3_at` moves on the send and on the hand-over only.
E-17.5 CONTROL: shared with E-9 (arrivals counted as S3).

**E-18 — Conversation signal: values, windows and independence from READY.** Links: R-20, SI-2, R-21, AFC-15. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-27).
E-18.1 EVIDENCE: `operator` from a `human` or `unknown` prompt until 120 s (V-19) after its turn stopped, extended by any client input; `agent` for 120 s after the session's agent sent or was handed a conversation message with an owed answer; while `operator`, no ring at all; while `agent`, only mail of other conversations is held; READY never enters the computation (ARC4 6.14.4).
E-18.2 ARRANGE: simulated clock; a table of event sequences, each with READY forced to every value.
E-18.3 ACT: evaluate the signal and the ring decision at 119 s, 120 s, 121 s after each trigger; with a client keystroke at 100 s; with an `automated` prompt; with mail of the same and of another conversation.
E-18.4 ASSERT: the signal and ring decisions equal the expected table for every READY value (identical rows); `automated` prompts never produce `operator`; `unknown` produces `operator`.
E-18.5 CONTROL: mutant that reads READY into the signal produces different rows per READY value; mutant that treats `unknown` as `automated` rings during an unknown-origin turn.

**E-19 — Status `activity` per session is fresh or reads unknown.** Links: R-6, R-29, FX-8. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-27, B-14).
E-19.1 EVIDENCE: status shows per session `activity {s1_at, s2_at, s3_at, last, source, conversation}` with freshness stamps; a stale field reads "unknown" (ARC4 6.14.5, FP-1 7.2.b).
E-19.2 ARRANGE: estate session; freeze the meter with `TERMLINK_SIDECAR_TEST_FREEZE`.
E-19.3 ACT: read status before and after the staleness bound.
E-19.4 ASSERT: fields present with stamps; after the bound every frozen field reads `unknown`, never its last value.
E-19.5 CONTROL: mutant that serves cached values without a stamp fails.

### 6.3 Main lease tightening and the waiting-mail signal (K-22, K-34; ARC4 6.13; B-25)

**E-20 — Only human-origin claims renew, acquire or take over.** Links: R-67, R-68, SI-20, TH-33, TH-41, AFC-16. Category: adversarial. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-25).
E-20.1 EVIDENCE: a claim with origin `human` counts; `automated` and `unknown` are answered with the built exit codes but change nothing and are recorded `ignored: origin` in the lease's recent records (ARC4 6.13.1).
E-20.2 ARRANGE: tier 2 throwaway hub with the lease and `TERMLINK_MAIN_TEST_*` seams; two sessions S1 (holder) and S2; project already origin-reporting.
E-20.3 ACT: for each origin (`human` basis `pty-input`, `human` basis `inferred`, `automated` each basis, `unknown`) claim from S1 (renew) and from S2 (takeover attempt) at every lease state.
E-20.4 ASSERT: generation and holder change only on `human`; every other claim leaves `termlink main status` unchanged except a record `ignored: origin` with the basis; exit codes as LEASE ("exit 0 main, 1 not-main, 2 unknown").
E-20.5 CONTROL: mutant that counts `unknown` as human renews in the `unknown` row (ARC4 6.13.6).

**E-21 — PostToolUse no longer renews; SessionStart takes a vacant lease only provisionally.** Links: R-67, AFC-16. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-25).
E-21.1 EVIDENCE: tool activity never renews; a SessionStart on a vacant lease yields `provisional`, never `held` (ARC4 6.13.1, 6.13.2).
E-21.2 ARRANGE: hook fixtures for the TermLink hooks with a fake hub client recording calls.
E-21.3 ACT: run PostToolUse 100 times over 20 simulated minutes with no human prompt; run SessionStart on a vacant lease.
E-21.4 ASSERT: no renewal call carries a counted claim from PostToolUse; the lease lapses at 15 min (V-9); SessionStart yields `state: provisional, valid_s: 300`.
E-21.5 CONTROL: the built hook (renew on PostToolUse) keeps the lease held for 20 min and fails.

**E-22 — Projects whose hooks send no origin keep working (NID).** Links: R-67, AFC-17. Category: edge case. Level: L2. Tier: 1. Phase: PH-2 (B-25).
E-22.1 EVIDENCE: a claim with no origin field is counted as before and recorded `origin: absent`, status says "this project's hooks do not report prompt origin"; from the first claim of a project that carries an origin field, `absent` claims from it are ignored like `unknown` (ARC4 6.13.1.a).
E-22.2 ARRANGE: hub with two projects P (old hooks) and Q.
E-22.3 ACT: P claims without origin for 30 minutes; Q claims once with origin `human`, then without origin.
E-22.4 ASSERT: P's lease stays held and renewed, with the status line; Q's later `absent` claim is `ignored: origin`; P is never marked origin-reporting by Q's claim.
E-22.5 CONTROL: mutant that ignores `absent` everywhere lets P's lease lapse (an implicit switch-off, NID).

**E-23 — Provisional claim: taken at once by a human claim, lapses in 5 min, is main while it holds.** Links: R-67, SI-20, AFC-16. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-25).
E-23.1 EVIDENCE: a provisional holder never blocks anyone, lapses straight to VACANT after 5 min without a human prompt (V-20), and answers `am-i` as main while it holds (ARC4 6.13.2).
E-23.2 ARRANGE: simulated clock; S1 provisional.
E-23.3 ACT: (a) S2 human claim at 1 min; (b) no claim, 4:59 and 5:01; (c) `am-i` from S1 at 2 min.
E-23.4 ASSERT: (a) S2 held, generation +1, no quiet period, no cooldown; (b) provisional at 4:59, VACANT at 5:01; (c) exit 0.
E-23.5 CONTROL: mutant that applies the quiet period to a provisional holder refuses (a).

**E-24 — Fast takeover at 5 min of holder human silence, never at 4:59, never of a pinned holder.** Links: R-67, R-68, SI-20, TH-33, AFC-16. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-25).
E-24.1 EVIDENCE: a human claim from S2 takes main at once (generation +1, record `reason: fast-takeover, from: S1, holder_human_age_s`) when S1's `last_human_at` is older than 5 min and S1 is not pinned (ARC4 6.13.3).
E-24.2 ARRANGE: simulated clock; S1 held with a human renewal at t0; variant with S1 pinned by the operator.
E-24.3 ACT: S2 human claim at t0 + 4:59 and at t0 + 5:01; same against the pinned S1; an `automated` claim from S2 at t0 + 30 min.
E-24.4 ASSERT: 4:59 refused (S1 keeps main); 5:01 taken with the record fields; pinned S1 never taken; the `automated` claim changes nothing.
E-24.5 CONTROL: mutant without the 5 min check takes at 4:59 (ARC4 6.13.6).

**E-25 — Lease state machine: model check of D-4 (b).** Links: R-67, R-68, SI-20, TH-33, TH-41, AFC-16. Category: core function. Level: L1. Tier: 1. Phase: PH-2 (B-25).
E-25.1 EVIDENCE: over every sequence of claims (each origin), renewals, releases, pins, lapses and clock steps in a bounded model (3 sessions, 2 agents, 40 steps): at most one holder per generation; the generation never decreases; a non-human claim never changes holder or generation; a pinned holder is displaced only by an operator pin; every holder change has a record; the 6.13.3 bound holds (main reaches a session prompted at least 5 min after the holder's last human prompt at that prompt).
E-25.2 ARRANGE: an explicit-state model of ARC4 10.1 (b) and 6.13; the implementation's transition function exposed to the checker.
E-25.3 ACT: exhaustive exploration to the bound, plus random walks of 10^4 steps.
E-25.4 ASSERT: no invariant violated; implementation transitions equal the model's on every explored state. Result: state count and the violated invariant (if any) with its trace.
E-25.5 CONTROL: mutant "operator pin not recorded" violates the record invariant; mutant "automated claim may resume a LAPSED lease" violates the non-human invariant.

**E-26 — Lapse at 15 min without a human renewal, even with a busy agent; resume and take-at-once.** Links: R-67, R-68, AFC-16. Category: edge case. Level: L2. Tier: 1. Phase: PH-2 (B-25).
E-26.1 EVIDENCE: a holder lapses after 15 min (V-9) without a human prompt whatever its agent does; a human prompt in the holder resumes at the same generation; one in another session takes at once (ARC4 6.13.4).
E-26.2 ARRANGE: simulated clock; S1 held; S1's agent busy (tool events every 30 s, automated prompts).
E-26.3 ACT: advance 15:01; then (a) human prompt in S1, or (b) human prompt in S2.
E-26.4 ASSERT: LAPSED at 15:01; (a) held by S1, same generation; (b) held by S2, generation +1, no quiet period.
E-26.5 CONTROL: mutant counting tool events as renewal never lapses.

**E-27 — The doorbell's own line never renews main.** Links: R-67, R-23, AR-6, AFC-16. Category: adversarial. Level: L4. Tier: 3. Phase: PH-2 (B-25, B-29).
E-27.1 EVIDENCE: the UserPromptSubmit triggered by a doorbell or watchdog line is reported `automated` basis `doorbell` and does not renew or take main (ARC4 17.5.a, 17.9 negative control).
E-27.2 ARRANGE: EP-5, `testdev-real` not main, doorbell on; a second testdev session holds main.
E-27.3 ACT: a ring is typed and submitted in `testdev-real`.
E-27.4 ASSERT: lease records show `ignored: origin, basis: doorbell` for that claim; holder unchanged.
E-27.5 CONTROL: adapter fixture reporting the doorbell line as `human` fails the parity scenario (E-108).

**E-28 — Waiting-mail signal: 10 min, its four landing places, and honest OOB status.** Links: R-67, R-53, R-29, FX-8, AFC-18. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-25; B-28 for Watchtower and OOB).
E-28.1 EVIDENCE: role mail STORED and not handed over for 10 min (V-21) while the lease is vacant, lapsed, provisional or unknown, or its holder has had no human prompt for 10 min, raises: a needs-attention entry and status `main.mail_waiting {count, oldest_age_s, holder, holder_human_age_s, since}` at once; one session-start line; one line per generation in a non-main session at its next human prompt; an out-of-band notice at 30 min, at most one per project per hour; before GP-11 exists status shows `oob: not available` and never "sent"; it clears on hand-over (ARC4 6.13.5). The session identity is `CLAUDE_CODE_SESSION_ID` (LEASE, T-3668).
E-28.2 ARRANGE: simulated clock; role mail for project P; each of the five lease conditions; a fake OOB sink, absent and present.
E-28.3 ACT: advance 9:59, 10:00, 29:59, 30:00, 90:00; start a session; prompt a non-main session twice in one generation; hand the mail over.
E-28.4 ASSERT: nothing at 9:59; all fields at 10:00; the session-start line from `scripts/session-start-alerts.sh`; exactly one non-main line per generation; OOB at 30:00 then not again before 90:00 (two in 90 min at most); with the sink absent `oob: not available` and zero "sent" records; the field clears at hand-over. The weekly count is reported, never capped (RNU).
E-28.5 CONTROL: mutant that reports OOB "sent" without a sink fails; mutant keyed on exact-instance mail raises the signal for conversation mail and fails.

**E-29 — Same signal definition at AEF's receiver (parity).** Links: R-47, R-67, R-49, AFC-18. Category: integration. Level: L2. Tier: 1. Phase: PH-2 (B-29, B-28; AEF T-5054).
E-29.1 EVIDENCE: for the same facts, the TermLink sidecar and AEF's receiver raise the same waiting-mail signal (ARC4 17.7).
E-29.2 ARRANGE: one scenario file (facts: lease state, holder human age, role mail ages) shipped with the adapter parity fixtures.
E-29.3 ACT: run it against both receivers.
E-29.4 ASSERT: identical `mail_waiting` objects at every checkpoint of E-28.
E-29.5 CONTROL: scenario variant with exact-instance mail must give "no signal" on both.

**E-30 — R-68 standing tests: both end in a visible takeover.** Links: R-68, R-67, SI-20, TH-33. Category: integration. Level: L4. Tier: 2. Phase: PH-2 (built lease; re-run with B-25).
E-30.1 EVIDENCE: (1) a lease lapse during an open obligation and (2) a stuck holder with a live sibling each end in a takeover visible in `termlink main status` and to the sender (R-68.a).
E-30.2 ARRANGE: throwaway hub, two sessions, an owed conversation message to main.
E-30.3 ACT: (1) stop the holder's human prompts; (2) freeze the holder while the sibling gets human prompts.
E-30.4 ASSERT: (1) lapse then takeover by the next human prompt, generation +1; (2) fast takeover after 5 min (E-24); in both the owed message reaches the new main and the record names the takeover.
E-30.5 CONTROL: with takeover disabled the owed message stays unhandled and the test fails by its deadline.

**E-31 — W2 measurement: renewals by basis and ignored claims.** Links: R-67, AR-6. Category: integration. Level: L7. Tier: 4. Phase: PH-2 (B-25, B-17).
E-31.1 EVIDENCE: over 7 days on EP-7b the share of counted renewals with basis `pty-input` and `inferred`, and the count of `ignored: origin` claims, are known (ARC4 6.13.6; AR-6 is measured, not closed).
E-31.2 ARRANGE: lease records retained for 7 days.
E-31.3 ACT: normal work.
E-31.4 ASSERT: report with numerator and denominator per basis; no threshold (a measurement for AR-6); any `automated` claim that changed the holder is a FAIL.
E-31.5 CONTROL: report generator checked on a seeded record set.

### 6.4 Forward-once of role mail (K-23; ARC4 7; B-20)

**E-32 — Forward-once happy path across two sidecars.** Links: R-67.a1, R-51, R-62, TH-33, AFC-19. Category: core function. Level: L4. Tier: 2. Phase: PH-2 (B-20).
E-32.1 EVIDENCE: role mail reaching agent Y while the lease names X of another agent of the same project is stored at Y, forwarded once in a `FORWARD` envelope signed by Y carrying the original signed bytes unchanged with `generation` and `hop` +1, handed over at X, and the sender gets a signed `VIA` notice (ARC4 7.2, 7.3).
E-32.2 ARRANGE: throwaway hub; project P with agents `fx-x` and `fx-y` on separate sidecars; lease held by `fx-x`; sender `fx-s`.
E-32.3 ACT: `fx-s` sends to `main` of P at Y's inbox.
E-32.4 ASSERT: Y's copy STORED then `forwarded` (final at Y, never handed over at Y); X's inbox holds one FORWARD whose inner bytes equal the original byte for byte and whose inner signature verifies against the original topic; X hands over once; the sender's record shows the VIA line "delivered via Y; main is X (generation N)".
E-32.5 CONTROL: mutant that re-signs the content as Y (DR-10 option B) fails the byte-equality and inner-signature assertions.

**E-33 — FORWARD-STALE when the lease moves in flight.** Links: R-67.a1, R-67.e, R-51, AFC-19. Category: edge case. Level: L4. Tier: 2. Phase: PH-2 (B-20).
E-33.1 EVIDENCE: if X is no longer main when the FORWARD arrives, X stores it, does not forward it, posts `FORWARD-STALE`; the sender re-resolves through the home hub and re-sends the same message id; exactly one hand-over happens, at the new main (ARC4 7.3).
E-33.2 ARRANGE: as E-32, plus a third agent `fx-z`; a test hook that moves the lease to `fx-z` between Y's forward and X's pull.
E-33.3 ACT: send; move the lease; let all pull.
E-33.4 ASSERT: one FORWARD-STALE notice; one re-send with the same id; exactly one HANDED_OVER across X, Y, Z (at Z); identity-triple dedupe holds (R-51).
E-33.5 CONTROL: mutant without the generation check at X hands over at X and again at Z (two hand-overs).

**E-34 — Forward refusals: what is never forwarded or accepted.** Links: R-67.a1, R-62, SI-3, SI-26, TH-33, AFC-19. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-20).
E-34.1 EVIDENCE: a FORWARD is accepted only with an outer signature by a roster member of the same project, an intact inner signature for a role address of that project and a matching generation; Y never forwards conversation mail bound to an exact instance, never forwards a FORWARD, and never forwards on an unknown, stale or vacant lease (ARC4 7.2.d, 7.3).
E-34.2 ARRANGE: estate EP-2 with a project-foreign signer `fx-c`.
E-34.3 ACT: deliver (a) a FORWARD signed by `fx-c`; (b) one with one inner byte flipped; (c) one whose inner topic is another project's role address; (d) a FORWARD of a FORWARD; (e) exact-instance mail to Y with the lease on X; (f) role mail to Y with the lease unknown, stale (> 15 s read period), vacant.
E-34.4 ASSERT: (a)..(d) refused, recorded, no hand-over, signed refusal notice to the outer signer; (e) delivered at Y as addressed, not forwarded; (f) delivered at Y as before, status says why no forward; `hop` never exceeds 1 for a forward.
E-34.5 CONTROL: one mutant per check (outer roster check, inner signature, generation, hop rule, exact-instance rule, lease freshness), each turning its case red.

**E-35 — H-24, forward part: 1000 forwards, none processed twice.** Links: R-67.a1, R-51, H-24, SI-30. Category: integration. Level: L6. Tier: 2. Phase: PH-2 (B-20); duplicate-main part PH-4.
E-35.1 EVIDENCE: across 1000 forwarded role messages with lease moves and sidecar kills at random points, zero messages are handed over twice and none is lost after STORED.
E-35.2 ARRANGE: EP-2 on a throwaway hub; a seeded schedule of lease moves (one per 50 messages) and kills of X or Y (one per 100).
E-35.3 ACT: send 1000 role messages.
E-35.4 ASSERT: per message id exactly one HANDED_OVER or a visible non-final state by its deadline; 0 doubles; 0 lost after STORED. Result: counts in the evidence record.
E-35.5 CONTROL: the E-33 mutant produces doubles in this run.

### 6.5 Hub identity, signing key and the revocation list (K-9a, K-9b, K-25; ARC4 6.1, 6.2; B-1, B-5)

**E-36 — Hub id: mint once, keep, refuse a silent re-mint.** Links: R-61, R-54, R-33, FX-2, TH-3, AFC-20. Category: core function. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-1).
E-36.1 EVIDENCE: a fresh runtime directory mints `hub.id` (fingerprint-derived when a certificate exists, else 64 random bits); an existing id is kept across restarts and certificate rotation; `hub.id` missing beside existing state refuses start with the stated message; a socket-only hub has an id; `hub.version` reads the file, never the fingerprint (ARC4 6.1).
E-36.2 ARRANGE: throwaway hub runtime directories: empty; with cert only (legacy hub); with `hub.secret` and cert but no `hub.id`; with `hub.id` and a regenerated cert; socket-only.
E-36.3 ACT: start the hub in each; call `hub.version`; restart; rotate the certificate; restart.
E-36.4 ASSERT: ids as expected (legacy hub keeps `first16(fingerprint)`, so no inbox renames); the partial-state case exits non-zero with "hub.id missing beside existing state" and writes no file; after rotation id and signing key unchanged; `hub.id` is 16 lowercase hex, mode 0644, written via temp + fsync + rename.
E-36.5 CONTROL: mutant "derive from the fingerprint on each call" fails the rotation case (ARC4 6.1.7).

**E-37 — Hub signing key: persisted, never printed, pinned on first contact.** Links: R-61, R-75, SI-25, SI-11, AR-1, AFC-20. Category: adversarial. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-1).
E-37.1 EVIDENCE: `hub.signing.key` (0600) persists like the secret; `hub.version` returns only `ed25519:<fingerprint>`; each sidecar pins the fingerprint on first verified contact and refuses a different key with a needs-attention entry naming both (ARC4 6.1.4, 6.1.5).
E-37.2 ARRANGE: throwaway hub; a sidecar with an empty pin; a second hub key prepared.
E-37.3 ACT: first contact; restart hub; swap the key file; contact again.
E-37.4 ASSERT: pin recorded; same key after restart; after the swap every signed hub object is refused and the entry names old and new fingerprints; the private key bytes appear in no log, status or record (SI-25 scan, E-101).
E-37.5 CONTROL: mutant that re-pins silently on mismatch passes the swapped key and fails. Limit (AR-1): a rogue key present at first contact is pinned; recorded as `LIMIT` in a separate run.

**E-38 — Mail-hub declaration: refuse on mismatch, no fallback.** Links: R-54, FX-2, TH-3, BP-8, AFC-21. Category: error handling. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-1; AEF owns the key, DR-7).
E-38.1 EVIDENCE: a sidecar whose config copy of `mail_hub` differs from `.framework.yaml`, or whose hub answers another `hub_id`, does not run, and status names both ids; there is no runtime-directory fallback (V-7, ARC4 9.10).
E-38.2 ARRANGE: two throwaway hubs (EP-3); project files with matching, differing and missing declarations.
E-38.3 ACT: start the sidecar in each case; then point the declared address at the second hub.
E-38.4 ASSERT: matching runs; differing and missing refuse at start (exit non-zero, reason); wrong hub at the address: refusal, status `hub {declared, match: false}` with both ids; no message posted to the wrong hub.
E-38.5 CONTROL: the built mutant `fc-14-no-hub-id-check` fails the wrong-hub case.

**E-39 — A revoked key is refused within one tick.** Links: SI-11, R-46, RK-1, TH-26, TH-56, AFC-22. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-5).
E-39.1 EVIDENCE: after `admin revoke --key F`, a message signed with F is refused at the receiver from the next tick with the hub reachable, recorded, and the sender sees a signed refusal notice; mail to the revoked agent waits (ARC4 6.2.5).
E-39.2 ARRANGE: throwaway hub; sidecars `fx-a` (sender, key F) and `fx-b`.
E-39.3 ACT: revoke F through the root-only admin socket; `fx-a` sends immediately and again after one tick; `fx-b` sends to `fx-a`.
E-39.4 ASSERT: time from revoke to the first refusal ≤ one tick + one pull; nothing STORED from F after that; the sender's record shows "refused-revoked"; `fx-b`'s message to `fx-a` waits (not dropped); admin-log row and needs-attention entry for the revocation (SI-24).
E-39.5 CONTROL: mutant that skips the revocation check stores the message.

**E-40 — Rollback of the list and restore from backup.** Links: SI-11, TH-58, RR-20, AFC-22. Category: adversarial. Level: L4 + L6. Tier: 2. Phase: PH-2 (B-5).
E-40.1 EVIDENCE: a list with a lower `seq` is refused and the previous kept and flagged; a sidecar restored from backup with an older list and its older stored `seq` detects the higher head on first contact and refuses the revoked key; with the hub unreachable after such a restore, normal delivery from the revoked key is the stated limit RR-20 (TM SI-11 probe).
E-40.2 ARRANGE: throwaway hub at `seq` 7 with F revoked at 6; a snapshot of the sidecar store taken at `seq` 5.
E-40.3 ACT: (a) serve `seq` 5 from a test proxy; (b) restore the snapshot and reconnect; (c) restore the snapshot with the hub unreachable and send from F.
E-40.4 ASSERT: (a) refused, status `ROLLBACK refused, kept seq 7`; (b) first head read shows 7, list fetched, F refused; (c) verdict `LIMIT` naming RR-20, with no circuit and no urgent delivery from F.
E-40.5 CONTROL: mutant that accepts any `seq` passes (a) and fails.

**E-41 — A failed pull is not an empty list; the stale rule narrows, never stops.** Links: SI-11, SI-18, R-63, DR-8, AFC-22. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-5).
E-41.1 EVIDENCE: when only the list fetch fails while posts succeed, status shows `unverified since T`; after PN-13 (1 h) the interrupt allow-list narrows to the own project's verified roster, urgent mail from others is downgraded (never dropped), a needs-attention entry appears, and keys on the last verified list stay refused (ARC4 6.2.4).
E-41.2 ARRANGE: simulated clock; a hub client stub that fails `revocation.head` and `revocation.get` only.
E-41.3 ACT: advance 59:59 and 60:01; send urgent from another project's allowed sender and from a key on the last list.
E-41.4 ASSERT: status values at both times; after 60:01 the urgent message is STORED and handed over as normal priority with a recorded downgrade; the revoked key's message refused; zero messages dropped.
E-41.5 CONTROL: mutant "failed pull treated as an empty list" accepts the revoked key (ARC4 6.2.6).

**E-42 — Forged or malformed revocation lists are refused (fuzz).** Links: SI-11, TH-26, TH-58, AFC-22. Category: adversarial. Level: L3. Tier: 1. Phase: PH-2 (B-5).
E-42.1 EVIDENCE: only a list whose canonical form (RFC 8785) hashes to `digest`, whose `signature` verifies over `termlink.revocation/1\n<digest>` with the pinned key, whose `prev_digest` chains and whose `seq` is not lower is accepted.
E-42.2 ARRANGE: a valid list; a mutator over JSON (key order, whitespace, number forms `1.0`/`1e0`, Unicode escapes, duplicate keys, missing fields, signature by another key, signature over the digest without the domain prefix, broken `prev_digest`).
E-42.3 ACT: feed at least 10^5 mutated lists.
E-42.4 ASSERT: every list that is not canonically equal to a validly signed one is refused; semantically equal re-encodings of the valid list verify the same; no crash.
E-42.5 CONTROL: mutant that verifies over the raw bytes instead of the canonical form fails the key-order cases.

**E-43 — Self-revocation and the audit of every revocation.** Links: SI-11, SI-24, TH-34, TH-56. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-5).
E-43.1 EVIDENCE: a `revocation.request` signed by the old key is accepted without the operator; one signed by another key is refused; every revocation writes an admin-log row, a needs-attention entry and re-signs the list (ARC4 6.2.2).
E-43.2 ARRANGE: throwaway hub; agents `fx-a`, `fx-b`.
E-43.3 ACT: `fx-a` self-revokes; `fx-b` submits a request to revoke `fx-a`'s new key.
E-43.4 ASSERT: first accepted (`by: self`), `seq` +1; second refused and logged; each accepted revocation has its three traces.
E-43.5 CONTROL: mutant accepting any signed request revokes another agent's key.

### 6.6 Blobs and per-agent encryption keys (K-13, K-24; ARC4 6.4; B-11, B-12, B-13)

**E-44 — Blob round trip with both digests; STORED only after the plaintext is on disk.** Links: R-11, R-15, R-80, SI-28, FX-5, AFC-23. Category: core function. Level: L4. Tier: 2. Phase: PH-2 (B-12).
E-44.1 EVIDENCE: a file of 1, 5 and 16 MiB sent by path is encrypted to the recipient, stored as ciphertext in the recipient's home hub store, fetched in 256 KiB chunks, checked against `cipher_sha256`, decrypted, checked against `plain_sha256`, written to `<store>/blobs/<plain_sha256>` (0600) by temp + fsync + rename, and only then reported STORED (ARC4 6.4.3, 6.4.4).
E-44.2 ARRANGE: throwaway hub; `fx-a` and `fx-b` with signing and X25519 keys in the estate register.
E-44.3 ACT: send each size; send 16 MiB + 1 byte.
E-44.4 ASSERT: received file equals the sent file byte for byte; STORED stage time is after the rename (stage log order); file mode 0600; 16 MiB + 1 refused at the sender before upload with the PN-11 reason.
E-44.5 CONTROL: mutant reporting STORED before the plaintext digest check fails the order assertion (pattern of ARCH `fc-03-stored-line-before-the-record`).

**E-45 — A partial or corrupt blob is never success.** Links: R-11, R-15, R-42, AFC-23. Category: error handling. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-12).
E-45.1 EVIDENCE: a corrupted chunk, a missing chunk, a wrong wrapped key or a plaintext digest mismatch yields not STORED, a signed refusal to the sender and a needs-attention entry (the T-3661 lesson).
E-45.2 ARRANGE: a test proxy in front of `artifact get` that flips a byte, drops a chunk, truncates; a message with a key wrapped to another agent.
E-45.3 ACT: deliver each case.
E-45.4 ASSERT: no file in `blobs/`, no temp file left, no STORED stage; refusal notice and entry present per case.
E-45.5 CONTROL: mutant that skips `cipher_sha256` verification stores the flipped-byte case.

**E-46 — A peer-supplied name is never a file name (SI-28, fuzz).** Links: SI-28, TH-57, R-11, AFC-23. Category: adversarial. Level: L3. Tier: 1. Phase: PH-2 (B-12).
E-46.1 EVIDENCE: whatever `name_hint` holds, the blob is stored only under its digest in the private directory.
E-46.2 ARRANGE: generator of `name_hint` values: `../x`, `../../etc/passwd`, absolute paths, NUL bytes, 4 KiB names, Unicode slashes, `.`, `..`, symlink names pre-created in the store.
E-46.3 ACT: store at least 10^4 blobs with generated hints.
E-46.4 ASSERT: every file created is `<store>/blobs/<64 hex>`; no file or link created outside; the hint appears only as display text.
E-46.5 CONTROL: mutant "store under name_hint" (ARC4 6.4.7) writes outside the directory.

**E-47 — The hub holds ciphertext only (FX-5 probe).** Links: FX-5, R-80, TH-22, BP-21, AFC-23. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-11).
E-47.1 EVIDENCE: a hub-token holder reading the artifact store finds no arc-011 plaintext: every new object starts with the wrapper's magic and version; a writer without the wrapper is refused; message fields have a plaintext size cap so content cannot be tunnelled (ARC4 6.4.5).
E-47.2 ARRANGE: throwaway hub; blobs containing a known 32-byte canary string; the FX-5 baseline of 9 allow-listed writers.
E-47.3 ACT: send blobs; read the whole store with a hub token; try a raw `artifact put` of plaintext through the arc-011 path; post a message with an over-cap inline field.
E-47.4 ASSERT: the canary is found in no object; every arc-011 object has the wrapper header; the raw put is refused; the over-cap field is refused; the writer inventory still lists 9 plus the wrapper (`scripts/check-arc011-inventory.sh` green).
E-47.5 CONTROL: mutant that uploads plaintext makes the canary search hit.

**E-48 — Encryption key register: mismatch refused; one agent's key opens only its blobs.** Links: R-80, FX-5, AFC-24. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-11).
E-48.1 EVIDENCE: a sidecar whose private X25519 key does not match its root-owned register entry refuses to start; a blob wrapped to A cannot be opened with B's key (ARC4 6.4.2, 15 K-24).
E-48.2 ARRANGE: estate register with entries for `fx-a`, `fx-b`; a swapped entry variant.
E-48.3 ACT: start with the swapped entry; decrypt A's blob with B's key.
E-48.4 ASSERT: start refused with both fingerprints in the reason; decrypt fails authenticated decryption (no partial plaintext written).
E-48.5 CONTROL: mutant without the start-up comparison starts.

**E-49 — `artifact put --hub` needs `--encrypt-to`; plaintext cross-hub put refused.** Links: FX-5, BP-21, R-80, DR-5. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-13).
E-49.1 EVIDENCE: as ruled O-1 A, `--hub <addr>` without `--encrypt-to <agent>` is refused with a reason naming the rule and the `--encrypt-to` form; with it, only ciphertext reaches the remote store (ARC4 6.4.6).
E-49.2 ARRANGE: two throwaway hubs (EP-3); a recipient key in the register.
E-49.3 ACT: put a canary file with `--hub` only; with `--hub --encrypt-to`.
E-49.4 ASSERT: first exits non-zero with the reason text and leaves nothing at the remote store; second leaves one wrapped object; canary absent from the remote store.
E-49.5 CONTROL: mutant that downgrades to plaintext on a missing key fails the canary search. Existing routes (`file send`, ssh, SMB) are not exercised here: out of scope (ARC4 6.4.6.a, T-3663).

### 6.7 Caps, quotas, hop limit, adaptive window (K-11; ARC4 6.3; B-7)

**E-50 — Receiver limits, in order, with nothing dropped.** Links: SI-19, R-56, TH-28, TH-30, TH-36. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-7).
E-50.1 EVIDENCE: at STORED time the receiver checks inline ≤ 8 KiB, blob ≤ 16 MiB, per-sender open ≤ 32, aggregate ≤ 256 and ≤ 64 MiB, reserved shares 10% receipts and 20% own project; over a limit the message is not STORED, a signed `QUOTA` notice names the limit, one coalesced needs-attention entry per sender per tick appears, and nothing is dropped (ARC4 6.3.1, 6.3.2).
E-50.2 ARRANGE: estate EP-2; a sender that does not stop at `QUOTA`.
E-50.3 ACT: send 8 KiB and 8 KiB + 1 inline; 33 open messages from one sender; fill the aggregate to 256.
E-50.4 ASSERT: refusals at exactly the stated limits with the limit named; each refused message is still in the sender's outbox; entries coalesced (one per sender per tick).
E-50.5 CONTROL: mutant that drops over-limit messages at the sender loses them from the outbox.

**E-51 — Many honest senders cannot starve receipts or the own project (SI-19).** Links: SI-19, R-56, TH-28, TH-30. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-7).
E-51.1 EVIDENCE: a dishonest sender ignoring its own cap, and many admitted senders each within quota up to the aggregate ceiling, still leave room for receipts and own-project mail (TM SI-19 probe).
E-51.2 ARRANGE: 20 foreign senders each with 31 open messages; one dishonest sender; own-project sender; receipts flowing.
E-51.3 ACT: fill to the ceiling; then send an own-project message and receipts.
E-51.4 ASSERT: own-project message STORED; every receipt processed; receipts never counted against a sender quota.
E-51.5 CONTROL: mutant counting receipts against the quota stops the receipts (ARC4 6.3.6).

**E-52 — Sender window AIMD and the 101st message.** Links: R-56, AFC-5. Category: core function. Level: L1. Tier: 1. Phase: PH-2 (B-7).
E-52.1 EVIDENCE: the window starts at 100, rises +5 per tick with receipts advancing and no `QUOTA`, to 400; halves on `QUOTA`, floor 10; overflow waits with a needs-attention entry (V-4).
E-52.2 ARRANGE: property generator of tick sequences with receipt progress and `QUOTA` notices.
E-52.3 ACT: run the window function on at least 10^4 sequences.
E-52.4 ASSERT: window always in [10, 400]; equals the AIMD recurrence; message 101 in the first tick waits, with one entry.
E-52.5 CONTROL: mutant without the floor reaches 0 and deadlocks the sender.

**E-53 — Hop limit stops automatic loops; receipts answer nothing.** Links: R-56, SI-19, TH-29. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-7).
E-53.1 EVIDENCE: deliberate content starts at hop 0, automatic content and forwards add 1, a receiver refuses hop > 3 with a notice; `hop` is inside the signature (ARC4 6.3.4).
E-53.2 ARRANGE: two fake agents that auto-reply to everything; a message with `hop` edited after signing.
E-53.3 ACT: send one deliberate message; deliver the edited message.
E-53.4 ASSERT: the loop ends at hop 3 with a refusal notice (at most 4 automatic messages); the edited message fails signature verification; no receipt produces a message.
E-53.5 CONTROL: mutant with `hop` outside the signed payload accepts the edited message.

**E-54 — Jitter and early re-send refusal.** Links: R-56, R-30. Category: edge case. Level: L1. Tier: 1. Phase: PH-2 (B-7, B-9).
E-54.1 EVIDENCE: each ladder rung waits its interval × U[0.8, 1.2]; a re-send earlier than 0.8 of the rung is refused by the sender's own ledger (ARC4 6.3.5).
E-54.2 ARRANGE: seeded generator of rung schedules.
E-54.3 ACT: 10^4 re-send attempts at random times.
E-54.4 ASSERT: every accepted re-send is at ≥ 0.8 × rung; every earlier attempt refused and recorded; the empirical factor distribution covers [0.8, 1.2] (Kolmogorov–Smirnov p > 0.01 against uniform) [P].
E-54.5 CONTROL: mutant without the ledger check accepts an early re-send.

### 6.8 Status, CLEAR, reachability card, strength flags (K-16; ARC4 6.7; B-14)

**E-55 — Status fields present, stamped, and fast under a fault.** Links: R-6, R-29, R-53, R-70, FX-8. Category: core function. Level: L2 + L4. Tier: 1 and 3. Phase: PH-2 (B-14).
E-55.1 EVIDENCE: status returns `mode`, `binary`, `supervision`, `main`, `hub {id, declared, match}`, `revocation`, `quota`, `consumers`, `reachability`, `clear`, `strength {…}` and `activity`, each with a freshness stamp; a stale field reads unknown; p95 ≤ 5 s under each injected fault of E-63 (ARC4 6.7.1).
E-55.2 ARRANGE: tier 1 estate; tier 3 EP-5 with the installed artifact.
E-55.3 ACT: 100 status calls per fault.
E-55.4 ASSERT: JSON schema check of every field; p95 latency ≤ 5 s per fault; stale fields `unknown`.
E-55.5 CONTROL: mutant returning a cached field without a stamp fails the schema check.

**E-56 — CLEAR is never a fresh heartbeat alone (FX-8; standing control INC-d).** Links: FX-8, R-29, R-53, TH-23. Category: adversarial. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-14).
E-56.1 EVIDENCE: a sidecar with a seconds-old heartbeat that refuses every confirmation reads NOT CLEAR with the field bad; CLEAR needs, when anything was pending in the last hour, a real surfacing after it (ARC4 6.7.4).
E-56.2 ARRANGE: the 32,205-refusal injection (a receiver whose confirmations are all refused) with a fresh heartbeat; a healthy receiver.
E-56.3 ACT: send mail; read status.
E-56.4 ASSERT: injected: `clear: NOT CLEAR`, `reachability.last_surface` bad; healthy: CLEAR after the hand-over.
E-56.5 CONTROL: mutant "heartbeat counts as surfacing" reads CLEAR in the injected case (ARC4 6.7.5).

**E-57 — The no-surfacing threshold (V-8) and UNKNOWN, never DEAD.** Links: R-53, R-29, R-44, FX-8. Category: edge case. Level: L2. Tier: 1. Phase: PH-2 (B-14).
E-57.1 EVIDENCE: "last surface time" turns bad when a STORED message is not handed over 2 ticks after the agent became READY or after the first hook call that did not hand it over; an idle agent with nothing pending is never bad by age; a silent back channel reads UNKNOWN, never DEAD (ARC4 6.7.3).
E-57.2 ARRANGE: simulated clock; idle agent for 48 h with nothing pending; then a STORED message; then a cut back channel.
E-57.3 ACT: tick through each case.
E-57.4 ASSERT: idle 48 h: not bad, age shown; bad exactly at READY + 2 ticks without hand-over; cut channel: UNKNOWN.
E-57.5 CONTROL: mutant with an age threshold on the idle agent turns it bad.

**E-58 — Strength flags tell the truth about the host.** Links: SI-14, R-82, FP 3.1.e, TH-25. Category: adversarial. Level: L4. Tier: 3. Phase: PH-2 (B-14).
E-58.1 EVIDENCE: `strength` flags state absent strength honestly: `per_agent_accounts: no`, `same_user_separation: "absent: agents run as root"` while agents run as root, `operator_key: no`, `confirmation_provider: no`, `policy_records: "none (fixed defaults)"` (ARC4 6.7.1; status honesty).
E-58.2 ARRANGE: EP-5 and the W1 pair host.
E-58.3 ACT: read status; read the process owner of every agent and sidecar (`ps -o user=`).
E-58.4 ASSERT: each flag equals what the host shows; a flag claiming strength that the host lacks is FAIL.
E-58.5 CONTROL: a fixture that fakes `per_agent_accounts: yes` on a root host fails the comparison.

**E-59 — Skew and the observer's own clock.** Links: R-57, SI-9, TH-37, T-16. Category: edge case. Level: L2. Tier: 1. Phase: PH-2 (B-14).
E-59.1 EVIDENCE: deadlines use only the observer's monotonic clock; the skew estimate from hub round trips flags "clock skew" above 2 s and marks cross-host delay figures unusable above 30 s; NTP state is shown or "unchecked" (V-5).
E-59.2 ARRANGE: hub stub that reports wall times offset by 0, 3, 45 s and jumps the sidecar's wall clock by ±1 h.
E-59.3 ACT: run deadlines and status across the jumps.
E-59.4 ASSERT: no deadline fires early or late by the wall jump; flags at 3 s and 45 s as stated; NTP field `unchecked` when `timedatectl` and `chronyc` are absent.
E-59.5 CONTROL: mutant using wall time for a deadline fires at the jump.

**E-60 — GP-4 offline behaviour is visible, never silent.** Links: R-7, R-44, R-45, TH-35. Category: error handling. Level: L4. Tier: 2. Phase: PH-2 (B-14).
E-60.1 EVIDENCE: receiver host offline: the sender sees WAITING FOR RECIPIENT after the liveness card expires (600 s), not STUCK; sender without network: `send` returns an id, the message is durable in the outbox, the sender sees WAITING "home hub unreachable since T", nothing claims RECEIVED; on return the sender reads the hub record first and posts what is missing (ARC4 8.2).
E-60.2 ARRANGE: throwaway hub; network namespace or firewall rule cutting a sidecar.
E-60.3 ACT: cut the receiver for 30 min; separately cut the sender, send 5, restore.
E-60.4 ASSERT: states as stated with the "since T"; after restore no duplicates on the hub (identity triple) and all 5 present.
E-60.5 CONTROL: mutant that reports RECEIVED from the local outbox fails.

**E-61 — Reachability card content and readers.** Links: R-53, TH-23, CR-11. Category: core function. Level: L4. Tier: 2. Phase: PH-2 content (B-14); read scope enforced PH-3.
E-61.1 EVIDENCE: each agent's card on its home hub carries receiver up, right hub, adapter present, last surface time (a real HANDED_OVER or confirmation, never a heartbeat); other projects' readers see only reachable yes/no and the version class once the PH-3 read scope exists (ARC4 6.7.2).
E-61.2 ARRANGE: throwaway hub, one agent with deliveries, one without.
E-61.3 ACT: read the cards; (PH-3) read as another project.
E-61.4 ASSERT: four fields with values matching the stage log; `last_surface` equals the latest HANDED_OVER time, never a heartbeat time; PH-3: foreign read shows two fields only.
E-61.5 CONTROL: mutant writing heartbeat time into `last_surface` fails.

### 6.9 Fault injectors and the drill (K-17; ARC4 6.8; B-15)

**E-62 — Each injector has a negative control (meta-evidence).** Links: R-69, R-6, GP-12, T-10. Category: integration. Level: L2. Tier: 1. Phase: PH-2 (B-15).
E-62.1 EVIDENCE: for each of the six faults of ARC4 6.8.1, a run with the diagnosis code removed fails, so a green injector run is not vacuous.
E-62.2 ARRANGE: `scripts/arc011-fault-inject.sh <fault> --tier 1` with a build that removes the diagnosis for that fault.
E-62.3 ACT: run all six.
E-62.4 ASSERT: six FAIL verdicts, each naming the missing diagnosis.
E-62.5 CONTROL: this spec is the control of E-63.

**E-63 — Six faults named with their fix within 60 s, 100 of 100, zero manual recovery.** Links: R-6, R-70, R-29, R-49, R-54, FX-2, FX-8, SI-16, H-18, H-19. Category: error handling. Level: L6. Tier: 1 and 3. Phase: PH-2 (B-15).
E-63.1 EVIDENCE: (1) signing key filed under another agent, (2) second inbox consumer, (3) receipts under another identity, (4) a dead component (tier 1 one loop; tier 3 SIGSTOP of the sidecar), (5) a second hub or wrong runtime directory, (6) unsurfaced mail (hooks removed) are each named by status with cause and fix within 60 s (30 s for start-up checks), and recovery needs zero manual steps once the fault is removed (ARC4 6.8.2).
E-63.2 ARRANGE: tier 1 simulated clock; tier 3 EP-5 on the installed artifact.
E-63.3 ACT: 20 injections of each fault per tier.
E-63.4 ASSERT: 120 of 120 per tier named within the bound with the documented fix text; after removal, status returns to its pre-fault values without any operator step. Result: per-fault counts and latency distribution.
E-63.5 CONTROL: E-62.

**E-64 — The drill: first-time readers diagnose from status alone.** Links: R-6, GP-12, H-18. Category: user workflow. Level: L5. Tier: 4. Phase: PH-2 (B-15).
E-64.1 EVIDENCE: three readers who have not seen the system, given only status output of an injected fault, name the cause correctly with median ≤ 5 min.
E-64.2 ARRANGE: three status captures from E-63 (different faults); readers are agents in fresh sessions with no repository access, or people the operator names.
E-64.3 ACT: each reader answers "what is wrong and what do you do".
E-64.4 ASSERT: all three correct; median time ≤ 5 min; answers stored unedited.
E-64.5 CONTROL: one capture with the cause line removed MUST lead to a wrong or "cannot tell" answer, else the drill is not measuring the status text.

**E-65 — H-19: 500 process kills, every one recovers or is visible.** Links: R-8, H-19, SI-15. Category: error handling. Level: L6. Tier: 3. Phase: PH-2 (built supervisor; re-run with B-15).
E-65.1 EVIDENCE: after 500 injected kills of sidecar components, ≥ 99% recover within the supervision interval and the rest end in a visible needs-attention entry; none silent.
E-65.2 ARRANGE: EP-5 with systemd units and the cron backstop as installed.
E-65.3 ACT: 500 kills at random times and signals.
E-65.4 ASSERT: per kill: recovered within interval, or entry present; 0 silent.
E-65.5 CONTROL: with the backstop disabled and the unit's restart off, kills produce silent cases and the run fails.

### 6.10 Hub changes (K-15; ARC4 6.6; B-2, B-3, B-4, B-6)

**E-66 — A second inbox consumer is flagged within 60 s, refused after bind.** Links: SI-16, R-49, BP-15, DR-11, R-60. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-2).
E-66.1 EVIDENCE: the hub records per inbox every identity that subscribed, acked or claimed (16 kept, eviction logged); a second identity within 24 h (V-11) shows in status as "inbox consumed by two identities: A, B" and needs-attention within 60 s; after `admin inbox bind`, ack and claim from any other identity are refused; a second live copy of a project on the same hub is seen within one tick (V-12) (ARC4 6.6.1, 6.6.2).
E-66.2 ARRANGE: throwaway hub; sidecar `fx-a` consuming its inbox; a raw client with identity `fx-z`.
E-66.3 ACT: `fx-z` subscribes and acks `fx-a`'s inbox; then bind to `fx-a` and repeat; register 17 identities; register a second copy of `fx-a`'s project.
E-66.4 ASSERT: flag within 60 s naming both; after bind `fx-z` refused with a reason and an admin-log row; eviction of the oldest logged; second copy flagged within one tick.
E-66.5 CONTROL: mutant that records only subscribers (not ackers) misses `fx-z` (the BP-15 route).

**E-67 — Admin RPCs are logged; a sweep of open mail is refused; retention ceilings.** Links: FX-6, BP-7, R-35, TH-30, SI-24. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-3).
E-67.1 EVIDENCE: every admin-scope RPC appends caller, method, params digest, result with `admin: true`, kept ≥ 30 days; `channel.sweep`, `set-retention` and trim on an `inbox:` or conversation topic are refused when they would remove an offset whose message is not final; past a V-3 ceiling the oldest final records go first, loudly, and open ones only as a last resort with needs-attention (ARC4 6.6.3, V-3).
E-67.2 ARRANGE: throwaway hub; an inbox with final and open messages; ceilings shrunk by a test seam.
E-67.3 ACT: sweep, set-retention, trim across the open offset; call each admin RPC once; exceed the ceiling.
E-67.4 ASSERT: refusals with reason; one admin row per call; the log rotation keeps `admin: true` rows ≥ 30 days (rotation simulated); ceiling: final removed first, entry raised, no open message removed while a final one remains.
E-67.5 CONTROL: mutant without the open-offset check removes an open message.

**E-68 — Founding verbs need their own scope, are logged, and can be owner-only.** Links: FX-4, R-4, BP-1, BP-2, TH-2. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-4).
E-68.1 EVIDENCE: `inject` needs `control` and `exec` needs `execute` (built); every `command.inject`, `pty inject`, `command.execute` and `remote exec` is logged with the caller at the session and at the hub; a session spawned `--owner-only` refuses other callers; R-4's ability to inject and read back stays (ARC4 6.6.4).
E-68.2 ARRANGE: EP-4 session; tokens with `observe`, `notify`, `control`, `execute`; a second identity.
E-68.3 ACT: call each verb with each token and identity.
E-68.4 ASSERT: refusals with `observe` and `notify`; every attempt (allowed or refused) has a log row at session and hub with caller identity; owner-only refuses the second identity; the owner can still inject and read output back.
E-68.5 CONTROL: mutant with no hub-side log row fails the row count.

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

### 6.11 Vendor channels and the SendMessage deny (K-20; ARC4 6.10; B-18, B-26)

**E-71 — Every vendor-channel use leaves a row and an entry; the hook never blocks.** Links: FX-1, R-79, BP-14, R-47. Category: core function. Level: L2 + L4. Tier: 1 and 3. Phase: PH-2 (B-18 log half).
E-71.1 EVIDENCE: a PreToolUse call for `SendMessage`, `Agent` or `Task` appends a row to the sidecar's admin chain and a routine needs-attention entry `vendor-channel-use`, coalesced per session per day; the hook exits 0 (ARC4 6.10.1).
E-71.2 ARRANGE: tier 1 hook fixture with the three tool names; tier 3 EP-5 with the hook installed.
E-71.3 ACT: call each tool twice in one session.
E-71.4 ASSERT: six rows, three entries (coalesced); each tool call completes.
E-71.5 CONTROL: mutant hook matching `SendMessage` only misses the sub-agent rows.

**E-72 — The SendMessage deny is an idempotent operator action that waits for the re-based prover.** Links: FX-1, R-79, BP-14. Category: user workflow. Level: L2. Tier: 1. Phase: PH-2 (B-18 deny half).
E-72.1 EVIDENCE: the runme action checks for the `permissions.deny` entry, adds it, verifies it on disk, reports FAILED otherwise, and refuses to add it until B-26's prover has passed once on the TermLink rail (ARC4 6.10.2, 6.10.2.a).
E-72.2 ARRANGE: `tests/runme-fixtures.sh` with `RUNME_*` seams; a fake settings file; a fake prover result absent, failed, passed.
E-72.3 ACT: run the action three times per prover state.
E-72.4 ASSERT: prover absent or failed: action refuses, settings unchanged, log line FAILED with the reason; passed: first run adds the entry and verifies (OK), second and third `skip`; a corrupted settings file after the write gives FAILED.
E-72.5 CONTROL: mutant that trusts the write's exit code without re-reading misses the corruption case.

**E-73 — After the deny, SendMessage is refused by the harness; what the log sees is recorded.** Links: FX-1, BP-14. Category: integration. Level: L4. Tier: 3. Phase: PH-2 (B-18).
E-73.1 EVIDENCE: with the deny in place a SendMessage call in a session is refused by the harness; whether the refused call still reaches the PreToolUse log is observed and recorded, not assumed (ARC4 6.10.3).
E-73.2 ARRANGE: EP-5 session with a project settings copy that has the deny.
E-73.3 ACT: the agent attempts a SendMessage.
E-73.4 ASSERT: the tool result is a refusal; the record states `logged: yes|no` as observed.
E-73.5 CONTROL: same session without the deny: the call is not refused (otherwise the spec is vacuous).

**E-74 — The session message prover works on the TermLink rail.** Links: FX-1, R-24, R-42, R-79. Category: integration. Level: L4 + L5. Tier: 1 and 4. Phase: PH-2 (B-26).
E-74.1 EVIDENCE: `scripts/session-message-selftest.sh`, re-based on the sidecar send API (or `termlink agent send`), keeps assert-on-the-receiver: DELIVERED (0), BLOCKED (1), ENQUEUED (1), UNDELIVERED (1), TOOLING (2), with its fixtures and mutants (CLAUDE.md T-2876 section).
E-74.2 ARRANGE: tier 1 its fixture suite with `SESSION_MSG_TEST_*` seams; tier 4 EP-7.
E-74.3 ACT: run the suite; live: send a sentinel over TermLink and assert on the receiver's transcript; send a control sentinel that was never sent.
E-74.4 ASSERT: suite green with its pinned mutants red; live DELIVERED rc 0 and control UNDELIVERED rc 1; an unreadable receiver gives TOOLING rc 2.
E-74.5 CONTROL: the suite's pinned mutants (removing the BLOCKED branch, the reachability gate, collapsing ENQUEUED into UNDELIVERED) stay red after the re-base.

### 6.12 W3 acceptance, soak, canary and the PH-2 exit (ARC4 6.11; B-19, B-21, B-22, B-23)

**E-75 — Two-agent live test with a negative control on the installed artifact.** Links: R-2, R-3, R-14, R-15, R-24, R-26, R-27, R-44, R-45, R-66, R-69, T-4a. Category: user workflow. Level: L5. Tier: 4. Phase: PH-2 (B-22; ARCH2 exit 2).
E-75.1 EVIDENCE: a round trip between two real agents through the installed artifact shows the five stage rows with times (RECEIVED, STORED, HANDED_OVER with transcript evidence, REPLIED, and the reply's own stages) to the sender; with the hooks removed from the receiver (fault 6) the sender sees a visible non-delivered state by its deadline.
E-75.2 ARRANGE: EP-9 (two agents of 010 on .107 via the reachable launcher), `scripts/arc011-ph1-demo.sh` in installed-artifact mode.
E-75.3 ACT: A asks B a question; B replies; repeat with B's hooks removed.
E-75.4 ASSERT: run 1: five rows, HANDED_OVER `record_ref` resolvable in B's transcript, REPLIED linked to the conversation; run 2: no HANDED_OVER, sender state WAITING FOR RECIPIENT, STUCK or UNKNOWN by the deadline, never "delivered".
E-75.5 CONTROL: run 2 is the negative control; it MUST end not-delivered.

**E-76 — "No action" is a visible terminal stage.** Links: R-28, R-44, T-4b. Category: user workflow. Level: L5. Tier: 4. Phase: PH-1 built; re-run in PH-2 (B-22).
E-76.1 EVIDENCE: a woken agent that has nothing to do ends the message ACKNOWLEDGED_NO_ACTION, visible to the sender; it never stays silent.
E-76.2 ARRANGE: EP-7.
E-76.3 ACT: send an informational message that needs no answer.
E-76.4 ASSERT: sender sees ACKNOWLEDGED_NO_ACTION within the deadline.
E-76.5 CONTROL: an agent session with the no-action path removed (fixture) leaves the message owed and the sender sees STUCK.

**E-77 — H-14 soak: the approach works for real agents (REFUTES THE APPROACH).** Links: R-1, R-2, R-3, R-24, R-42, R-44, FX-7, H-14, T-21. Category: integration. Level: L7. Tier: 4. Phase: PH-2 (B-22).
E-77.1 EVIDENCE: over 14 days and at least 500 messages on EP-9, at least 95% of the denominator (messages to an agent that was READY, or whose hook fired, within twice the R-44 deadline after STORED) reach HANDED_OVER with transcript evidence in time; 100% of the rest show a sender state by their deadline; zero HANDED_OVER without evidence; zero STORED messages missing (FP 9 H-14).
E-77.2 ARRANGE: stage records, transcripts and ring logs retained 14 days.
E-77.3 ACT: real work.
E-77.4 ASSERT: report with numerator and denominator per attempt and per message (RNU, ARC4 15.2.a); any false HANDED_OVER or silent loss that survives one fix cycle is FAIL and triggers the operator brief.
E-77.5 CONTROL: the report generator runs on a seeded week with one false HANDED_OVER and MUST fail.

**E-78 — Daily canary from the installed artifact, with staleness.** Links: R-66, R-69, T-12, V-13. Category: integration. Level: L7. Tier: 4. Phase: PH-2 (B-19, T-3361).
E-78.1 EVIDENCE: once a day the canary pair EP-8 completes RECEIVED, STORED, HANDED_OVER in the receiver's transcript and REPLIED within the R-44 deadlines, with a negative control; a missing run for 26 h is detected by the canary meta-check; it re-runs after a rotation, a re-vendor and a reboot.
E-78.2 ARRANGE: testdev cron via runme; the canary meta-check (T-1723).
E-78.3 ACT: daily; stop the canary for 27 h once.
E-78.4 ASSERT: daily log rows with the four stages; control red each day; staleness alarm at 26 h; re-run entries after each event.
E-78.5 CONTROL: the daily negative control (a message to a receiver with hooks removed) MUST be red.

**E-79 — Standing controls of the three real failures.** Links: R-69, R-29, R-54, FX-8, T-11, H-15. Category: adversarial. Level: L2 + L4. Tier: 1 and 2. Phase: PH-0 red, green from PH-2, per release.
E-79.1 EVIDENCE: INC-a (2026-10-03 mail stored and receipted, never surfaced), INC-b (the stray hub), INC-d (the 32,205-refusal sidecar) are each detected: red before the fix, green after, red again on revert.
E-79.2 ARRANGE: the three fixtures of H-15.
E-79.3 ACT: run per release on the release commit and on the commit with each fix reverted.
E-79.4 ASSERT: green on release; red with each revert, for its named reason.
E-79.5 CONTROL: the revert runs are the controls.

**E-80 — Installed system: clean host, upgrade, rollback, restart, reboot, mixed version.** Links: R-39, R-8, SI-27, TH-17, T-8, T-9, T-15. Category: integration. Level: L6. Tier: 3. Phase: PH-2 (built P9; re-run per release).
E-80.1 EVIDENCE: the release artifact installs on a clean host, upgrades, rolls back and restarts with the runtime directory persisted (PL-021); after a reboot the sidecar is back and the canary clean; checksums are verified in every status call; a mixed-version pair is refused loudly, not dropped.
E-80.2 ARRANGE: `tests/arc011-clean-host.sh`; EP-12 VM; two binaries of adjacent versions on EP-3.
E-80.3 ACT: install, upgrade, rollback, restart, reboot; replace the binary with a modified copy; pair old and new.
E-80.4 ASSERT: each step's status green; `hub.id` and secrets unchanged across reboot; modified binary flagged by status (SI-27); mixed pair refused with a reason naming both versions.
E-80.5 CONTROL: a runtime directory on a volatile path (PL-021) MUST give a new hub id after reboot and fail E-36's keep rule, proving the check sees it.

**E-81 — Rollback keeps and lists STORED messages (OQ-j A, XR-1).** Links: R-15, R-49, SI-15. Category: error handling. Level: L4. Tier: 2. Phase: PH-2 (built K-4; re-run per wave).
E-81.1 EVIDENCE: a wave rollback by the K-4 drain keeps every STORED message in the store and lists it in the rollback report; the count compare between old and new receivers matches.
E-81.2 ARRANGE: throwaway hub; an agent with STORED, ATTEMPTED and HANDED_OVER messages.
E-81.3 ACT: run the drain and rollback.
E-81.4 ASSERT: report lists every non-final message; counts equal; none re-posted as bare (the signature binds the topic).
E-81.5 CONTROL: a drain that skips the count compare MUST be refused by the runme action.

**E-82 — W4 decommission refuses without the operator's recorded approval (NID).** Links: R-49, BP-16, NID. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-23).
E-82.1 EVIDENCE: the W4 runme action switches AEF's receiver and the notify-sidecar off for .107 inboxes only when the W4 task's Decisions contain the operator's approval in the operator's own words; otherwise it refuses and changes nothing (ARC4 6.11.3).
E-82.2 ARRANGE: `tests/runme-fixtures.sh` with a fake task file with and without the approval record, and a fake unit state.
E-82.3 ACT: run the action in both states, twice each.
E-82.4 ASSERT: without approval: FAILED with the reason, units untouched; with approval: units off and verified on disk, second run `skip`; switched-off rails remain installed.
E-82.5 CONTROL: mutant that reads the approval from the task's status field instead of its Decisions runs without the record and fails.

**E-83 — One receiver per inbox after W4, and bypass routes do not grow.** Links: R-49, SI-16, H-26, BP-15, BP-16. Category: integration. Level: L7. Tier: 4. Phase: PH-2 exit (B-22).
E-83.1 EVIDENCE: over the V-11 window after W4 each .107 inbox shows one consumer identity; at each phase exit the TM section 17 bypass inventory is re-run and no route is newly open (H-26).
E-83.2 ARRANGE: hub consumer lists; the bypass inventory script.
E-83.3 ACT: read lists daily; run the inventory at the exit.
E-83.4 ASSERT: one identity per inbox for 24 h; inventory diff shows no new open route and BP-15 moved to removed.
E-83.5 CONTROL: inject a second consumer (E-66) during the window; the check MUST fire.

### 6.13 The adapter contract `termlink.adapter/2` (ARC4 17; B-29, AEF T-5054)

**E-84 — Parity fixtures per report, with negative controls, against every adapter.** Links: R-47, R-21, R-24, R-50, R-51, R-79, GP-8, T-19, AFC-14. Category: integration. Level: L2. Tier: 1. Phase: PH-2 (fixtures with B-25/B-27; AEF adapter B-29).
E-84.1 EVIDENCE: one scenario file per report (R1 readiness, R2 turn events, R3 prompt origin, R4 evidence and complete point, R5 framing and waiting-mail definition, R6 vendor channels) runs against every adapter per release with the same meaning; the negative controls fail: a doorbell line reported `human`, a missing READY read as ready, evidence without the nonce; a harness without an adapter is unreachable, never delivered (ARC4 17.9).
E-84.2 ARRANGE: TermLink's W1 hooks as the reference adapter; AEF's Claude Code adapter when T-5054 delivers; 055's opencode adapter when it exists.
E-84.3 ACT: replay each scenario through each adapter into a receiver in a test estate.
E-84.4 ASSERT: identical receiver-side outcomes per scenario across adapters; each negative control scenario FAILS for its adapter variant; "no adapter" yields unreachable status.
E-84.5 CONTROL: the three negative-control scenarios are the controls; an adapter that passes them is defective.

**E-85 — Prompt-origin bases are reported as the contract says.** Links: R-47, R-67, AR-6. Category: adversarial. Level: L4. Tier: 3. Phase: PH-2 (B-29).
E-85.1 EVIDENCE: `automated` is reported for a doorbell line (basis `doorbell`), a non-interactive session (`non-interactive`), input that reached the PTY by `inject` or `notify`; `human` only with basis `pty-input` from client input or `inferred`; `unknown` otherwise (ARC4 17.5).
E-85.2 ARRANGE: EP-5 under a TermLink PTY; a print-mode session; an injected prompt.
E-85.3 ACT: submit a prompt by each route.
E-85.4 ASSERT: origin and basis per route as stated; the `inferred` share is reported (AR-6 measured, not closed).
E-85.5 CONTROL: an adapter variant that reports every prompt `human` MUST fail the doorbell and inject routes.

## 7 PH-2 rows that already have tasks (P12, P14, P16; B-8, B-9, B-10)

**E-86 — Sequence window: equivocation, out-of-window numbers, namespace across key rotation.** Links: SI-5, R-59, TH-11, TH-31. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-8, T-3362).
E-86.1 EVIDENCE: the same (sender identity, conversation, number) with another digest is refused and flagged as equivocation; numbers beyond the window (256) are refused; a key rotation keeps the namespace (TM SI-5 probe).
E-86.2 ARRANGE: estate EP-1.
E-86.3 ACT: send number 5 twice with different content; send 2^31; rotate `fx-a`'s key and resend number 5 with other content.
E-86.4 ASSERT: second 5 refused with a CONFLICT/equivocation entry and the first copy kept; 2^31 refused; post-rotation 5 refused as equivocation, not accepted as a fresh namespace.
E-86.5 CONTROL: the built mutant `fc-10-no-digest-comparison` accepts the second 5.

**E-87 — Gap wait (V-6): store at once, hold, resend request, bounded release.** Links: R-59, SI-5, SI-6, TH-31, H-3. Category: edge case. Level: L2. Tier: 1. Phase: PH-2 (B-8).
E-87.1 EVIDENCE: messages beyond a gap are STORED at once and held from hand-over; after 2 ticks a signed resend request for the gap is sent; after 10 ticks they are handed over marked "gap before N" with a needs-attention entry naming the gap; the largest held count is recorded for H-3.
E-87.2 ARRANGE: simulated clock; sender sends 1, 3, 4 (2 withheld).
E-87.3 ACT: tick 0..12; then deliver 2.
E-87.4 ASSERT: 3 and 4 STORED at tick 0 and not handed over; resend request at tick 2; release with the mark at tick 10; 2 handed over on arrival; `max_held` metric = 2.
E-87.5 CONTROL: mutant without the release bound holds 3 and 4 forever (DR-12 option B).

**E-88 — Receipts are signed, contiguous and believed only when valid.** Links: SI-6, R-45, TH-12, TH-15. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-8).
E-88.1 EVIDENCE: `up_to` is the highest contiguous durably stored number; a forged receipt with `up_to` beyond the stored number is not believed and the sender keeps chasing (TM SI-6 probe).
E-88.2 ARRANGE: estate; receiver stores 1 and 3.
E-88.3 ACT: read the receipt; post a forged receipt `up_to: 5` signed by a non-recorder key.
E-88.4 ASSERT: genuine receipt says 1; forged one refused; sender ladder continues for 2..5.
E-88.5 CONTROL: mutant computing `up_to` as the highest stored number says 3.

**E-89 — Ladders and the owed list: never dropped silently.** Links: R-18, R-25, R-28, R-30, R-31, R-52, R-58, SI-18. Category: core function. Level: L2. Tier: 1. Phase: PH-2 (B-9, T-3358).
E-89.1 EVIDENCE: an unanswered message is re-sent on each rung of its priority class and stays on the owed list until answered or ended; the owed count equals the hub record's each tick; the queue is read highest priority first, oldest first; priority is clamped at the receiver; an edited priority in the local queue file has no effect (ARC4 15 K-12; TM SI-18 probe).
E-89.2 ARRANGE: simulated clock; messages of priorities 9, 0, -9; a sender off the allow-list sending 9.
E-89.3 ACT: run both ladders for 24 simulated hours with no answer; edit the queue file's priority field.
E-89.4 ASSERT: re-sends at the rung times; owed list complete each tick and equal to the hub-record computation; hand-over order by priority then age; the off-list 9 delivered as normal with a recorded downgrade; the file edit ignored.
E-89.5 CONTROL: mutant that drops a message after the last rung empties the owed list silently.

**E-90 — Reconciliation on start: each outcome (RC-1..RC-6).** Links: SI-15, R-15, R-45, TH-14. Category: error handling. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-10).
E-90.1 EVIDENCE: on start, a local stage missing on the hub is written (idempotent); a hub STORED-or-later stage for a message the store lacks yields `lost-after-STORED`, needs-attention and a signed RESEND-REQUEST, and, if the sender lacks the payload, both sides show the loss; a different digest or chain link is a conflict kept on both sides as "unreconciled" for the operator; the label clears only after a clean pass (ARC4 6.5.2).
E-90.2 ARRANGE: throwaway hub; a store edited per case between stop and start.
E-90.3 ACT: start the sidecar per case; run twice.
E-90.4 ASSERT: per case the stated outcome; the second start adds nothing (idempotence); no silent merge.
E-90.5 CONTROL: mutant "merge a conflicting record silently" passes the conflict case and fails.

**E-91 — Daily head compare and the chain limits.** Links: SI-22, TH-14, TH-15, TH-19, TH-65, RR-15. Category: adversarial. Level: L2. Tier: 1. Phase: PH-2 (B-10).
E-91.1 EVIDENCE: removing a record in the middle, altering a digest, truncating inside the anchored prefix and rewriting the receiver's chain alone are detected on the next read or the daily compare; truncating only after the newest receipt and newest hub write, or rewriting chain, hub copy and receipt copy together, is the stated limit (TM SI-22 probe).
E-91.2 ARRANGE: estate with a 50-record conversation and receipts.
E-91.3 ACT: each edit, then read and run the compare.
E-91.4 ASSERT: first four: "unreconciled" fork entry; last two: verdict `LIMIT` naming RR-15 and the coordinated rewrite.
E-91.5 CONTROL: the built mutant `fc-15-no-chain-check` misses the middle removal.

**E-92 — Host crash on a real disk: STORED survives.** Links: SI-15, R-15, R-16, T-7, F-7. Category: error handling. Level: L6. Tier: 3. Phase: PH-2 (B-10).
E-92.1 EVIDENCE: a VM killed mid-write on a real disk image loses no message reported STORED, and every record that fails its checksum after boot is flagged (ARC4 6.5.4).
E-92.2 ARRANGE: EP-12 VM with the installed artifact; a sender outside the VM sending continuously.
E-92.3 ACT: 50 hard power-offs at random times during writes.
E-92.4 ASSERT: after each boot, every id the sender saw STORED is present and verifies; flagged records counted; 0 unflagged corruption.
E-92.5 CONTROL: a build without the fsync of the directory entry MUST lose a STORED message in at least one of 50 runs, else the test is not reaching the window (record as TOOLING if not).

## 8 Baseline re-assertion: built invariants that stay under evidence

8.1 These subjects are built and live (ARC4 4). They are not redesigned. They stay under evidence because every PH-2
item touches their code paths; the specs below are the regression gate each PH-2 build item MUST run.

**E-93 — PH-1 acceptance X1..X6 and the FC matrix stay green with their mutants red.** Links: R-2, R-5, R-14, R-15, R-16, R-24, R-42, R-45, R-51, R-69, FX-7, T-10, T-13. Category: core function. Level: L2 + L4. Tier: 1 and 2. Phase: PH-1 built; every push.
E-93.1 EVIDENCE: the built acceptance (`crates/termlink-sidecar/tests/acceptance.rs`, `fc_matrix.rs`) passes and every patch `fc-01`..`fc-19` turns its named test red under `scripts/arc011-mutants.sh`.
E-93.2 ARRANGE: the workspace at the commit under test.
E-93.3 ACT: `cargo test --workspace`; `bash scripts/arc011-mutants.sh --report <file>`.
E-93.4 ASSERT: tests exit 0; mutants exit 0 (every mutant killed); report lists 19 of 19 killed plus every new PH-2 mutant named in this document.
E-93.5 CONTROL: the mutants are the controls; a surviving mutant is exit 1.

**E-94 — Message identity and conflict (SI-4).** Links: SI-4, R-51, TH-9, TH-10, RR-10. Category: adversarial. Level: L2. Tier: 1. Phase: PH-1 built.
E-94.1 EVIDENCE: (sender identity, conversation id, message id) bound to a digest; the same identity with another digest is never stored or handed over and creates a CONFLICT; the first signed copy stays; after the stage memory ages past retention the conflict is not recognised (limit RR-10) (TM SI-4 probe).
E-94.2 ARRANGE: estate EP-2.
E-94.3 ACT: id X with A then B; B then A; A from a second sender key; aged stage memory.
E-94.4 ASSERT: first three as stated; aged case `LIMIT` naming RR-10.
E-94.5 CONTROL: `fc-10-no-digest-comparison`.

**E-95 — Framing (SI-7) cannot be closed or imitated by content.** Links: SI-7, R-50, TH-38, TH-39. Category: adversarial. Level: L3. Tier: 1. Phase: PH-1 built (other rails PH-4).
E-95.1 EVIDENCE: peer content is delivered only inside a frame with a per-delivery random boundary; content containing the frame markers or a copy of an earlier boundary leaves the frame intact.
E-95.2 ARRANGE: fuzz corpus of contents with marker strings, earlier boundaries, ANSI sequences, Unicode confusables.
E-95.3 ACT: frame at least 10^5 contents.
E-95.4 ASSERT: a parser of the frame recovers exactly the original content and the verified header fields for every case.
E-95.5 CONTROL: `fc-12-frame-without-boundary`.

**E-96 — Owner-only sockets and separate admin credential (SI-14).** Links: SI-14, TH-2, TH-43. Category: adversarial. Level: L4. Tier: 2. Phase: PH-1 built.
E-96.1 EVIDENCE: the send/status API answers only its owner; admin needs its own credential; another user's connect is refused.
E-96.2 ARRANGE: estate sockets; a second Unix user (`nobody`).
E-96.3 ACT: connect as `nobody`; call admin with the agent credential.
E-96.4 ASSERT: both refused; status still names "same-user separation absent" (E-58) because root is not stopped.
E-96.5 CONTROL: a socket created with mode 0666 in a fixture MUST let `nobody` connect, proving the probe reaches the check.

**E-97 — HANDED_OVER evidence (SI-17) and its limit.** Links: SI-17, R-24, TH-7, RR-7. Category: adversarial. Level: L2. Tier: 1. Phase: PH-1 built.
E-97.1 EVIDENCE: echoing the nonce in the agent's own output gives no HANDED_OVER; appending a well-formed harness-type record with the nonce as a same-user process is the stated limit RR-7.
E-97.2 ARRANGE: estate transcript writer.
E-97.3 ACT: echo case; forged-record case.
E-97.4 ASSERT: echo: no HANDED_OVER; forged record: `LIMIT` naming RR-7, never a pass of the control.
E-97.5 CONTROL: `fc-11-any-record-is-evidence`.

**E-98 — Atomic claim: one delivery per message across kills and concurrency (SI-30, H-16).** Links: SI-30, R-51, R-24, TH-52, TH-63, H-16, T-6. Category: error handling. Level: L6. Tier: 1. Phase: PH-1 built; re-run with B-16 (the doorbell does not bypass the claim).
E-98.1 EVIDENCE: with kills at every step between delivery and record, and two hook calls (or a hook and a doorbell-triggered hook) at once for one message, a message is handed over at most once; after restart an open attempt resolves by nonce or waits for the evidence-complete point or the operator.
E-98.2 ARRANGE: estate; 1000 randomised kill points across store, claim and record (H-16).
E-98.3 ACT: run the kill campaign; fire concurrent pulls.
E-98.4 ASSERT: 0 lost after STORED; 0 delivered twice in the transcript; every unresolved attempt visible.
E-98.5 CONTROL: `fc-04-no-claim-file` produces a double delivery.

**E-99 — Exact instance addressing and no cross-project fallback.** Links: SI-26, R-34, R-62, TH-54. Category: adversarial. Level: L4. Tier: 2. Phase: PH-1 built (dead letter E-70 in PH-2).
E-99.1 EVIDENCE: a message to an exact instance reaches only it and fails loudly if it ended; a `to_circuit` address never falls back to another project.
E-99.2 ARRANGE: throwaway hub; two projects with an agent of the same role name.
E-99.3 ACT: send to an ended instance; send to project P's role while only Q has a live agent of that role.
E-99.4 ASSERT: loud failure state; nothing reaches Q.
E-99.5 CONTROL: `fc-18-stranger-pass-taken-as-admission` and the R-62 standing test.

**E-100 — Release checksums in status and the "accepted/stored" wording.** Links: SI-27, FX-7, R-42, R-39, TH-17. Category: adversarial. Level: L4. Tier: 3. Phase: PH-2 built (P9).
E-100.1 EVIDENCE: every status call verifies the installed binaries against `checksums.txt`; no tool prints "delivered" without a recorded HANDED_OVER.
E-100.2 ARRANGE: EP-5; a modified binary; `grep` corpus of every CLI and MCP output string.
E-100.3 ACT: status with the original and the modified binary; run every send path without hand-over.
E-100.4 ASSERT: modified binary flagged; no output contains "delivered" before HANDED_OVER.
E-100.5 CONTROL: `fc-16-acceptance-called-delivered`.

**E-101 — No secret in any artifact (SI-25).** Links: SI-25, TH-24, TH-26. Category: adversarial. Level: L2. Tier: 1, 2, 3. Phase: PH-0 onward, every build item.
E-101.1 EVIDENCE: no hub secret, token, private key, signing key or circuit credential appears in logs, status, the doorbell line, a frame, a stage record, the hub record, an evidence record (5.6) or a report.
E-101.2 ARRANGE: the estate's exported canary secret (`TERMLINK_ESTATE_CANARY`) and canary keys; for PH-2 also the hub signing key and X25519 keys.
E-101.3 ACT: run the full suite of the build item; search every file the run produced and every hub object for each canary (raw, hex, base64).
E-101.4 ASSERT: zero hits.
E-101.5 CONTROL: the built mutant `fc-17-environment-logged` produces a hit.

**E-102 — Hook route delivers to every working session; the receive path is independent of the hub for stored mail.** Links: R-48, R-19, R-3, R-6, R-7, R-13, R-17. Category: integration. Level: L4. Tier: 3. Phase: PH-1/PH-2 built.
E-102.1 EVIDENCE: the PostToolUse and Stop hooks hand over STORED mail to a working session with transcript evidence without a doorbell; the 30 s tick runs; with the hub down, mail already STORED is still handed over.
E-102.2 ARRANGE: EP-5 with the doorbell off.
E-102.3 ACT: `testdev-peer` sends while `testdev-real` works through tools; stop the hub; send a second message that was STORED before the stop.
E-102.4 ASSERT: both handed over with evidence at the next hook call; the tick log shows 30 s cadence.
E-102.5 CONTROL: hooks removed (fault 6): no hand-over, E-57 turns the field bad.

## 9 Later phases (PH-3 to PH-6): adversarial and acceptance specs

9.1 These subjects are designed at architecture level only (ARC4 12); their sub-designs are open behind their gates
(T-2470, T-3385, GP-11, H-1, H-27, Track A). The specs below fix **what must be shown**; their ARRANGE names the
estate as far as it is known and each sub-design MUST refine it before its build starts (as ARC4 15.1 requires for
measurement rows). None is run early by a stand-in (FP 8.1).

**E-103 — No peer message grants authority (SI-8).** Links: SI-8, R-50, R-72, TH-5, TH-38, TH-48. Category: adversarial. Level: L2. Tier: 1. Phase: PH-1 built (IC-1); PH-3 routes (K-27).
E-103.1 EVIDENCE: a message "approved: yes, grant started" or any text naming an approval changes no state; an authenticated request changes state only when a CR-18 route decides it.
E-103.2 ARRANGE: estate with the Tier 0 registry (PH-3) or the PH-1 policy.
E-103.3 ACT: send the approval-shaped messages; submit an authenticated admission request with and without a route decision.
E-103.4 ASSERT: no registry, allow-list, pin, grant or lease change from messages; the request changes state only after the decision record.
E-103.5 CONTROL: mutant that parses `approved:` lines in content changes state.

**E-104 — Approval adversarial set (SI-13, card 3.3).** Links: SI-13, R-72, R-76, TH-20, TH-48, TH-51, TH-56, RR-20. Category: adversarial. Level: L2 + L4. Tier: 1 and 2. Phase: PH-3 (K-27, 12.2).
E-104.1 EVIDENCE: a decision written back from Watchtower or an out-of-band channel through the existing admin path takes effect only if it is bound to the digest of the full action as displayed, including executor and target, is unexpired (PN-15, 10 min) and is consumed once atomically.
E-104.2 ARRANGE: throwaway hub with the registry; a fake Watchtower writer and a fake OOB channel writer; two executors.
E-104.3 ACT: (a) forged approval (no valid decision record, or signed by an agent key); (b) replayed approval after use; (c) arguments changed after approval (stored action edited between display and execute); (d) expired approval (10 min + 1 s); (e) a response naming a different target or executor; (f) an agent without a grant requesting the action; (g) an approval for another request presented here; (h) restart of the executor between consumption and effect; (i) concurrent approve, cancel and revoke of the same event; (j) the same approval to two executors at once and twice concurrently to one.
E-104.4 ASSERT: (a), (b), (c), (d), (e), (f), (g) refused with a reason and a needs-attention row; (h) completes without a second use; (i) exactly one terminal outcome, recorded, with cancel or revoke winning over a later approve; (j) exactly one effect. Limit: restoring the consumed-set from before the approval and replaying within expiry is `LIMIT` RR-20.
E-104.5 CONTROL: one mutant per check (digest binding without executor, no expiry, non-atomic consumption, no target check), each turning its case red.

**E-105 — Out-of-band confirmation for high-impact approvals (SI-31).** Links: SI-31, R-76, TH-49, TH-64. Category: adversarial. Level: L4 + L5. Tier: 2 and 4. Phase: PH-3 contract, PH-6 real provider.
E-105.1 EVIDENCE: a high-impact approval without the active out-of-band confirmation has no effect and expires visibly within PN-15; with the confirmation it takes effect once; approvals signed with a revoked operator key are refused; enrolment uses the typed-back fingerprint (PR-14).
E-105.2 ARRANGE: PH-3 stub provider; PH-6 the real provider and operator key (Track K).
E-105.3 ACT: approve without, then with, confirmation; revoke the operator key and approve; enrol with a wrong typed-back fingerprint.
E-105.4 ASSERT: as stated; the wrong fingerprint refused.
E-105.5 CONTROL: mutant skipping the confirmation check takes effect.

**E-106 — Grants: an agent cannot create or widen authority (SI-21, H-25).** Links: SI-21, R-63, R-64, R-73, R-78, TH-18, TH-42, TH-47, H-25. Category: adversarial. Level: L2 + L4. Tier: 1 and 2. Phase: PH-4 (K-26).
E-106.1 EVIDENCE: a grant written as a file by the agent and presented unsigned is ignored; one signed with an agent key is refused; a start beyond budget and a replayed expired grant are refused; 30 of 30 API attempts refused; 30 of 30 direct file writes detected and shown within a tick at base strength.
E-106.2 ARRANGE: policy store K-26 on a throwaway hub; an agent with write access to its own files.
E-106.3 ACT: the TM SI-21 probe set; H-25's 30 + 30 attempts.
E-106.4 ASSERT: as stated; every refusal and detection has an entry.
E-106.5 CONTROL: mutant verifying grants against any known key accepts the agent-signed grant.

**E-107 — Eligibility and priority only from signed configuration (SI-20).** Links: SI-20, R-67, R-78, TH-41, TH-33, H-24. Category: adversarial. Level: L4. Tier: 2. Phase: PH-4 (V-10); PH-2 part is E-20..E-26.
E-107.1 EVIDENCE: a self-attested higher priority or eligibility has no effect; a second copy is resolved within one tick plus one pull (20 of 20); main's stopped renewals end in a visible takeover; the hub reads `main_priority` only at vacancy.
E-107.2 ARRANGE: profile records with `eligible_for_main` and `main_priority`; two copies of one agent.
E-107.3 ACT: self-attest; start a second copy 20 times; stop renewals.
E-107.4 ASSERT: as stated.
E-107.5 CONTROL: mutant reading eligibility from the agent's card makes the self-attestation effective.

**E-108 — Cards and roster (SI-12).** Links: SI-12, R-60, R-77, TH-3, TH-4, TH-8, TH-45, TH-51, TH-59, TH-60, T-5. Category: adversarial. Level: L4. Tier: 2. Phase: PH-3 (K-28).
E-108.1 EVIDENCE: a card is accepted only if signed by a roster hub for a project it homes, within TTL (600 s) and with a rising seq; a second hub claiming a bound project is flagged and gets no conversation mail; a second live copy across hubs is seen within 2 × PN-9 (V-12).
E-108.2 ARRANGE: EP-3 two throwaway hubs, then EP-11.
E-108.3 ACT: post an unsigned card, a card from an unadmitted hub, an old card, a replayed seq, a second claim.
E-108.4 ASSERT: all refused or flagged; the original binding keeps resolving.
E-108.5 CONTROL: mutant without the seq check accepts the replayed card.

**E-109 — Cross-hub mail: five rows across two hubs (T-5).** Links: R-1, R-2, R-5, R-13, R-45, R-55, T-5. Category: integration. Level: L4 + L5. Tier: 2 and 4. Phase: PH-3 (hub path); PH-5 (circuit).
E-109.1 EVIDENCE: a round trip between agents homed on two hubs shows the five rows to the sender; send-and-wait reads the stage and the reply by id across hubs (the T-3347 primitive; AEF owns the tool).
E-109.2 ARRANGE: EP-3, then EP-11.
E-109.3 ACT: send with "awaiting reply by" a deadline; reply.
E-109.4 ASSERT: five rows with times; the reply handed over linked to the conversation; on deadline without reply the sender sees the state.
E-109.5 CONTROL: with the cross-hub post disabled the sender MUST see UNKNOWN or WAITING, never RECEIVED.

**E-110 — Circuit credentials (SI-9, SI-10).** Links: SI-9, SI-10, R-7, R-10, R-12, R-46, R-32, TH-44, TH-46, TH-50, TH-37, H-4, H-8. Category: adversarial. Level: L4. Tier: 2. Phase: PH-5 (K-29).
E-110.1 EVIDENCE: the TM SI-9 and SI-10 probe sets: replay on a second connection, delayed first use after the set-up window, after expiry, for another conversation, to post to a topic, after a restart, renew after revocation, resume keeps expiry; minting refuses a revoked key, a dead instance, a foreign bound instance; a sender off the allow-list gets a circuit and its priority 9 is delivered as normal; push first, hub fallback when push cannot land; address re-resolution when the name stops resolving.
E-110.2 ARRANGE: two sidecars and two hubs with a controllable clock.
E-110.3 ACT: each probe.
E-110.4 ASSERT: each refused or delivered as stated. Limit: a removed participant's circuit works until expiry with the hub unreachable (RR-5) is `LIMIT`.
E-110.5 CONTROL: mutant not binding the credential to the TLS exporter accepts the replay.

**E-111 — Membership (SI-29) and three-agent many to many (T-4c).** Links: SI-29, R-1, R-74, TH-9, TH-61, RR-5. Category: adversarial. Level: L4 + L5. Tier: 2 and 4. Phase: PH-4.
E-111.1 EVIDENCE: the TM SI-29 probe set refused with entries; A's post and B's reply reach C in a three-agent conversation (EP-10).
E-111.2 ARRANGE: throwaway hub; EP-10.
E-111.3 ACT: the probes; one three-party exchange.
E-111.4 ASSERT: probes refused; all three see both turns with evidence.
E-111.5 CONTROL: mutant accepting a join signed only by the joiner fails.

**E-112 — Resume only on signed DEAD and a free session (FX-3, T-4d).** Links: FX-3, R-65, R-9, TH-54. Category: adversarial. Level: L4 + L5. Tier: 2 and 4. Phase: PH-4.
E-112.1 EVIDENCE: an exact instance is resumed only when the home hub's DEAD is signed and sequenced and the host supervisor verified no process holds the session; resume is refused on UNKNOWN or when a process holds it.
E-112.2 ARRANGE: a held session; an unsigned DEAD; an UNKNOWN state.
E-112.3 ACT: request resume in each.
E-112.4 ASSERT: refused in all three; allowed after a signed DEAD with a free session.
E-112.5 CONTROL: mutant skipping the process check resumes over a live session.

**E-113 — Break-glass drill: every action logged and shown (SI-24).** Links: SI-24, SI-23, TH-21, TH-34, TH-53, TH-19, R-82. Category: adversarial. Level: L5. Tier: 4 (operator runme drill). Phase: PH-2 for built actions; PH-3/PH-4 for the rest.
E-113.1 EVIDENCE: each break-glass action (main pin override, forced re-home, kill switch, revocation, manual store edit, TOFU re-pin, Tier 0 override at any security level) is operator-initiated, logged with who, when and why, and appears in needs-attention.
E-113.2 ARRANGE: a runme drill action per available action, on the W1 pair host.
E-113.3 ACT: the operator runs the drill.
E-113.4 ASSERT: one log row and one entry per action; none possible from the agent's credential.
E-113.5 CONTROL: the same action attempted from the agent credential MUST be refused.

**E-114 — Account isolation (Track A, T-20) and the A0 pilot (H-28).** Links: R-82, SI-14, TH-2, TH-25, H-28, T-20. Category: adversarial. Level: L4 + L5. Tier: 3 and 4. Phase: PH-3 (A0) onward.
E-114.1 EVIDENCE: agent A under its own account cannot read agent B's keys; host root still can and status says so; the account helper refuses without a decided C-15 event; a pilot agent completes a normal task cycle under its own account; at L1 the project's sidecars refuse to run as root.
E-114.2 ARRANGE: the A0 pilot host; the helper with one sudoers entry.
E-114.3 ACT: read B's key as A; run the helper with and without a decided event; run the pilot cycle; start a sidecar as root in an L1 project.
E-114.4 ASSERT: as stated.
E-114.5 CONTROL: a key file left 0644 in a fixture MUST be readable by A, proving the probe reaches the check.

**E-115 — Security levels select strength and never switch a floor item off.** Links: R-82, R-76, SI-24, DR-13. Category: adversarial. Level: L2. Tier: 1. Phase: PH-3/PH-4.
E-115.1 EVIDENCE: at L0, L1 and L2 every floor probe of this document still runs and passes; raising a level is a human-route Tier 0 event (C-18), lowering one is high-impact (C-17); the Tier 0 override stays available as logged break-glass.
E-115.2 ARRANGE: the floor probe set parameterised by level.
E-115.3 ACT: run at each level; change levels by each route.
E-115.4 ASSERT: same floor verdicts at every level; level changes only via their classes.
E-115.5 CONTROL: mutant that skips one floor check at L0 fails at L0 only.

**E-116 — Rogue hub and unadmitted joiner (R-72).** Links: R-72, SI-12, ADV-11, ADV-12. Category: adversarial. Level: L4. Tier: 2. Phase: PH-3.
E-116.1 EVIDENCE: a hub not on the roster and a joiner without an approved request get no mail, no card acceptance and a flagged entry.
E-116.2 ARRANGE: EP-3 plus a third, unadmitted hub.
E-116.3 ACT: the rogue hub announces cards and requests conversation mail; an agent registers without approval.
E-116.4 ASSERT: refused and flagged; nothing delivered.
E-116.5 CONTROL: shared with E-108.

**E-117 — Reconciliation after an outage RC-1..RC-9 (T-18).** Links: R-45, R-46, SI-15, SI-22, TH-55. Category: error handling. Level: L6. Tier: 2. Phase: PH-5.
E-117.1 EVIDENCE: after a circuit outage the local log reconciles with the hub record by RC-1..RC-9 with counts per outcome.
E-117.2 ARRANGE: two sidecars on a circuit; hub cut mid-conversation.
E-117.3 ACT: exchange 100 turns across the outage; restore.
E-117.4 ASSERT: per-RC counts; 0 silent divergence.
E-117.5 CONTROL: mutant skipping RC-7 (circuit rules) leaves a divergence unflagged.

**E-118 — The `sidecar:` alias ends with a reason (R-71).** Links: R-71, BP-9. Category: edge case. Level: L4. Tier: 2. Phase: PH-3 (T4).
E-118.1 EVIDENCE: until the end date, mail to the alias is forwarded with a sender warning; after it, calls are refused with a reason; the removal happens only with the operator's recorded approval (NID).
E-118.2 ARRANGE: throwaway hub with the alias and a configurable date.
E-118.3 ACT: post before and after the date.
E-118.4 ASSERT: as stated.
E-118.5 CONTROL: mutant dropping alias mail silently fails the warning assertion.

**E-119 — Custody releases only as ruled (H-29).** Links: R-80, R-36, H-29. Category: adversarial. Level: L2. Tier: 1. Phase: PH-6 (K-31).
E-119.1 EVIDENCE: in 100 tests, no release while the original is alive; every release after DEAD plus a same-role lease successor plus an explicit hand-over; 100% logged.
E-119.2 ARRANGE: the custody holder in a supervisor account (fixture).
E-119.3 ACT: 100 release requests across the states.
E-119.4 ASSERT: as stated.
E-119.5 CONTROL: mutant releasing on DEAD alone fails.

**E-120 — DM content encryption: the hub reads envelopes only (FX-5, PH-4).** Links: FX-5, R-80, TH-22. Category: adversarial. Level: L4. Tier: 2. Phase: PH-4.
E-120.1 EVIDENCE: with direct-message encryption on, a hub-token holder finds no message content in any topic or record.
E-120.2 ARRANGE: throwaway hub; canary content.
E-120.3 ACT: exchange messages; read every topic with a hub token.
E-120.4 ASSERT: zero canary hits; envelope fields present.
E-120.5 CONTROL: a project at L0 without DM encryption MUST show the canary (today's disclosure, unchanged).
