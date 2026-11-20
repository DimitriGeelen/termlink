# T-3708 spec extract — P-24 / B-14: status strength flags, reachability card, CLEAR, skew, GP-4 lines

Prepared 2026-11-20 (research only; build in a later session). Paths relative to `docs/design/` unless stated.

## Spec sources

1. Plan row `08-plan.md:167`: P-24, B-14, R-6, R-7, R-29, R-44, R-45, R-53, R-57, R-70, R-82; evidence E-19, E-55..E-61,
   E-134; depends P-12 (built 2026-11-20), P-15 (built), P-23 (skew line reads the NTP state); exit "B-14 specs pass;
   status answers in 5 s; fixture suite PASS line per E-n, mutants all killed"; rollback "fields hidden; status as before".
2. Pseudo-code AL-29, `06a-pseudo-code-ph2-hardening.md:502-534` (2.13): every field = (value, produced_at mono, bound:
   2 ticks per-tick / 60 s heartbeat / 1 h daily); render "unknown (last at T)" past the bound (FCW-120); bad fields
   (hub.match false/unknown, revocation stale, consumers > 1, supervision dead, quota refusing receipts, last_surface
   bad, any component heartbeat older than 3 ticks); last_surface bad (V-8) = a STORED message not handed over 2 ticks
   after READY or after a hook call that did not hand it over; idle with nothing pending is never bad; CLEAR iff no field
   bad or unknown and (nothing pending in the last hour or a HANDED_OVER with evidence after the earliest pending STORED)
   (FCW-121); reachability card {receiver_up, right_hub, adapter_present (valid adapter report within 24 h),
   last_surface} published with the liveness note every tick; a reader with a card older than 2 × PN-9 sees UNKNOWN,
   never DEAD (FCW-122); strength flags computed per call, an uncomputable flag reads as absent (FCW-123); skew from hub
   round trips: "clock skew" > 2 s, cross-host delays "unusable" > 30 s, NTP from timedatectl/chronyc else "unchecked".
   Edge cases: status reads in-memory fields only (no blocking RPC) so p95 ≤ 5 s under faults; freshness on mono();
   after a restart every field is absent → unknown, so a restarted sidecar is never CLEAR until it has re-measured.
3. Architecture 6.7.1–6.7.5 (`04-architecture.md:523-542`): field list `mode, binary, supervision, main,
   hub {id, declared, match}, revocation, quota, consumers, reachability, clear, strength {per_agent_accounts: no,
   same_user_separation: "absent: agents run as root", operator_key: no, confirmation_provider: no, policy_records:
   "none (fixed defaults)"}`; card readers: own project and operator; other projects see reachable yes/no + version class
   (read scope enforced PH-3). Proof: the 32,205-refusal injection reads NOT CLEAR.
4. Evidence (`05-evidence.md`): E-19 (`:354`, activity fresh or unknown, freeze the meter), E-55 (`:642`, all fields
   stamped, p95 ≤ 5 s over 100 calls per E-63 fault), E-56 (`:649`, CLEAR never a heartbeat alone), E-57 (`:656`, V-8
   threshold, idle 48 h not bad, cut channel UNKNOWN), E-58 (`:663`, strength flags vs `ps -o user=`), E-59 (`:670`, skew
   0/3/45 s, ±1 h wall jump, deadlines on mono), E-60 (`:677`, GP-4 offline visible: WAITING "home hub unreachable since
   T", no RECEIVED from the outbox, no duplicates on restore), E-61 (`:684`, card content = stage log, last_surface never
   a heartbeat), E-134 (`:1311`, status names the cause within 60 s and answers in 5 s; latency log).
5. Carried into T-3708 from T-3703 reviews (task ACs): "key revoked, operator action needed" WAITING reason (RK-1);
   E-127's reader (`refused_revoked` counts); the `revocation` status vocabulary; the DOWNGRADED line naming
   `revocation list stale`.

## What exists today (code, 2026-11-20)

`crates/termlink-sidecar/src/api.rs::status_at` already returns `mode`, `strength` (+ `missing_strength`),
`supervision`, `hub`, `hub_id`, `hub_signing_key`, `revocation`, `consumers` (P-12, with `checked_ms`), and the live path
adds `binary`, `main` (+ liveness, lease, mail_waiting), `activity`, `doorbell`, `keys`, `vendor_log`. The liveness note
(T-3646) is the base the reachability card generalises.

## Missing for P-24 (first cut, to be confirmed in the design)

1. A uniform freshness stamp per field and the bound/`unknown (last at T)` render (FCW-120); today only some fields carry
   a time (`consumers.checked_ms`, revocation "at T").
2. `quota`, `reachability` (card published with the liveness note; reader-side UNKNOWN past 2 × PN-9), `clear`.
3. V-8 last-surface tracking (READY / hook-call times vs STORED not handed over).
4. Skew estimate from hub round trips + NTP state line (P-23 built the NTP check).
5. Strength flags computed per call from the host (process owners), absent when not computable (FCW-123); compare with
   today's static `strength`.
6. GP-4 offline lines (E-60): WAITING "home hub unreachable since T".
7. The four carried T-3703 items above.

## Size note

Nine E-n at L2/L4 with a p95 latency bound under injected faults and an L4 network cut (E-60). Likely two slices:
(a) freshness/CLEAR/V-8/strength/skew in status (E-19, E-55, E-56, E-57, E-58, E-59, E-134), (b) reachability card +
GP-4 offline (E-60, E-61). To be proposed in the design with the operator.
