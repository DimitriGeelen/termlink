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
| OQ-f | What supervises the sidecar? | A a systemd template unit `termlink-sidecar@<agent>` (Restart=always); B the reachable launcher starts it beside the agent; C the notify-sidecar cron supervisor pattern | **Ruled 2026-10-12: D+** (see 6.1) |
| OQ-b | Agents run as root here, so the socket owner check separates nobody. Accept for PH-2? | A accept, status names "same-user separation absent"; B wait for per-agent accounts (Track A, PH-3) | **Ruled 2026-10-12: A** (accept for PH-2; status names "same-user separation absent: agents run as root"; the A+ after-the-fact admin-act surfacing was offered and not taken; real separation is Track A in PH-3) |
| OQ-c | STUCK deadline | A R-44.e as written; B keep the single 10-minute rule; C R-44.e structure, looser numbers | **Ruled 2026-10-12: A** (three deadlines, each only while the responsible side is reachable: not accepted 1 min; not handed over 2 ticks after the agent becomes ready; not answered 1 h, per-message override; urgent 15 s / next tool call or turn end / 5 min; STUCK names the overdue step; replaces `STUCK_AFTER_MS`; row P2) |
| OQ-d | The tick | A 30 s (R-17) as the unit of every bound, 500 ms poll kept internally; B make the poll the tick; C a shorter tick (changes R-17) | **Ruled 2026-10-12: A** (tick = 30 s: deadlines, revocation pull, component heartbeat and status checks run per tick; the 500 ms loop keeps pulling, handing over and reading the kill switch; tests use a simulated clock) |
| OQ-e | Fleet kill switch | A `admin delivery off --all` loops over the host's admin sockets and verifies each; B one fleet flag file every sidecar reads | **Ruled 2026-10-13: A** (`admin delivery off|on --why [--all]` over the host's listed admin sockets, each flip read back and logged where it happened; non-zero exit if any failed; a runme action) |
| OQ-h | Revocation list: where and who signs | A a hub topic signed by the hub signing key; requested by the agent's own project (signed by the old key) or by the operator through runme; B operator-only | **Ruled 2026-10-13: A** (operator revokes through the hub's root-only local socket on its host via a runme action, root on the hub host standing in for the operator key until K; the hub signs the list with its signing key (P10) and keeps it as signed, numbered hub state, not a topic; self-revocation by a statement signed with the old key stays; every revocation logged and a needs-attention entry. Open: replace root-on-host by the operator key in PH-3; whether a revocation also needs the operator's confirmation once a provider exists) |
| OQ-i | Does the daily canary run as `testdev`, and is OQ-3 ruled A? | A yes, as `testdev`; B as root | A: same proof as the PH-1 acceptance, no operator credential involved |
| OQ-j | On rollback, how does mail the sidecar stored but never handed over reach the agent? (re-post with the original signature is impossible, 6.4) | A kept in the store and listed to the operator, who reads or forwards it; B the drain also forwards each kept message to the old inbox as a new message from the sidecar quoting the original id and sender (provenance stated, not the original signature); C kept, and the sidecar stays up read-only so the agent can still `status`/read them | **Ruled 2026-10-13: A** (kept in the store and listed to the operator in the rollback report; revisit before W4) |
| OQ-g | FX-5 baseline | — | Not a decision: no PH-0 count was found, so P8 takes it before W1 |

### 6.1 OQ-f ruling D+: start with the project, keep running only where configured, cron backstop

1. **Per-project setting** `SIDECAR_KEEP_RUNNING` in the project's `.framework.yaml`, default off. The fleet
   cockpit (055) reads and writes this same key and keeps no copy, so a project works the same with or without
   the cockpit, and outside ring20.
2. **Project start always starts its sidecar**: the launcher starts it beside the agent, whatever the setting.
3. **Off**: the sidecar lives with the project; no restart after a crash; a project stop kills it. **On**: a
   supervisor restarts it after a crash or a reboot, and it keeps receiving and storing while the agent session is
   closed.
4. **An agent session ending** (auto-restart, compaction, closed terminal) does **not** stop the project. **An
   explicit project stop** (cockpit stop, or a stop script) always stops the sidecar, even when the setting is on,
   and switches its restarts off until the project starts again: the supervisor never fights a stop.
5. **Supervisor**: where systemd exists, the template unit `termlink-sidecar@<project>.service`
   (`Restart=always`). Where it does not (macOS, containers), the cron check of item 6 is the supervisor. The launcher
   chooses one at start; never two supervisors for one sidecar.
6. **Cron backstop on every host, every 10 minutes.** If the setting is on and the sidecar is not running: on a
   systemd host it repairs **through** systemd (`systemctl reset-failed` then `start`), never around it; without
   systemd it starts the sidecar. Each time it acts it writes its canary log and a needs-attention entry naming why
   the primary supervisor did not recover it: a backstop that had to act is a fault. Why on systemd hosts too: a unit
   can give up silently (`start-limit-hit`, as D-Bus did in the 2026-10-12 boot failure, T-3425), be disabled or
   masked, or lose its unit file in an upgrade.
