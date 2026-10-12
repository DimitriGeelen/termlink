# arc-011 PH-2 architecture: the first operational release

PH-2 per `interactive-agent-communication-03-floor-and-phasing.md` 6.2.3 ("FP" below): one host (.107), one project
(010-termlink), the hub path, real agents and real credentials. The first phase in which the floor protects
something that matters. PH-1 (`arc-011-ph1-architecture.md`, "ARCH") built and proved the core in a disposable
estate; PH-2 takes it out of the estate.

Gap evidence for every row below: `docs/reports/T-3426-arc011-ph2-gapmap.md` (T-3426, survey of FP, REQ, ARCH, the
PH-1 acceptance report and `crates/termlink-sidecar/src`, with file:line). Four of its claims were re-checked by
hand (estate guard, STUCK_AFTER_MS, READY_MAX_AGE_MS, client admin verbs).

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 DRAFT | 2026-10-12 | First draft: what PH-1 leaves, components, waves, build table, operator questions | T-3426 |

## 1 Where PH-1 leaves us

1. **Done and proven (PH-1 acceptance, X1 to X6):** durable store and chain, signed wire, local API with three
   sockets, receive with stage records, hand-over only with evidence, OQ-2 E admission (held, allow, deny), the
   Claude adapter by hooks, the per-agent delivery switch, exact-instance addressing, hub declared and verified.
2. **Runs only inside a test estate.** `estate::guard` refuses to start without the estate marker
   (`estate.rs:53`). Nothing real can use it yet: this is the first blocker.
3. **Not built at all:** the doorbell (typed line, readiness re-check, activity veto), supervision, revocation
   and the hub signing key it needs, blobs, quotas, the sender's chase ladders, `up_to` and the sequence window,
   reconciliation with the hub record, the daily canary, the fault injectors, packaging.
4. **Built to PH-1's needs and too loose for PH-2:** STUCK after 10 minutes (`api.rs:65`; R-44.e wants about 1
   minute); readiness fresh for 24 hours (`deliver.rs:164`; SI-2 wants re-checked just before typing); the kill
   switch is per store with no fleet form and no client verb (`client.rs` has `held`, `allow`, `deny` only).
5. **Stale text in ARCH** (gap map D-1 to D-8: subcommand naming, `stage.rs`, `delivery off --all`, the
   `handed-over` field list, "strace-free"). Fixed in build row P0 so PH-2 starts from a true description.

## 2 Components

| id | Component | Where | New or changed |
|---|---|---|---|
| K-1 | Production start mode | `estate.rs`, `daemon.rs` | Changed: a production config under `/var/lib/termlink-sidecar/<agent>/` replaces the estate marker outside tests; the estate guard stays for tests |
| K-2 | Tick | `daemon.rs` | New: one named tick (R-17, 30 s) that every FP-1 bound, the kill switch and the revocation pull count in; the 500 ms poll stays as the internal loop |
| K-3 | Supervisor + component heartbeat | systemd template unit, `daemon.rs` | New (R-8; FP-1 failure 4) |
| K-4 | Drain and rollback | `client.rs`, `api.rs` | New (6.2.3.e): refuse new mail, re-post each STORED-not-handed-over message to the old inbox with its id and signature, compare counts |
| K-5 | Needs-attention surface | `store.rs`, `scripts/session-start-alerts.sh` | Changed: every attention entry is surfaced, not only held messages (R-37) |
| K-6 | Kill switch, fleet form | `client.rs`, `runme.sh` | Changed (SI-23): `admin delivery on|off`, and `--all` over the admin sockets listed for the host |
| K-7 | Real-agent wiring | `.claude/settings.json` of this project, launcher, hub | New: per-agent keys registered (PR-1a), roster from the real project, hooks in the real settings, instance from the real SessionStart, fixed default "own project" and an operator-pinned main (SI-20, SI-21) |
| K-8 | Packaging | `release.yml`, `install.sh`, formula, `check-release-artifact-drift.sh` | Changed: the sidecar ships in the release artifact; checksums at install and in status (SI-27) |
| K-9 | Hub signing key + revocation list | `termlink-hub`, `pull.rs` | New (R-61, RK-1, SI-11): an ed25519 key apart from the TLS cert; a signed list pulled each tick; a failed pull is not an empty list |
| K-10 | Sequence window, `up_to` | `wire.rs`, `pull.rs` | New (SI-5, SI-6) |
| K-11 | Caps and quotas | `policy.rs`, `pull.rs` | New (SI-19, R-56) |
| K-12 | Sender ladders | `deliver.rs` or a sender module | New (R-30, R-31, R-52 `--priority`, R-58 owed answers) |
| K-13 | Blobs | `store.rs`, `wire.rs` | New (SI-28) |
| K-14 | Reconciliation | `daemon.rs`, `store.rs` | New (SI-15 on start; SI-22 daily head compare) |
| K-15 | Hub changes | `termlink-hub` | New: consumer list per inbox (SI-16), admin RPC log and sweep refusal (FX-6), a scope of its own for the founding verbs (FX-4) |
| K-16 | Status call, CLEAR, reachability | `api.rs` | Changed: freshness, CLEAR needs surfacing progress (FX-8, R-29), the reachability card (R-53), new strength flags |
| K-17 | Fault injectors | `tests/`, `scripts/` | New: the five GP-12 faults and the unsurfaced-mail case, each diagnosed by status within FP-1 |
| K-18 | Activity veto + readiness re-check | `adapter/claude.rs` | New (SI-2, R-81, T-3387) |
| K-19 | Doorbell | `adapter/claude.rs` | New (SI-1, R-19 to R-23): one fixed typed line, counts and 32-hex ids only, no peer byte |
| K-20 | Vendor-channel log | `hooks.rs` | New (FX-1) |
| K-21 | Daily canary | `.context/cron/`, `scripts/` | New (R-66, H-14): the two-agent test with a negative control on the installed artifact |

