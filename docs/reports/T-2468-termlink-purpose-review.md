# T-2468 — Critical Review of TermLink's Purpose & Goals

**Inception task:** T-2468
**Started:** 2026-07-31
**Question:** Where does TermLink's implementation diverge from its stated purpose,
and which divergences warrant action?

## Method

Four parallel review dimensions (IW-1..IW-4), each producing ranked gaps with
file:line evidence, synthesized into a prioritized gap register (IW-5). Yardstick:
the Four Constitutional Directives (Antifragility, Reliability, Usability, Portability).

## Canonical purpose statements

- `README.md:3-4` — "A coordination substrate for parallel AI agents — a hub-mediated,
  durable append-log message bus with terminal endpoints."
- `README.md:11-13` — "It began as a cross-terminal session-control tool and grew into
  the coordination layer for multi-agent parallel execution" (mission-evolution admitted
  in the canonical doc).
- `docs/architecture/parallel-execution-substrate.md:24-27` — "most of the substrate's
  job is not 'isolate trees' but 'provide the primitives that let the orchestrator assign
  disjoint work and detect when disjointness is violated.'"
- `docs/ARCHITECTURE.md:3` — "Cross-terminal session communication system in Rust …
  enables multiple terminal sessions to communicate." (older purpose, still canonical, unretired)

**Key correction (IW-1):** The Four Constitutional Directives (Antifragility/Reliability/
Usability/Portability) belong to the **AEF meta-framework**, NOT to TermLink-the-product.
TermLink has **no product-level charter of its own** — its purpose lives only in README +
the substrate ADR, and those two disagree. This itself is finding #1.

## Findings by dimension

### IW-1 — Purpose–reality alignment

- **[HIGH] Two live, non-identical purpose statements coexist unreconciled** — "coordination
  substrate for parallel AI agents" (README.md:3) vs "cross-terminal session communication
  system" (docs/ARCHITECTURE.md:3). A new user gets a different product depending which doc
  they open. Drift documented but not resolved.
- **[HIGH] OVER-SERVICE: ~57 of 276 MCP tools (~21%) are Slack-clone social analytics** —
  emoji_stats, reaction_rate, star/pin/poll, thread_health, top_reacted, top_pinners,
  co_posters, post_streak, typers. No substrate purpose requires reaction leaderboards. Pure
  mission creep. (evidence: 276 tools total, ~57 social-analytics.)
- **[HIGH] UNDER-SERVICE: the substrate's own stated core job — collision detection on
  disjoint write-sets — is unbuilt.** ADR §2: "No filesystem-write observation, anywhere…
  the hub cannot see what files an agent touches." The system that exists to "detect when
  disjointness is violated" cannot observe writes.
- **[MED] Scope formally undecided (multi-tenant)** — trust model is single-operator
  ("treat every authenticated peer as trusted", README.md:53) yet concerns.yaml:480 still
  asks whether multi-tenant is in scope. Unowned purpose boundary.
- **[MED] Task/verb sprawl is now a cost** — 2097 completed tasks, ~40 CLI/skill verbs,
  ~39k tokens/agent of MCP descriptions (arc-005 exists because the surface is too large).
- **[LOW] README claims "30 commands"** while real surface is 276 MCP tools + ~40 skills —
  headline understates sprawl (README.md:82).

Recommended inceptions: (1) reconcile into one canonical TermLink charter [Usability];
(2) social-analytics decommission-or-justify review [recovers context budget];
(3) close-or-descope collision-detection under-service [Reliability].

### IW-2 — Comms rail reliability (the doorbell replacement)

Round-trip map: **DELIVER** = durable/guaranteed (offline-queue T-2051, dedupe T-2049);
**WAKE** = best-effort (WS push over poll floor, drops on Lagged / pre-subscribe;
depends on out-of-band pushwaker that may be dead); **CONSUME** = **NOT guaranteed, silent
break** (a busy/manual-accept session gets the inject unsubmitted, discarded on next
`--continue`; LIVE+pty_session ≠ listening); **ACK** = observable + retry-enforced but ONLY
via `--await-ack` path (raw `inject`/bare post bypass it); **REPLY** = app-level.

- **[HIGH] Wake-consumption is unconfirmed (rung-but-not-read fails silently)** — the
  load-bearing hole (G-083). Heartbeat proves process-alive, not session-listening; no
  post-ring read-cursor check. Blind since the doorbell shipped (T-1800).
- **[MED] Exactly-once does not survive hub restart / >5-min blip** — hub dedupe is
  in-memory (5-min TTL) while client replays client_msg_id durably → double-append on
  restart. ADR §5 overstates the guarantee (G-088, incepted T-2459).
