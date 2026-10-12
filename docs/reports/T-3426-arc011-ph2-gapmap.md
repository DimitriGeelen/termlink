# arc-011 PH-2 gap map (T-3426)

Read-only survey, 2026-10-12, by agent `ph2-gapmap`. Nothing was built or run; no cargo, no tmux.

Sources and abbreviations:

- **FP**: `docs/design/interactive-agent-communication-03-floor-and-phasing.md`. PH-2 is at 435-441, the floor table at 89-118, the FX table at 130-137, FP-1 at 553-569, and H-n at 633-658.
- **REQ**: `docs/design/interactive-agent-communication-01-requirements.md`.
- **TM**: `-02-threat-model.md`. PR-1a is at 239 and RK-1 at 397.
- **ARCH**: `docs/design/arc-011-ph1-architecture.md`.
- **ACC**: `docs/reports/arc011-ph1-acceptance.md`.
- **SC**: `crates/termlink-sidecar/src/`.

A grep for `up_to`, `blob`, `equivocat`, `revoc`, `supervis`, `heartbeat`, `quota`, `reconcil`, `inject` and `tick` finds **none** of them in SC.

## A. Element table

DONE = PH-1 built it and ACC proves it. PARTIAL = some of it exists. NONE = nothing exists.

### A.1 Scope 6.2.3.a (1): the sidecar binary, adapter, supervision, hooks, doorbell

| id | requirement | PH-1 | evidence | PH-2 must add |
|---|---|---|---|---|
| R-6 / R-39 / OD-9 C | `termlink sidecar` ships in the binary with every deployment | PARTIAL | A separate binary `termlink-sidecar` (SC/main.rs:1-41; ARCH 0.5 row line 11). No sidecar subcommand in termlink-cli (grep finds only cli.rs:2014, a comment) | The `termlink sidecar ...` subcommand group in the release binary, or an operator ruling that a second binary satisfies OD-9 C. Packaged in the release artifact (see item 6) |
| R-47 | Harness adapter contract | PARTIAL | SC/adapter/claude.rs (116 lines); hook.rs. ACC BV-5 | A written contract and a parity test per release (FP 326). Readiness reporting for R-21 |
| R-21 | Readiness reported by the harness | PARTIAL | The Stop hook records ready (ARCH 6.2, line 145; deliver.rs:284 "PH-1 only records it; the PH-2 doorbell reads it"). READY_MAX_AGE_MS = 24 h (deliver.rs:164) | Readiness that is fresh and re-checked just before typing (SI-2). A 24-hour freshness is far too loose for that |
| R-8 | Always respawns (supervised) | NONE | ACC "Limits" item 2: "The sidecars are not supervised" | A supervisor such as a systemd unit or the reachable launcher, a component heartbeat (FP-1 failure 4), and macOS behaviour (R-8.g) |
| R-48 | Hook delivery plus the launcher | PARTIAL | The PostToolUse, SessionStart and Stop hooks are DONE (ACC BV-5). They are installed only in estate settings (ARCH 6.2) | Install the hooks into this project's real agents' settings. Launcher wiring "by natural restarts" (FP 327) |
| R-19, R-20, R-22, R-23 | Doorbell: one short typed line, only when the prompt is free, never inferred from the screen | NONE | ARCH 13.2 (line 305): "doorbell (typing) … not in PH-1". `inject` is not in SC | The whole doorbell: typing path, fixed line text (SI-1), READY plus foreground re-check (SI-2) |
| R-81 / T-3387 | Activity veto, busy bit | NONE | FP 494: T-3387 is `captured`, horizon next | Build the veto, with a no-`/proc` behaviour entry (exit 5) |

### A.2 Scope 6.2.3.a (2): the whole delivery chain on one host