7. **Safety**: the store lock (SI-16) refuses a second sidecar on the same store, so a mistaken second start fails
   rather than giving an inbox two receivers.
8. **Visibility**: status reports the setting and the active supervisor; a sidecar that should run and does not is
   a needs-attention entry. The canary meta-check (T-1723) covers a dead cron.

Row P3 builds items 5 to 8; row P7 builds items 1, 2 and 4 into the launcher.

### 6.2 K-1 as built (row P1, T-3365)

1. **One entry point.** `termlink_sidecar::cli::run(prefix, args)` serves every verb; the `termlink-sidecar`
   binary is a thin alias (prefix empty) and `termlink sidecar ...` passes its arguments through unchanged (prefix
   `sidecar`). The sidecar builds its own tokio runtime, so `termlink` runs it on a plain thread with no runtime
   context. The hook's detached finalizer re-executes the same binary with the same prefix.
2. **Production root.** A store under a directory carrying `.termlink-sidecar-root` with the first line
   `termlink-sidecar-production v1` runs in production mode, with the real HOME and runtime dir. The marker is
   written only by the operator's `runme.sh` (row P7). It is refused when it sits at `/`, is not a regular file,
   is owned by another user, or is group- or world-writable; HOME and the runtime dir must not lie inside a test
   estate (the two modes are never mixed). Without a production root the PH-1 estate rule applies unchanged.
3. **Visible.** Status carries `"mode": "estate"|"production"`; a refusal names both markers.

### 6.3 K-2 as built (row P2, T-3428)

1. **Tick and table.** `deadline::TICK_MS` = 30 s; `Deadlines::R44E` holds accept 60 s / 15 s, hand-over 2 ticks / 1
   tick after the agent became ready, answer 1 h / 5 min (normal / urgent). `STUCK_AFTER_MS` is gone; `status_view`
   takes the table. The sidecar's sweeper thread runs the receiver check once per tick; the 500 ms loop is unchanged.
2. **STUCK needs evidence of a live responsible side (R-44.a).** Until liveness exists (row P3), the sender calls
   STUCK only when the recipient itself proves it: RECEIVED without STORED past the accept deadline ("not accepted"),
   or the recipient's signed overdue notice ("not handed over", "not answered"). The view names the step in
   `stuck_step`. Posted with nothing back, or handed over and unanswered past the deadline with no notice, reads
   UNKNOWN. Stored without a notice reads WAITING FOR RECIPIENT however old: no false STUCK on a busy agent.
3. **Receiver notices.** `OVERDUE-HANDOVER` and `OVERDUE-ANSWER` join `wire::NOTICES`. Once per tick, while one of
   its sessions holds a fresh ready flag, the receiver sends a notice for each stored message not handed over within
   the deadline counted from the later of storing and becoming ready, and each handed-over message it has not
   replied to past the answer deadline; once per message and step, recorded after it is queued.
4. **Readiness ends with the session.** A new hook event `session-end` clears the session's ready flag, so a closed
   session is not ready and its mail waits rather than going overdue. A crashed session keeps its flag until P3/P18.
5. **Urgency on the sender side.** The SENT line records `priority=N`, so the sender applies the urgent deadlines
   without keeping the payload.
6. **Deferred:** DEAD (the home hub must declare an instance ended, R-60) to row P17; the per-message answer deadline
   (a new wire field) and "no-action" to row P14; STUCK for "not accepted" and "not answered" without a notice needs
   liveness, row P3.

### 6.4 K-4 as built (row P4, T-3429)

1. **Drain** (`termlink sidecar admin drain --why TEXT`, admin socket): sets `receive.drained` in the store. The
   receiver reads it at the start of every pass and before every message: nothing new is read, the mail stays on the
   hub and the cursor does not move (senders keep chasing). Nothing is handed over while drained. The kill switch is
   separate and unchanged.
2. **Count compare.** After 1.5 s (a pass mid-message finishes), the drain lists every own STORED message with no
   HANDED_OVER or REFUSED, re-reads each record against its chain digest, writes `state.drain-manifest`, and logs
   DRAIN with `listed=N kept=K unreadable=U` on the admin chain. It answers `ok` only when K == N; otherwise
   `count-mismatch` ("do not start the old receiver") and a needs-attention entry. Status shows `receiving`.
3. **Undrain** reverses it (UNDRAIN logged).
4. **Finding: re-post is not possible as FP 6.2.3.e words it.** The signature binds the topic it was posted to, the
   store keeps payloads but not signatures, and AEF's receiver does not read this message format. So "kept" is the
   built path: the messages stay in the store for the 14 days. Whether they must also reach the agent through the old
   path is an operator decision before W1 (OQ-j).
5. **Rollback per wave**: drain (count matches) -> stop the sidecar -> start the old receiver: a runme action of each
   wave.

## 7 Out of scope for PH-2

Cross-host and cross-operator delivery, grants and their knobs (T-3423), Tier 0 machinery, the Track A pilot
account (FP 6.2.4.a (12)): all PH-3 or later.
