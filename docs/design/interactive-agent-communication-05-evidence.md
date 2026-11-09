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