| id | requirement | PH-1 | evidence | PH-2 must add |
|---|---|---|---|---|
| R-11 + SI-28 | Hand to own sidecar; blobs stored only under their own digest | PARTIAL | `send` is DONE (ACC BV-2). The no-path rule exists (ARCH 3.3). Blobs: NONE (grep) | Blob transport and storage under the digest in a private dir. H-7 measurement |
| R-14 / R-15 / R-16 | RECEIVED; STORED durable; flag after store | DONE | ACC BV-4; ACC X3 item 7; ARCH 4.3 (line 113) | Host-crash test and reconciliation (see SI-15) |
| R-17 | 30-second tick, supervised in PH-2 | PARTIAL | A poll loop of 500 ms (daemon.rs:47-49). No "tick" concept | A defined tick that every FP-1 bound counts in. Supervision |
| R-18 | Highest priority first | UNVERIFIED / likely PARTIAL | The priority clamp exists (policy.rs:104). Pull ordering was not checked | A three-message fixture test (FP 297) |
| R-24 | HANDED_OVER only with evidence | DONE | ACC X3 items 9 and 15 | — |
| R-25 | Flag down only when the queue is empty | UNVERIFIED | Not checked in SC | A test that a lowered ready or pending flag never hides mail |
| R-26 | ANSWER READY / reply path | DONE | ACC BV-6 | — |
| R-30 / R-31 | Two polling ladders; the chase as sidecar code | NONE | "simulated-clock test already in PH-1" (FP 309). No ladder code found (UNVERIFIED beyond grep) | The sender-side chase or re-send ladder, with jitter (R-56) |
| R-44 | States and "stuck"; DEAD as a dead letter | PARTIAL | Four labels exist; STUCK_AFTER_MS = 10 min (api.rs:65) | DEAD as a dead letter. Deadlines to match R-44.e, about 1 min (see D-3) |
| R-45 | The hub record is the truth | PARTIAL | Chain plus signed stages (ACC X3 items 3 and 11) | Chain-head anchor compare (SI-22 completing) |
| R-51 | Idempotence per stage | DONE | ACC X3 items 2 and 15 | — |
| R-52 | How urgent is marked | PARTIAL | Clamp and downgrade (ARCH 7.1, line 212). FP 331: "send tooling lacks `--priority`" | `--priority` on the client `send` (UNVERIFIED whether client.rs has it) |
| R-56 | Flooding and storm control | NONE | grep `quota`: none | Caps, quotas and ceiling (SI-19), jitter, hop limit |
| R-58 | Owed-answers list | NONE | Not in SC | A view of HANDED_OVER messages still waiting for a reply or no-action |
| R-59 | Sender-assigned numbers | PARTIAL | `seq` is carried and signed (wire.rs:45). ARCH 5.4 (line 137): "PH-1 records seq and refuses nothing" | A window and equivocation check (SI-5), and `up_to` (SI-6) |

### A.3 Scope 6.2.3.a (3): visibility

| id | requirement | PH-1 | evidence | PH-2 must add |
|---|---|---|---|---|
| R-53 | Reachability card | NONE | — | A visible reachability state; peers see yes or no (CR-11) |
| R-29 / FX-8 | Deaf means halt; never CLEAR on a heartbeat alone | NONE | No CLEAR rule in SC | A CLEAR computation that needs recent surfacing progress |
| R-37 | Needs-attention surface | PARTIAL | `Store::attention` writes a durable list (store.rs:339-341). It is listed at session start for held messages only (ARCH 7.2.b) | An operator-facing surface that reads every attention entry. Alarms only for urgent |
| R-66 | Daily end-to-end canary | NONE | No arc011 crontab in `.context/cron/` | A cron canary that runs the two-agent test daily on the installed artifact (exit 2) |
| status call | The status call | PARTIAL | api.rs:844-870: strength flags, missing_strength, chain head, delivery state | Freshness stamps, the 5-second p95, and diagnosis of the five GP-12 failures |
| GP-12 x5 + INC-a (R-6.e, R-70) | Standing fault injections | NONE | FP 7.2.d table (FP ~559-566). `scripts/check-arc011-inventory.sh` exists (purpose UNVERIFIED) | Five injectors plus unsurfaced mail, each tied to a status-call diagnosis |

### A.4 Scope 6.2.3.a (4): identity and consent in their NK form

