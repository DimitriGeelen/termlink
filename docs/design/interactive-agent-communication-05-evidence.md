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