- **[MED] WS push drops lagged/pre-subscribe wakes with no gap signal** — degrade-to-poll
  contract undocumented/unenforced, no cursor on ws_subscribe (T-2371, open).
- **[MED] Shipped≠live: rail depends on out-of-band wakers/binaries that may never run**
  (G-069) — mitigated only by crons that must themselves fire.
- **[MED] Ack-with-retry only fires on the agent-send path** — enforced leg is opt-in, not
  structural (T-2286).
- **[LOW] Reliability substantially achieved by a canary pile** — observers, not closers;
  the consumption-confirmation primitive is still unbuilt.

Recommended inceptions: (1) consumption-confirmation as next loud-contract link (closes
G-083, highest value) [Reliability]; (2) persist await-ack dedupe to runtime_dir OR
downgrade ADR §5 to at-least-once + idempotent consumer [Reliability]; (3) enforce WS
degrade-to-poll invariant [Antifragility].

### IW-3 — Coordination substrate: coherence vs bloat

Inventory: **276 MCP tools** in one 45,619-line tools.rs, ~140 tokens each = ~39k
tokens/agent, flat-loaded, no gating. Only **19 of 276 are substrate tools** (8 actuators
+ 11 pure read/history). Slice multiplier: CLI + watch + notify + log + history-CLI +
history-MCP + MCP + slash ≈ 8-9 slices × 6 primitives ≈ ~50 surfaces; **15 substrate slash
verbs** alone.

- **[HIGH] Work-stealing has ~1 lifetime real claim, which expired.** `claims-summary
  work-queue` → `active=0 expired=1`; the one claim ever placed was never completed.
  orchestrator-backlog-drain.sh defaults to `--dry-run`; only `--live` smoke tests exist
  (T-2204 calls itself "first in-tree consumer"). Dispatch substrate = solution built ahead
  of any recurring problem.
- **[HIGH] The watch/notify/log/history tier is ritual, not load-bearing.** The four log
  files the `--log` slices read (`~/.termlink/{claims,find-idle,governor,queue}.log`) **do
  not exist** — every history verb reads a never-written file. ~30+ surfaces, zero production
  reads.
- **[HIGH] 39k tokens/agent for ~7% substrate + 276-tool flat catalog = Usability harm**
  (choice paralysis, unconditional context tax). Near-zero value-per-token for the 11
  substrate read/history MCP tools no agent invokes.
- **[MED] Symmetry-for-symmetry's-sake** — each primitive got full CLI+MCP+watch+notify+log+
  history "for parity" (CLAUDE.md rows repeatedly "mirror of T-XXXX"). Parity with an unused
  sibling is self-justifying scope.