| id | requirement | PH-1 | evidence | PH-2 must add |
|---|---|---|---|---|
| PR-1a | Per-agent keys registered | PARTIAL | Keys are created per run in the estate (ARCH 8.1, E-5). The roster is filled by the harness (assumption A-5). There is no registration | Real per-agent keys on this host, registered at the hub (TM 239). Key replacement signed by the old key or the operator |
| RK-1 / SI-11 | Agent-key revocation via a signed list pulled each tick | NONE | grep `revoc`: none, in SC and in the hub | Hub revocation list, a signed pull each tick, and "a failed pull is not an empty list" |
| R-61 (PH-2 part) | Hub signing key apart from the TLS cert | NONE | The only hub signing key found is the webhook HMAC (termlink-hub/src/webhook.rs:40), which is unrelated | A hub ed25519 signing key, first used to sign the revocation list |
| FX-2 | Hub declared and verified | DONE | ACC X3 item 16 | — (PH-3 extends it to every client) |
| R-62 | Exact-instance conversations | DONE | ACC X3 item 14 (SI-26); `from_instance` (ARCH 0.6) | Bind the instance from the real SessionStart outside the estate |
| SI-20 / SI-21 | Fixed default "own project", operator-pinned main | PARTIAL | Roster plus the operator's admit and refuse lists (ARCH 7.2; policy.rs:109-123). No "main" pin found; "pin" hits are hub_id | The fixed "own project" default from the real roster. An operator-pinned main. No configuration knob |

### A.5 Scope 6.2.3.a (5): FX items and the kill switch

| id | requirement | PH-1 | evidence | PH-2 must add |
|---|---|---|---|---|
| FX-1 | Log and show every vendor-channel use | NONE | ARCH 1.1 (CR-21 observed, no enforcement) | Detect use of a vendor peer channel or sub-agent from the transcript or hooks; log it and add a needs-attention entry |
| FX-4 | Founding verbs (inject, exec, remote exec) get their own scope and are logged with the caller | NONE / UNVERIFIED | The hub router uses the existing PermissionScope::Execute (router.rs:2596). No dedicated scope or log found | A new scope, a log row per call, owner refusal, and a cross-project Tier-0 refusal |
| FX-5 | Disclosure does not grow | UNVERIFIED | Bodies travel in the hub payload as plaintext (ARCH 5.1, line ~130). The PH-0 baseline count was not checked | A per-phase count of plaintext content fields against the PH-0 baseline (see D-6) |
| FX-6 | Every hub admin RPC logged; sweep below open messages refused | NONE / UNVERIFIED | `handle_channel_sweep_with` (channel.rs:708) shows no open-message refusal in the first 30 lines | A hub change: an admin RPC log, and sweep refusal against open claims or messages |
| SI-23 | Kill switch, per agent and fleet, within one tick, audited | PARTIAL | Per-store flag, read every call, logged (api.rs:525-545; policy.rs:161; ACC X3 item 12) | A fleet form (`--all`), an operator client verb (client.rs has only held, allow and deny: client.rs:20-22), and a proof that typed doorbells also stop |

### A.6 Scope 6.2.3.a (6): install and upgrade

| id | requirement | PH-1 | evidence | PH-2 must add |
|---|---|---|---|---|
| R-39.e / SI-27 | Install from the release artifact, checksums at install and in status, protocol refusal | NONE | The estate guard refuses to run outside a marked estate (estate.rs:53; ACC X3 item 7) | A production start mode. Release packaging (release.yml, install.sh, the formula: the artifact-drift check covers names only). Checksum verification in status. Rollback and restart |

### A.7 Floor items of 6.2.3.c not already covered above

