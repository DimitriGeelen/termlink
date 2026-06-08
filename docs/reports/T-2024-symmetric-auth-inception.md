# T-2024 Inception Research — Substrate primitive #6: symmetric authentication across UDS + TCP

**Status:** DEFER with measurement-first plan. Spike latency under concurrent-agent load before locking the migration. Resume the decision with data.
**Artifact created:** 2026-06-08
**revisit_at:** 2026-09-08 (90 days — gives Foundation primitives time to ship and surface real concurrency-driven incidents)
**revisit_evidence_needed:** Latency-spike numbers under concurrent-agent load (≥10 simultaneous clients); concrete UID-trust incident or audit finding; explicit operator decision to retire the privileged sidecar.
**See also:** T-2018 ADR §6 #6, §7 (transport unification); CLAUDE.md "Hub Auth Rotation Protocol".

## 1. The §6 framing

ADR §6 primitive #6 + §7: *"Same-host UDS today is auth-bypassed (UID trust); cross-host is HMAC + cert pinning. Two code paths, two trust models, and a long-lived privileged sidecar UDS listener. §7 decision: unify on one authenticated path — loopback TCP same-host, TCP cross-host, both HMAC + cert pinning."*

This is the only §6 primitive that's a SECURITY ARCHITECTURE CHANGE. It is not adding a verb; it is retiring a transport. Bigger blast radius than the verb-shaped primitives, and the failure mode (broken local clients post-cutover) is operator-visible, not a quiet substrate gap.

## 2. What the substrate has today

- **Cross-host:** TCP with HMAC + cert pinning. Working, exercised daily on ring20 (5 hubs, frequent rotation, T-1051 protocol documented). This path is the well-trodden one.
- **Same-host UDS:** `/tmp/termlink-0/hub.sock` (or `<runtime_dir>/hub.sock`). UID-trust — anything that can `connect()` to the socket file is implicitly authenticated. The privileged listener runs under the same user as the hub.
- **Secret/cert material:** Per CLAUDE.md and T-933 / T-1051, hubs persist `hub.secret` + `hub.cert.pem` + `hub.key.pem` under their `runtime_dir`. Local profiles in `hubs.toml` can point `secret_file` at the hub's authoritative file directly (R3 — read-live, not cache).

So **the cross-host path's primitives already exist on localhost** — the hub's HMAC secret is local-readable, the cert is local-readable, the TCP port is local-bindable. The unification is *not blocked by missing infrastructure*; it's blocked by inertia and by an unmeasured latency assumption.

## 3. Why the measurement question matters

§7 asserts loopback TCP latency is negligible at homelab scale. That's *probably* true — kernel loopback adds a few microseconds vs UDS for typical payloads, and TermLink's RPC pattern is request/response on the order of milliseconds to hundreds of milliseconds (channel.subscribe is the longest poll, channel.post is fast). For the typical workload, the latency delta is invisible.

But T-2019..T-2021 *increase* concurrent client load on the local hub. The substrate is built for "≤30 agents per fleet" (ADR §1) but the concurrency floor was ~3-5 active clients during prior measurement. Going to 10-30 *simultaneous* clients hitting the same hub may surface a different latency profile — kernel scheduling, TLS handshake cost, syscall overhead — than the small-N case.

**No measurement has been run.** §7's assertion is pre-evidence. Acting on it without data is the same shape as the T-1991 "agent-presence topic bloat" assumption that turned out to be wrong (per-binary-version slowdown, not topic-size). The framework's stance is to spike, then decide.

## 4. Migration risk surface

Even with favorable latency, cutover has known failure modes:

| Risk                                                | Mitigation                                        |
|-----------------------------------------------------|---------------------------------------------------|
| Local clients can't find the new secret/cert path   | Standardize on `~/.termlink/hubs.toml` → `secret_file = "<runtime_dir>/hub.secret"` per R3 |
| Framework-pickup-bridge / framework-listener break  | Stage migration: TCP path lives ALONGSIDE UDS for ≥1 release; deprecation warnings; auto-switch flag |
| TLS handshake adds wall-clock cost                  | Use cached connections; investigate TLS session resumption |
| `localhost` resolution edge cases (IPv4 vs IPv6)    | Bind explicitly to `127.0.0.1:<port>`, not `localhost` |
| Restart loop on rotation                            | Same persist-if-present pattern as cross-host (R3 already covers it) |