## 3 Waves (FP 6.2.3.b) and what gates each

| Wave | What runs | Gate before it starts | Rollback |
|---|---|---|---|
| W1 | One pair of this project's agents, hook delivery only, doorbell off; kill switch exercised | Rows P0 to P8 done; drain procedure (K-4) proven on the estate | K-4 for the pair; old receivers restart only after the count compare |
| W2 | The same pair, doorbell on; activity veto live; H-11 measured 7 days | W1 clean for 7 days; K-18, K-19 done; kill switch proven to stop a typed doorbell | Doorbell off first (switch), then K-4 |
| W3 | Every agent of this project on this host started through the reachable launcher | W2 H-11 within bound; canary K-21 running daily | K-4 per agent |
| W4 | AEF's receiver and the legacy notify-sidecar switched off on this host | W3 two-agent acceptance with negative control passes (R-49.a (3)) | AEF receiver stays installed, switched off, until PH-3 exits |

Exactly one receiver serves an inbox at every moment (FP 6.2.3.f). Each wave move is a `runme.sh` action the
operator runs; the agent never switches a receiver itself.

## 4 How each exit (FP 6.2.3.d) is proven

| Exit | Proof | Built in |
|---|---|---|
| 1 Floor probes green with kill-checked negative controls | Every row below ships its probe and its mutant; `scripts/arc011-mutants.sh` extended; a PH-2 FC matrix report | every row |
| 2 Two-agent test + negative control on the installed artifact, then daily 14 days (H-14) | `arc011-ph1-demo.sh` gains an installed-artifact mode and a negative control; the canary logs each day | P9, P23 |
| 3 GP-12 x5 + unsurfaced mail diagnosed within FP-1, zero manual recovery (H-18, H-19) | K-17 injectors, 20 injections each | P19 |
| 4 Kill switch stops mid-turn delivery within one tick | Flip during a live run, hook path and doorbell path | P6, P21 |
| 5 Clean host install, upgrade, rollback, restart; macOS CI green; platform-lock entry for the veto | A clean-host script; CI; `.context/checks/platform-lock-allowlist` entry | P9, P20 |
| 6 Status names the absent strength flags | Status output; flags for revocation reachable, supervisor, checksum verified | P18 |
| 7 H-3, H-7, H-11, H-13, H-20, H-30 measured | One harness each, inside the row that builds the mechanism | P12, P13, P15, P21, P23 |
| 8 Inventories updated | `scripts/check-arc011-inventory.sh` green | P8, last row |

## 5 Build table (dependency order; each one deliverable, sized to one session)