| id | requirement | PH-1 | evidence | PH-2 must add |
|---|---|---|---|---|
| SI-1 | Typed line: fixed text, count, 32-hex ids; no peer byte | NONE | No doorbell | Build it with the doorbell |
| SI-2 | READY plus foreground re-checked; activity veto; never the screen | NONE | Only the ready flag is recorded | Build it (H-11) |
| SI-5 | Equivocation; window refusal | NONE | ARCH 5.4 | Build it |
| SI-18 | Priority, urgency and trust at the receiver | DONE (PH-1 form) | ACC X3 item 10 | Re-prove with the real roster |
| SI-19 | Receiver caps and quotas, reserved shares | NONE | — | Build it (H-13) |
| SI-27 | Checksums | NONE | — | See A.6 |
| SI-28 | Blobs | NONE | — | See A.2 |
| SI-6 (completing) | `up_to` = highest contiguous number | PARTIAL | Signed stages DONE (ACC X3 item 3). `up_to` is NONE (ARCH 5.3) | `up_to` receipts (H-3) |
| SI-14 (completing) | Three scopes | PARTIAL | Send, status and admin sockets exist (ACC X3 item 6). Owner check is weak as root (ARCH 3.1, line 82) | All three scopes on the real host. Status must state that same-user separation is absent |
| SI-15 (completing) | Reconciliation with the hub record on start; host-crash test | PARTIAL | Local recovery DONE (ARCH 4.3). grep `reconcil`: none | Hub-record reconciliation on start. Host-crash (F-7) test |
| SI-16 (completing) | Consumer visibility on the hub | PARTIAL | Store lock DONE (ACC X3 item 8). No hub consumer list (grep `consumers` in the hub: none relevant) | A hub-side list of each inbox's consumers, read each tick (FP-1 failure 2) |
| SI-22 (completing) | Heads in receipts; daily compare | PARTIAL | The chain head is in stages (wire.rs and store.rs:546). TAIL_NOTE says "anchor comparison is PH-2" (store.rs:28) | The daily head compare, which closes the `chain_tail_anchor` strength flag (api.rs:869) |

## B. Exit criteria 6.2.3.d

| # | Criterion | What proves it | Tooling that exists | Missing |
|---|---|---|---|---|
| 1 | Probes of 6.2.3.c green, with kill-checked negative controls | One probe and one mutant per floor item | `scripts/arc011-mutants.sh`, 18 patches in `crates/termlink-sidecar/tests/mutants/`, `docs/reports/arc011-ph1-fc-matrix.md` | New probes and mutants for SI-1, SI-2, SI-5, SI-11, SI-19, SI-20/21, SI-27, SI-28, FX-1, FX-4, FX-5, FX-6, FX-8 and the completing parts |
| 2 | Two-agent live test plus negative control on the installed artifact, then 14 days daily (H-14) | The acceptance run against installed binaries, then the daily canary log, ≥500 messages | `scripts/arc011-ph1-demo.sh` (estate-only, `claude -p`) | An installed-artifact mode, a negative control, a cron canary and a soak counter |
| 3 | 5 GP-12 faults plus INC-a, each diagnosed by status within FP-1; zero manual recovery (H-18, H-19) | 20 injections each, 100/100 within bound; 500 kills | The PH-1 soak (`tests/soak.rs`, H-16/H-17) and failpoints | The injectors, status diagnoses, component heartbeat, supervisor, and the drill protocol |
| 4 | Kill switch stops mid-turn delivery within one tick | A flip during a run | `policy_delivery_switch_...` and `deliver_switch_off_...` tests (ACC X3 item 12) | Tick definition, fleet form, doorbell path |
| 5 | Clean-host install, upgrade, rollback, restart; macOS CI green; platform-lock entry for the veto | A clean-host script; CI; `.context/checks/platform-lock-allowlist` entry | macOS CI job (T-2692/T-3271), `scripts/check-platform-lock.sh`, `check-release-artifact-drift.sh` | Sidecar in the release artifact, a clean-host test, and the veto plus its allowlist entry |
| 6 | Status names the absent strength flags | Status output | DONE in PH-1 form: `strength_flags` (api.rs:864-870), shown in BV-1 | Add PH-2 flags (revocation reachable, supervisor, checksum verified); remove `chain_tail_anchor` once anchored |
| 7 | H-3, H-7, H-11, H-13, H-20, H-30 measured | FP section 9 designs (FP 633-658) | None found for these | A measurement harness for each: reorder depth, blob size, doorbell timing, share accounting, attention counts per day, re-send age ledger |
| 8 | Inventories updated | Inventory files | `scripts/check-arc011-inventory.sh` (contents UNVERIFIED) | Update the inventories for the new components and channels |

## C. Dependencies and ordering