- **[MED] Only offline-queue (#5) shows real passive use** (`outbound.sqlite`, 28KB live).
  governor/cv-index plausibly load-bearing as internal hub telemetry, not as agent MCP tools.
- **[LOW] 180 commits touch claim/dispatch keywords** — nearly all building/hardening the
  substrate itself, not using it. Effort is recursively self-directed.

Recommended inceptions: (1) substrate MCP gating/lazy-load — drop 11 read/history tools from
default manifest, reclaim 2-4k tokens/agent, prime arc-005 [Usability/Portability]; (2)
collapse watch/notify/log/history into one `substrate tail` or delete dead slices
[Reliability/Antifragility]; (3) "prove or retire dispatch #1/#2/#3" go/no-go — one real
drain run or a dated mothball decision [Reliability/Usability].

### IW-4 — Adoption / shipped≠live / portability

Pipeline: merged→OneDev (manual 1 step) → GitHub mirror (auto, silently fails G-058) →
tag→binaries/Releases (auto) → **binary on each hub (MANUAL, per-host, foothold-gated)** →
**swap+restart (MANUAL)** → **wakers re-armed (MANUAL, separate)** → verified (auto-DETECT
only, 11 canaries). **The release half is automated; the fleet-adoption half is entirely
manual, per-host, foothold-gated.** `fleet-deploy-binary.sh` takes ONE hub — "fleet-deploy"
is a misnomer.

- **[HIGH] No fleet-wide "ensure version X live everywhere" primitive — the canary count IS
  the symptom.** 11 detection canaries each born reactively after a silent field failure;
  none *close* the gap. No `fleet upgrade --all --restart --rearm` verb.
- **[HIGH] "Shipped" ≠ capability-live by definition — arc-closure gap OPEN.** arc-closing
  tasks have no capability-live Verification (G-069, concerns.yaml:282,299). arc-004 was
  closed=shipped while 0/2 prod hubs served the rails. "Done" omits "live" → drift is
  structural.
- **[MED] Version floors are hand-maintained state that itself goes stale.** T-2465: .122 sat
  279 commits stale yet PASSED the old floor — a stale floor = a blind canary.
- **[MED] Foothold fragility = un-upgradeable hosts.** .121 is EXEMPT purely because "no
  execution foothold." A reachable, floor-relevant hub that cannot be made live from anywhere.
- **[MED] .107 is a single point for deploy orchestration** — all binaries built+pushed FROM
  .107; footholds originate there; arc-004 `--expect-armed` declared only for .107.
- **[LOW] glibc/musl matrix is a silent-fail deploy footgun** (PL-100), mitigated by
  musl-default+`--probe` but still a per-host burden.

Portability of *code* holds (MCP, musl-static, git standards); portability of *operations*
does not (per-host SSH/systemd/foothold assumptions).

Recommended inceptions: (1) "make-it-live" fleet primitive `fleet upgrade --to <ver> --all`
(stage→swap→restart→rearm→verify, idempotent) — collapses the canaries' cause [Reliability];
(2) arc/task closure requires a capability-live gate before `closed=shipped` [Antifragility];
(3) foothold-independent adoption transport (self-pull agent / standing upgrade channel) so
.121 isn't un-upgradeable and .107 isn't a single origin [Portability].

## Prioritized gap register (IW-5 synthesis)

Ranked by (value × tractability). **Gov** = governance disposition.

| # | Gap | Dimensions | Severity | Gov |
|---|-----|-----------|----------|-----|
| **P1** | **No single product charter** — README ("coordination substrate for parallel agents") vs ARCHITECTURE.md ("cross-terminal session comm system") contradict; TermLink has no owned one-sentence purpose. Foundational — every cut/build decision hangs off it. | IW-1 | HIGH | Doc reconcile; human blesses the canonical sentence |
| **P2** | **Wake→consume silent break (G-083)** — the core value prop (reliable comms) has a load-bearing hole: a message can be rung but never read, nothing surfaces it. Heartbeat proves process-alive, not session-listening. | IW-2 | HIGH | New mechanism → inception + GO |
| **P3** | **Shipped≠live has no structural gate (G-069)** — "done" ≠ "live"; 11 canaries detect, none prevent; no make-it-live primitive; adoption half fully manual. | IW-2, IW-4 | HIGH | Gate = policy+tooling; primitive = larger GO |
| **P4** | **MCP/substrate over-service** — 276 tools/39k tokens flat-loaded; ~57 (21%) social-analytics trace to no purpose; substrate observability tier (~30 surfaces) reads never-written logs; work-stealing has ~1 real claim ever placed (expired). | IW-1, IW-3 | HIGH | Deletion needs GO (reversible); gating additive; overlaps arc-005 |
| **P5** | **Exactly-once weaker than advertised (G-088/T-2459)** + WS degrade-to-poll unenforced (T-2371). ADR §5 overstates the guarantee. | IW-2 | MED | Already incepted/tracked |
| **P6** | **Substrate's own stated core job unbuilt** — write-set collision detection can't observe writes (ADR §2); + multi-tenant scope formally undecided. | IW-1 | MED | Build-or-descope decision → GO |

## Recommendation

**Overall: GO on a prioritized, staged subset — not "build everything."** The review's dominant
finding is that TermLink over-built breadth while leaving its core promises (legible purpose,
reliable round-trip, live-across-fleet) incomplete. The correct response is **subtract and
deepen**, not add.

Per-gap recommendation:
- **P1 (charter): GO now** — cheapest, foundational, reversible. Reconcile to the already-primary
  README purpose; retire the stale ARCHITECTURE.md framing. (Human picks the canonical sentence.)
- **P4 (surface reduction): GO now, staged** — biggest concrete Usability/context win; start with
  the ~57 directive-untraceable social-analytics tools + the ~30 never-read substrate
  observability surfaces. Reversible via git. Coordinate with arc-005.
- **P2 (consumption-confirmation) + P3 (shipped≠live gate/primitive): GO-to-inception** — highest
  reliability value but need design; spawn as their own single-question inceptions.
- **P5: continue** existing tracked work (T-2459 / T-2371).
- **P6: DEFER pending a product decision** — build collision-detection or formally declare it a
  non-goal; tied to the multi-tenant scope question.

**Governance note:** This is an inception. Per inception discipline no build artifacts are
written under T-2468. On a human GO, each greenlit item becomes its own separate build/inception
task, then build+test+drive-to-completion.