| Row | Deliverable | Closes | Depends on |
|---|---|---|---|
| P0 | ARCH stale text fixed (gap map D-1 to D-8); this document reviewed | — | — |
| P1 | Production start mode (K-1) and, per OQ-a, the `termlink sidecar` subcommand | R-6, R-39 (part), OD-9 C | P0 |
| P2 | Tick (K-2); STUCK at R-44.e's bound; DEAD as a dead letter | R-17, R-44 | P1 |
| P3 | Supervisor unit + component heartbeat, installed by runme (K-3) | R-8 | P2 |
| P4 | Drain and rollback (K-4), proven on the estate with a count compare | 6.2.3.e | P1 |
| P5 | Needs-attention surface for every entry (K-5) | R-37 | P1 |
| P6 | Kill switch client verb and fleet form (K-6) | SI-23 | P2 |
| P7 | Real-agent wiring (K-7): keys registered, roster, hooks in real settings, own-project default, pinned main | PR-1a, R-48, R-62, SI-20, SI-21 | P1, P3 |
| P8 | FX-5 baseline count (before W1) + inventories | FX-5, exit 8 | — |
| **W1** | **Operator runs the W1 action** | | P0 to P8 |
| P9 | Packaging: release artifact, install, checksums in status, clean-host install/upgrade/rollback test | R-39.e, SI-27, exit 5 | P1 |
| P10 | Hub signing key (K-9 part 1) | R-61 | — |
| P11 | Revocation list pulled each tick (K-9 part 2) | RK-1, SI-11 | P2, P7, P10 |
| P12 | Sequence window + `up_to` + H-3 harness (K-10) | SI-5, SI-6 | P1 |
| P13 | Caps and quotas + H-13 harness (K-11) | SI-19, R-56 | P2 |
| P14 | Sender ladders, `--priority`, owed answers, R-18/R-25 tests (K-12) | R-30, R-31, R-52, R-58, R-18, R-25 | P2 |
| P15 | Blobs + H-7 harness (K-13) | SI-28, R-11 | P1 |
| P16 | Reconciliation on start + host-crash F-7 test + daily head compare (K-14) | SI-15, SI-22 | P2 |
| P17 | Hub changes (K-15): consumer list, admin RPC log, sweep refusal, founding-verb scope | SI-16, FX-6, FX-4 | — |
| P18 | Status, CLEAR rule, reachability card, new strength flags (K-16) | R-53, R-29, FX-8, exit 6 | P3, P5, P11 |
| P19 | Fault injectors + drill (K-17) | GP-12, R-6.e, R-70, exit 3 | P3, P17, P18 |
| P20 | Activity veto + readiness re-check + platform-lock entry (K-18) | SI-2, R-81 | P2 |
| P21 | Doorbell + H-11 harness; kill switch proven on the typed path (K-19) | SI-1, R-19 to R-23 | P6, P20 |
| **W2** | **Operator runs the W2 action** | | W1 + 7 days, P20, P21 |
| P22 | Vendor-channel use logged and surfaced (K-20) | FX-1 | P5 |
| P23 | Daily canary on the installed artifact with negative control; H-14 counter; H-20, H-30 harnesses (K-21) | R-66, exit 2 | P9 |
| **W3** | **Operator runs the W3 action** | | W2 bound met, P23 running |
| P24 | W3 two-agent acceptance with negative control; PH-2 exit report | R-49.a (3), exits 1 to 8 | all |
| **W4** | **Operator runs the W4 action** | | P24 |

Every row ships its floor probe and a mutant that turns it red (exit 1), and its own `## Verification`.

Existing backlog tasks that become rows (no duplicate tasks): T-3365 "termlink sidecar in the binary" is P1,
T-3362 "sender sequence numbers" is P12, T-3358 "--priority send option" is part of P14, T-3361 "daily two-agent
message canary" is P23. The other rows get a task each when they start.

## 6 Decisions awaiting the operator (asked one at a time)

| id | Question | Options | Recommendation |
|---|---|---|---|
| OQ-a | Is a separate `termlink-sidecar` binary acceptable, or must it be `termlink sidecar` in the main binary (OD-9 C)? | A fold it in as a subcommand (the crate is already a library); B keep two binaries | **Ruled 2026-10-12: A** (fold into `termlink sidecar`; T-3365 is row P1; T-3426 Decisions) |
| OQ-f | What supervises the sidecar? | A a systemd template unit `termlink-sidecar@<agent>` (Restart=always); B the reachable launcher starts it beside the agent; C the notify-sidecar cron supervisor pattern | A: survives reboot and crash independently of the agent; launchd is the macOS equivalent later |
| OQ-b | Agents run as root here, so the socket owner check separates nobody. Accept for PH-2? | A accept, status names "same-user separation absent"; B wait for per-agent accounts (Track A, PH-3) | A: the flag keeps it visible; Track A is already PH-3 |
| OQ-c | STUCK deadline | A about 1 minute (R-44.e, FP-1); B keep 10 minutes | A: the requirement is explicit; 10 minutes was a PH-1 convenience |
| OQ-d | The tick | A 30 s (R-17) as the unit of every bound, 500 ms poll kept internally; B make the poll the tick | A |
| OQ-e | Fleet kill switch | A `admin delivery off --all` loops over the host's admin sockets and verifies each; B one fleet flag file every sidecar reads | A: no new shared state, and each flip is logged where it happened |
| OQ-h | Revocation list: where and who signs | A a hub topic signed by the hub signing key; requested by the agent's own project (signed by the old key) or by the operator through runme; B operator-only | A (TM RK-1 allows the agent's own project) |
| OQ-i | Does the daily canary run as `testdev`, and is OQ-3 ruled A? | A yes, as `testdev`; B as root | A: same proof as the PH-1 acceptance, no operator credential involved |
| OQ-g | FX-5 baseline | — | Not a decision: no PH-0 count was found, so P8 takes it before W1 |

## 7 Out of scope for PH-2

Cross-host and cross-operator delivery, grants and their knobs (T-3423), Tier 0 machinery, the Track A pilot
account (FP 6.2.4.a (12)): all PH-3 or later.