1. The **production start mode** (relaxing or replacing the estate guard, estate.rs:53) blocks everything that runs on real agents: W1 and every exit criterion run on the installed artifact.
2. **Packaging** (sidecar in the release artifact, SI-27) comes before exit 2, exit 5 and the canary.
3. The **tick definition** (R-17) comes before the FP-1 bounds, the SI-23 "one tick" bound, the RK-1 pull and the component heartbeat.
4. The **supervisor plus component heartbeat** (R-8) comes before GP-12 failure 4 and H-19.
5. The **hub signing key** (R-61) comes before the **revocation list** (RK-1/SI-11), and **PR-1a registration** comes before both.
6. **Hub consumer visibility** (SI-16) comes before GP-12 failure 2. It is a hub change, as is FX-6.
7. **Needs-attention surface** (R-37) comes before FX-1, FX-8/CLEAR and every FP-1 "needs-attention entry".
8. The **readiness re-check plus activity veto** (SI-2, T-3387) comes before the doorbell (SI-1), which comes before W2 and H-11.
9. **`up_to` plus the seq window** (SI-6, SI-5) comes before H-3.
10. **W4** (switching off the AEF receiver and notify-sidecar) waits for W3 acceptance with a negative control. The **drain procedure** (6.2.3.e: refuse new mail, re-post STORED messages to the old inbox) must exist before W1. It has no code (grep: none).

Open questions for the operator:

- **OQ-a** Does a separate `termlink-sidecar` binary satisfy OD-9 C, or must PH-2 fold it into `termlink sidecar`?
- **OQ-b** Production identity: agents run as root on this host (ARCH 3.1). Does PH-2 accept a root-only owner check, or does it wait for Track A?
- **OQ-c** STUCK deadline: 10 min as built, or about 1 min as in R-44.e and FP-1?
- **OQ-d** Tick length: is R-17's 30 s the unit for every PH-2 bound, given the 500 ms poll as built?
- **OQ-e** Kill switch fleet scope: one admin socket per agent driven by a runme loop, or a fleet flag?
- **OQ-f** Who supervises the sidecar: systemd, the reachable launcher, or the existing notify-sidecar supervisor pattern?
- **OQ-g** FX-5 baseline: is the PH-0 plaintext-field count recorded anywhere? If not, it must be taken before W1.
- **OQ-h** Where does the hub's revocation list live, and who signs revocations before an operator key exists (TM RK-1: "the operator, or the agent's own project")?
- **OQ-i** OQ-3 is still marked recommendation A with no "Ruled" text (ARCH 279), although ACC ran as `testdev`. Does PH-2's canary also need `testdev`?

## D. Contradictions between design and code

1. ARCH 2.1 (line 40) and the ARCH 6.2 hook commands (line 145) say `termlink sidecar ...` subcommands in termlink-cli. The code is the separate `termlink-sidecar` binary (main.rs). ARCH 0.5 records the change but 2.1 and 6.2 were not updated. FP 6.2.3.a (1) also says `termlink sidecar`.
2. ARCH C-10 (line 53) names `stage.rs`. There is no such file; stage logic lives in api.rs, deliver.rs and wire.rs (`NOTICES` at wire.rs:32).
3. ARCH 7.3 (line 222) gives `delivery off [--agent X|--all]`. The code is a per-store flag with no `--agent` or `--all` (api.rs:525-545), and client.rs offers no `admin delivery` verb (client.rs:20-22 list held, allow and deny).
4. STUCK_AFTER_MS = 10 min (api.rs:65; ARCH 0.5) conflicts with R-44.e (REQ 796: "within 2 ticks … about 1 min") and with the FP-1 INC-a bound (one tick after a deadline of about 1 min).
5. ARCH 3.2's `handed-over` row (line ~92) still lists `transcript_path` as a field. ARCH 0.4 says it takes `nonce` and no verb takes a path. The table row was not updated.
6. ARCH 8.3 calls the path-open audit "strace-free". ARCH 0.9 and ACC say it ran under `strace -f`. Both forms are claimed; the design text is stale.
7. ARCH 1.1 lists `known_peers.rs` as "Reuse as the PH-1 key roster". The roster in code is `wire::Roster` in daemon.rs and policy.rs. Whether known_peers is used is UNVERIFIED.
8. READY_MAX_AGE_MS = 24 h (deliver.rs:164) is a looser readiness than SI-2 ("re-checked just before"). It is harmless in PH-1, where nothing is typed, but it must not be reused for the doorbell.