The migration is not unreversible (UDS code can stay deployed during the deprecation cycle), but it's not trivial either. The risk surface is comparable to a CLI flag rename, not comparable to a verb addition.

## 5. Open questions, partial dispositions

- **IW-1 (loopback latency vs UDS — confirmed negligible per §7, but measure under concurrent-agent load):** UNRESOLVED — **this is the gate**. Confidence=1. Spike before deciding.
- **IW-2 (cert pinning store location for loopback case — same `KnownHubStore` or separate):** SAME STORE. Loopback hubs are just hubs with `address = 127.0.0.1:port` — `KnownHubStore` is already keyed by address. No new store needed. Confidence=4.
- **IW-3 (migration — how does an existing UDS-only deployment upgrade without downtime):** STAGED COEXISTENCE. Phase-1: ship TCP path alongside UDS, no behavior change. Phase-2: clients opt into TCP via config flag, default still UDS. Phase-3: flip the default, UDS deprecation warning. Phase-4: remove UDS. Each phase = ≥1 release. Confidence=3 (the shape is clear; per-phase task scoping needs to be done at GO time). See artifact §4.

## 6. Recommendation: DEFER with measurement-first

**DEFER with two-track unblock plan:**

**Track A (measurement spike, can run now, ≤1 session):**
- Set up a local hub with both UDS and TCP listeners on loopback.
- Drive synthetic concurrent load: 10 clients × `channel.subscribe` + 10 clients × `channel.post` + 10 clients × `channel.claim/release` cycles.
- Measure: p50/p95/p99 round-trip latency for each path; CPU/syscall overhead; connection-establish cost.
- Reporting: write `docs/reports/T-2024-latency-spike.md` with raw numbers + recommendation update.

**Track B (re-decide based on data, after Track A):**
- If TCP-loopback p99 ≤ 2× UDS p99 → **GO**, file Phase-1 build task (coexistence path).
- If TCP-loopback p99 > 5× UDS p99 → **NO-GO**, document the perf gap, leave UID-trust UDS in place, but file a separate task to add audit logging on the UDS path so the trust-model gap is *observable* even if not eliminated.
- If between 2x and 5x → operator judgment call, with raw numbers in hand.

**Why not GO now:** §7 asserts the answer; the framework's posture is to verify assertions before locking architecture changes. The cost of a measurement spike (≤1 session) is small relative to the cost of cutting over and discovering a latency regression after some local-tool's UX degraded.

**Why not NO-GO:** The trust-model gap is real. The implicit UID-trust on UDS is a defect-in-depth — fine while there's exactly one privileged user per host, but the substrate is opening up to "vendored sidecar agents" (T-1898) and other-user clients in the future. Unifying on HMAC+cert is the right destination.

## 7. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §3 "durable channel logs are the primary recovery story" | ✓ Unchanged — transport unification doesn't touch storage. |
| §6 #6 framing | ⚠ Captures the destination correctly but asserts the latency answer without measurement. This artifact corrects that. |
| §7 transport unification claim | ⚠ Re-stated as a measurement question. |
| §9 "hard-dep for AEF" | ✓ AEF doesn't care which transport is used; it cares that auth is symmetric and audited. Either GO or NO-GO satisfies the AEF need (NO-GO with audit-logging mitigation also satisfies it). |

## 8. Open follow-up tasks to file

- **On DEFER (immediate):** spike task — "T-2024 latency-measurement spike: TCP-loopback vs UDS under concurrent-agent load". Owner: any agent. ≤1 session. Output: numbers + decision recommendation.
- **On Track A → GO:** Phase-1 build task — coexistence path (TCP listener alongside UDS, opt-in via config flag). ~200 LOC.
- **On Track A → GO:** Phase-2/3 build tasks — flip default, deprecation cycle. Each ~100 LOC.
- **On Track A → NO-GO:** audit-logging task — add structured audit log on the UDS path so the trust-model gap is observable. ~50 LOC.

## 9. Why this verdict differs from T-2020 / T-2021 / T-2025 / T-2027

The other §6 primitives this session collapsed via *substrate inspection*: the primitive either already existed (T-2025 presence), or was a composition of existing verbs (T-2020 registry, T-2021 pull), or was a small additive read pattern (T-2027 from-latest). T-2024 is genuinely a transport-and-trust-model change: it can't be done by composition, it can't be measured by inspection, and the failure modes affect every local client. The honest verdict is "spike, then decide" — not "GO with revised scope" and not "NO-GO".
